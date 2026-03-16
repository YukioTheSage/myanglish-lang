import * as path from "path";
import * as fs from "fs";
import * as os from "os";
import { promises as fsp } from "fs";
import * as vscode from "vscode";
import { execFile } from "child_process";
import { promisify } from "util";

import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from "vscode-languageclient/node";

let client: LanguageClient | undefined;
const execFileAsync = promisify(execFile);

function getWorkspaceBinaryCandidates(binaryName: string): string[] {
  const folders = vscode.workspace.workspaceFolders ?? [];
  const roots = new Set<string>();

  for (const folder of folders) {
    roots.add(folder.uri.fsPath);
    // Support monorepo layout where the Rust crate is nested under "mlang/".
    roots.add(path.join(folder.uri.fsPath, "mlang"));
  }

  const candidates: string[] = [];
  for (const root of roots) {
    candidates.push(path.join(root, "target", "release", `${binaryName}.exe`));
    candidates.push(path.join(root, "target", "release", binaryName));
    candidates.push(path.join(root, "target", "debug", `${binaryName}.exe`));
    candidates.push(path.join(root, "target", "debug", binaryName));
  }

  return candidates;
}

function resolveFormatterPath(config: vscode.WorkspaceConfiguration): string {
  const configured = config.get<string>("formatter.path", "").trim();
  if (configured) {
    return configured;
  }

  const candidates = getWorkspaceBinaryCandidates("mlang");

  for (const candidate of candidates) {
    if (fs.existsSync(candidate)) {
      return candidate;
    }
  }

  return "mlang";
}

async function runFormatter(
  document: vscode.TextDocument,
  outputChannel: vscode.OutputChannel,
): Promise<string> {
  const config = vscode.workspace.getConfiguration("mlang");
  const formatterPath = resolveFormatterPath(config);
  const extraArgs = config.get<string[]>("formatter.args", []);

  const tmpDir = await fsp.mkdtemp(path.join(os.tmpdir(), "mlang-fmt-"));
  const tmpFile = path.join(
    tmpDir,
    `document${path.extname(document.fileName) || ".ml"}`,
  );

  try {
    await fsp.writeFile(tmpFile, document.getText(), "utf8");
    const args = [...extraArgs, "fmt", tmpFile];
    await execFileAsync(formatterPath, args, {
      cwd:
        vscode.workspace.getWorkspaceFolder(document.uri)?.uri.fsPath ??
        vscode.workspace.workspaceFolders?.[0]?.uri.fsPath,
    });

    return await fsp.readFile(tmpFile, "utf8");
  } catch (error) {
    const e = error as { stderr?: string; message?: string };
    const details = e.stderr || e.message || "Unknown formatter error";
    outputChannel.appendLine(`Formatter failed: ${details}`);
    throw new Error(details);
  } finally {
    await fsp.rm(tmpDir, { recursive: true, force: true });
  }
}

export async function activate(context: vscode.ExtensionContext) {
  const config = vscode.workspace.getConfiguration("mlang");
  let serverPath = config.get<string>("server.path", "");

  if (!serverPath) {
    // Try to find mlang-lsp relative to the workspace
    const candidates = getWorkspaceBinaryCandidates("mlang-lsp");

    for (const candidate of candidates) {
      if (fs.existsSync(candidate)) {
        serverPath = candidate;
        break;
      }
    }

    // Fall back to PATH
    if (!serverPath) {
      serverPath = "mlang-lsp";
    }
  }

  const outputChannel = vscode.window.createOutputChannel(
    "M-Lang Language Server",
  );
  outputChannel.appendLine(`Starting mlang-lsp from: ${serverPath}`);

  const serverOptions: ServerOptions = {
    run: {
      command: serverPath,
      transport: TransportKind.stdio,
    },
    debug: {
      command: serverPath,
      transport: TransportKind.stdio,
    },
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "mlang" }],
    synchronize: {
      fileEvents: vscode.workspace.createFileSystemWatcher("**/*.ml"),
    },
    outputChannel,
    // Prevent vscode-languageclient from registering its own formatting
    // provider — we handle it explicitly so format-on-save works reliably.
    middleware: {
      provideDocumentFormattingEdits: () => undefined,
    },
  };

  client = new LanguageClient(
    "mlang-lsp",
    "M-Lang Language Server",
    serverOptions,
    clientOptions,
  );

  let lspRunning = false;
  try {
    await client.start();
    lspRunning = true;
    outputChannel.appendLine("M-Lang language server started successfully.");
  } catch (e) {
    const msg = e instanceof Error ? e.message : String(e);
    outputChannel.appendLine(`Failed to start language server: ${msg}`);
    vscode.window.showErrorMessage(
      `M-Lang LSP failed to start. Check that mlang-lsp is built.\n` +
        `Tried path: ${serverPath}\n` +
        `Run: cargo build --release --bin mlang-lsp`,
    );
  }

  // Always register our own formatting provider so format-on-save works
  // reliably. When LSP is running we forward the request over the protocol;
  // otherwise we fall back to the CLI formatter.
  context.subscriptions.push(
    vscode.languages.registerDocumentFormattingEditProvider("mlang", {
      async provideDocumentFormattingEdits(
        document: vscode.TextDocument,
        options: vscode.FormattingOptions,
        token: vscode.CancellationToken,
      ): Promise<vscode.TextEdit[]> {
        const cfg = vscode.workspace.getConfiguration("mlang");
        if (!cfg.get<boolean>("formatter.enabled", true)) {
          return [];
        }

        // --- LSP path (fast, in-process) ---
        if (client && client.isRunning()) {
          try {
            const params = {
              textDocument:
                client.code2ProtocolConverter.asTextDocumentIdentifier(
                  document,
                ),
              options: client.code2ProtocolConverter.asFormattingOptions(
                options,
                {},
              ),
            };
            const result = await client.sendRequest(
              "textDocument/formatting",
              params,
              token,
            );
            if (!result) {
              return [];
            }
            return (result as any[]).map((edit: any) => {
              const range = new vscode.Range(
                edit.range.start.line,
                edit.range.start.character,
                edit.range.end.line,
                edit.range.end.character,
              );
              return new vscode.TextEdit(range, edit.newText);
            });
          } catch (err) {
            const msg = err instanceof Error ? err.message : String(err);
            outputChannel.appendLine(`LSP formatting failed: ${msg}`);
            // fall through to CLI fallback
          }
        }

        // --- CLI fallback ---
        try {
          const formatted = await runFormatter(document, outputChannel);
          if (formatted === document.getText()) {
            return [];
          }

          const fullRange = new vscode.Range(
            document.positionAt(0),
            document.positionAt(document.getText().length),
          );

          return [vscode.TextEdit.replace(fullRange, formatted)];
        } catch (error) {
          const msg = error instanceof Error ? error.message : String(error);
          vscode.window.showWarningMessage(`M-Lang format failed: ${msg}`);
          return [];
        }
      },
    }),
  );

  // Manual format command
  context.subscriptions.push(
    vscode.commands.registerCommand("mlang.formatDocument", async () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor || editor.document.languageId !== "mlang") {
        return;
      }
      await vscode.commands.executeCommand("editor.action.formatDocument");
    }),
  );

  context.subscriptions.push(
    vscode.commands.registerCommand("mlang.restartServer", async () => {
      if (client) {
        await client.restart();
        vscode.window.showInformationMessage(
          "M-Lang language server restarted.",
        );
      }
    }),
  );
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}
