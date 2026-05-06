use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use mlang::ast::{BlockStatement, Expression, Program, Statement, Type};
use mlang::codegen::CodeGenerator;
use mlang::codegen_go::GoCodeGenerator;
use mlang::codegen_llvm::generate_llvm_ir;
use mlang::formatter;
use mlang::module_loader::{load_entry_program, LoadedProgram};
use mlang::stdlib::resolve_stdlib_module;
use mlang::typecheck::{Environment, TypeChecker};

const LOCK_VERSION: u32 = 1;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct GoTargetOptions {
    goos: Option<String>,
    goarch: Option<String>,
}

#[derive(Debug, Clone)]
struct BuildArgs {
    target: String,
    file_path: Option<String>,
    go_target: GoTargetOptions,
}

#[derive(Debug, Clone)]
struct GetArgs {
    import_path: String,
    git_url: String,
    git_ref: String,
    entry: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct LockFile {
    #[serde(default = "default_lock_version")]
    version: u32,
    #[serde(default)]
    deps: Vec<LockDependency>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct LockDependency {
    import: String,
    #[serde(default)]
    git: String,
    #[serde(rename = "ref", default)]
    git_ref: String,
    #[serde(default)]
    commit: String,
    entry: String,
    cache_dir: String,
}

fn default_lock_version() -> u32 {
    LOCK_VERSION
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        return;
    }

    let command = &args[1];
    match command.as_str() {
        "fmt" => match parse_fmt_args(&args[2..]) {
            Ok((check_mode, file_path)) => match file_path {
                Some(path) => format_file(&path, check_mode),
                None => {
                    eprintln!("Error: No file specified for fmt command.");
                    print_usage();
                    std::process::exit(1);
                }
            },
            Err(err) => {
                eprintln!("Error: {}", err);
                print_usage();
                std::process::exit(1);
            }
        },
        "build" => {
            let parsed = match parse_build_args(&args[2..]) {
                Ok(v) => v,
                Err(err) => {
                    eprintln!("Error: {}", err);
                    print_usage();
                    std::process::exit(1);
                }
            };
            let Some(file_path) = parsed.file_path.as_deref() else {
                eprintln!("Error: No file specified.");
                print_usage();
                std::process::exit(1);
            };

            if compile_file(file_path, &parsed.target, &parsed.go_target).is_none() {
                std::process::exit(1);
            }
        }
        "run" => {
            let parsed = match parse_build_args(&args[2..]) {
                Ok(v) => v,
                Err(err) => {
                    eprintln!("Error: {}", err);
                    print_usage();
                    std::process::exit(1);
                }
            };
            let Some(file_path) = parsed.file_path.as_deref() else {
                eprintln!("Error: No file specified.");
                print_usage();
                std::process::exit(1);
            };

            if parsed.target == "go" && is_cross_target(&parsed.go_target) {
                let (goos, goarch) = resolved_go_target(&parsed.go_target);
                eprintln!(
                    "`mlang run` does not support cross targets (requested {} / {}). Use `mlang build --goos ... --goarch ...` instead.",
                    goos, goarch
                );
                std::process::exit(1);
            }

            if let Some(exe) = compile_file(file_path, &parsed.target, &parsed.go_target) {
                println!("--- Running {} ---", exe);
                let status = Command::new(&exe)
                    .status()
                    .expect("Failed to execute compiled binary");
                println!("--- Process exited with status: {} ---", status);
                if !status.success() {
                    std::process::exit(status.code().unwrap_or(1));
                }
            } else {
                std::process::exit(1);
            }
        }
        "test" => {
            let file_path = match parse_test_args(&args[2..]) {
                Ok(v) => v,
                Err(err) => {
                    eprintln!("Error: {}", err);
                    print_usage();
                    std::process::exit(1);
                }
            };
            if !run_test_file(&file_path) {
                std::process::exit(1);
            }
        }
        "get" => {
            let parsed = match parse_get_args(&args[2..]) {
                Ok(v) => v,
                Err(err) => {
                    eprintln!("Error: {}", err);
                    print_usage();
                    std::process::exit(1);
                }
            };
            if let Err(err) = install_dependency(&parsed) {
                eprintln!("Dependency error: {}", err);
                std::process::exit(1);
            }
        }
        _ => {
            print_usage();
            std::process::exit(1);
        }
    }
}

fn print_usage() {
    println!("M-Lang (Myanmar Language) Compiler");
    println!("Usage:");
    println!(
        "  mlang build <file.ml>                                  Build (default: LLVM native backend)"
    );
    println!("  mlang build --target c <file.ml>                       Build with C backend");
    println!(
        "  mlang build --target llvm <file.ml>                    Build with LLVM backend (native)"
    );
    println!(
        "  mlang build --target go <file.ml>                      Build with Go interop/backend"
    );
    println!("  mlang build --goos <os> --goarch <arch> <file.ml>      Cross-build Go target");
    println!("  mlang run <file.ml>                                    Build and run on host");
    println!(
        "  mlang run --target c <file.ml>                         Build and run with C backend"
    );
    println!(
        "  mlang run --target llvm <file.ml>                      Build and run with LLVM backend"
    );
    println!(
        "  mlang test <file.ml>                                   Run top-level set_sae tests"
    );
    println!("  mlang get --import <path> --git <url> --ref <ref> --entry <rel_ml_file>");
    println!(
        "  mlang fmt <file.ml>                                    Format source file in place"
    );
    println!("  mlang fmt --check <file.ml>                            Check if file is formatted");
}

fn parse_build_args(args: &[String]) -> Result<BuildArgs, String> {
    let mut out = BuildArgs {
        target: "llvm".to_string(),
        file_path: None,
        go_target: GoTargetOptions::default(),
    };

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--target" => {
                if i + 1 >= args.len() {
                    return Err("--target requires a value ('c', 'go', or 'llvm')".to_string());
                }
                out.target = args[i + 1].to_lowercase();
                i += 2;
            }
            "--goos" => {
                if i + 1 >= args.len() {
                    return Err("--goos requires a value".to_string());
                }
                out.go_target.goos = Some(args[i + 1].to_lowercase());
                i += 2;
            }
            "--goarch" => {
                if i + 1 >= args.len() {
                    return Err("--goarch requires a value".to_string());
                }
                out.go_target.goarch = Some(args[i + 1].to_lowercase());
                i += 2;
            }
            flag if flag.starts_with("--") => {
                return Err(format!("Unknown flag `{}`", flag));
            }
            value => {
                if out.file_path.is_some() {
                    return Err("Multiple input files provided".to_string());
                }
                out.file_path = Some(value.to_string());
                i += 1;
            }
        }
    }

    if out.target != "c" && out.target != "go" && out.target != "llvm" {
        return Err(format!(
            "Unknown target '{}'. Use 'c', 'go', or 'llvm'.",
            out.target
        ));
    }
    if out.target != "go" && (out.go_target.goos.is_some() || out.go_target.goarch.is_some()) {
        return Err("--goos/--goarch are only supported with `--target go`".to_string());
    }

    Ok(out)
}

fn parse_fmt_args(args: &[String]) -> Result<(bool, Option<String>), String> {
    let mut check = false;
    let mut file = None;
    for arg in args {
        if arg == "--check" {
            check = true;
        } else if arg.starts_with("--") {
            return Err(format!("Unknown flag `{}`", arg));
        } else if file.is_some() {
            return Err("Multiple input files provided".to_string());
        } else {
            file = Some(arg.clone());
        }
    }
    Ok((check, file))
}

fn parse_test_args(args: &[String]) -> Result<String, String> {
    let mut file = None;
    for arg in args {
        if arg.starts_with("--") {
            return Err(format!("Unknown flag `{}`", arg));
        }
        if file.is_some() {
            return Err("Multiple input files provided".to_string());
        }
        file = Some(arg.clone());
    }
    file.ok_or_else(|| "No file specified.".to_string())
}

fn parse_get_args(args: &[String]) -> Result<GetArgs, String> {
    let mut import_path = None;
    let mut git_url = None;
    let mut git_ref = None;
    let mut entry = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--import" => {
                if i + 1 >= args.len() {
                    return Err("--import requires a value".to_string());
                }
                import_path = Some(args[i + 1].clone());
                i += 2;
            }
            "--git" => {
                if i + 1 >= args.len() {
                    return Err("--git requires a value".to_string());
                }
                git_url = Some(args[i + 1].clone());
                i += 2;
            }
            "--ref" => {
                if i + 1 >= args.len() {
                    return Err("--ref requires a value".to_string());
                }
                git_ref = Some(args[i + 1].clone());
                i += 2;
            }
            "--entry" => {
                if i + 1 >= args.len() {
                    return Err("--entry requires a value".to_string());
                }
                entry = Some(args[i + 1].clone());
                i += 2;
            }
            flag if flag.starts_with("--") => {
                return Err(format!("Unknown flag `{}`", flag));
            }
            value => {
                return Err(format!("Unexpected positional argument `{}`", value));
            }
        }
    }

    Ok(GetArgs {
        import_path: import_path.ok_or_else(|| "--import is required".to_string())?,
        git_url: git_url.ok_or_else(|| "--git is required".to_string())?,
        git_ref: git_ref.ok_or_else(|| "--ref is required".to_string())?,
        entry: entry.ok_or_else(|| "--entry is required".to_string())?,
    })
}

fn format_file(file_path: &str, check_mode: bool) {
    let source = match fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file {}: {}", file_path, e);
            std::process::exit(1);
        }
    };

    let formatted = match formatter::format_source(&source) {
        Ok(f) => f,
        Err(errors) => {
            eprintln!("Formatting errors in {}:", file_path);
            for err in &errors {
                eprintln!("  {}", err);
            }
            std::process::exit(1);
        }
    };

    if check_mode {
        if source == formatted {
            println!("{} is already formatted.", file_path);
        } else {
            println!("{} needs formatting.", file_path);
            std::process::exit(1);
        }
    } else if source == formatted {
        println!("{} is already formatted.", file_path);
    } else {
        if let Err(e) = fs::write(file_path, &formatted) {
            eprintln!("Error writing file {}: {}", file_path, e);
            std::process::exit(1);
        }
        println!("Formatted {}.", file_path);
    }
}

fn compile_file(file_path: &str, target: &str, go_target: &GoTargetOptions) -> Option<String> {
    let loaded = load_and_typecheck(file_path)?;
    let LoadedProgram {
        program,
        uses_local_modules,
    } = loaded;

    let path = Path::new(file_path);
    let file_stem = path.file_stem()?.to_str()?;
    let output_name = output_file_name(file_stem, target, go_target);
    let output_path = current_dir_path().join(output_name);

    match target {
        "c" => {
            if uses_local_modules {
                eprintln!(
                    "C backend does not support local package/module system. Use `--target go`."
                );
                None
            } else {
                compile_with_c_backend(&program, file_stem, &output_path)
            }
        }
        "go" => compile_with_go_backend(&program, &output_path, go_target),
        "llvm" => compile_with_llvm_backend(&program, uses_local_modules, file_stem, &output_path),
        _ => {
            eprintln!("Unknown target '{}'. Use 'c', 'go', or 'llvm'.", target);
            None
        }
    }
}

fn load_and_typecheck(file_path: &str) -> Option<LoadedProgram> {
    println!("-> Loading modules {}", file_path);
    let loaded = match load_entry_program(Path::new(file_path)) {
        Ok(p) => p,
        Err(errors) => {
            eprintln!("Module Errors:");
            for err in errors {
                eprintln!("  {}", err);
            }
            return None;
        }
    };

    println!("-> Type Checking {}", file_path);
    let mut type_checker = TypeChecker::new();
    let mut env = Environment::new();
    type_checker.check_program(&loaded.program, &mut env);
    if !type_checker.errors.is_empty() {
        eprintln!("Type Errors:");
        for err in &type_checker.errors {
            eprintln!("  {}", err);
        }
        return None;
    }

    Some(loaded)
}

fn run_test_file(file_path: &str) -> bool {
    let loaded = match load_and_typecheck(file_path) {
        Some(v) => v,
        None => return false,
    };
    let test_names: Vec<String> = loaded
        .program
        .statements
        .iter()
        .filter_map(|stmt| match stmt {
            Statement::TestDecl { name, .. } => Some(name.clone()),
            _ => None,
        })
        .collect();

    if test_names.is_empty() {
        println!("No `set_sae` tests found.");
        return true;
    }

    let test_program = Program {
        statements: loaded
            .program
            .statements
            .iter()
            .filter(|stmt| !matches!(stmt, Statement::FunctionDecl { name, .. } if name == "main"))
            .cloned()
            .collect(),
    };

    let mut codegen = GoCodeGenerator::new();
    let go_code = codegen.generate(&test_program);
    let runner_code = generate_test_runner_source(&test_names);

    let workspace = match create_go_workspace(
        "test",
        &[("program.go", &go_code), ("runner.go", &runner_code)],
    ) {
        Ok(dir) => dir,
        Err(err) => {
            eprintln!("Failed to prepare Go test workspace: {}", err);
            return false;
        }
    };

    println!("-> Running {} tests", test_names.len());
    let output = Command::new("go")
        .current_dir(&workspace)
        .arg("run")
        .arg(".")
        .output();

    let _ = fs::remove_dir_all(&workspace);

    match output {
        Ok(out) => {
            print!("{}", String::from_utf8_lossy(&out.stdout));
            eprint!("{}", String::from_utf8_lossy(&out.stderr));
            out.status.success()
        }
        Err(e) => {
            eprintln!("Failed to invoke `go`: {}", e);
            eprintln!("Please ensure Go is installed and available in your PATH.");
            false
        }
    }
}

fn compile_with_c_backend(
    program: &Program,
    file_stem: &str,
    output_path: &Path,
) -> Option<String> {
    if program_uses_phase4(program) {
        eprintln!(
            "C backend does not support Phase 4 features (`set_sae`, `baung`, context middleware, database runtime). Use `--target go`."
        );
        return None;
    }

    if program_uses_phase3(program) {
        eprintln!(
            "C backend does not support Phase 3 features (`kyoe`, `naut_sone`, `laung`, server/socket runtime). Use `--target go`."
        );
        return None;
    }

    for stmt in &program.statements {
        if let Statement::Import { module, .. } = stmt {
            if let Some(module_info) = resolve_stdlib_module(module) {
                eprintln!(
                    "C backend does not support stdlib module `{}` (import `{}`). Use `--target go`.",
                    module_info.mlang_name, module
                );
                return None;
            }
        }
    }

    println!("-> Generating C Code");
    let mut codegen = CodeGenerator::new();
    let c_code = codegen.generate(program);
    let c_file_name = format!("{}.c", file_stem);

    if let Err(e) = fs::write(&c_file_name, &c_code) {
        eprintln!("Failed to write intermediate C file: {}", e);
        return None;
    }

    println!(
        "-> Compiling Native Executable ({}) using gcc",
        output_path.display()
    );
    let output = Command::new("gcc")
        .arg(&c_file_name)
        .arg("-o")
        .arg(output_path)
        .output();

    match output {
        Ok(out) => {
            if !out.status.success() {
                eprintln!("C Compiler Error:");
                eprintln!("{}", String::from_utf8_lossy(&out.stderr));
                return None;
            }
        }
        Err(e) => {
            eprintln!("Failed to invoke `gcc`: {}", e);
            eprintln!(
                "Please ensure `gcc` (MinGW-w64 on Windows) is installed and available in your PATH."
            );
            return None;
        }
    }

    println!(
        "-> Compilation Successful! Executable saved as {}",
        output_path.display()
    );
    Some(output_path.to_string_lossy().to_string())
}

fn compile_with_go_backend(
    program: &Program,
    output_path: &Path,
    go_target: &GoTargetOptions,
) -> Option<String> {
    println!("-> Generating Go Code");
    let mut codegen = GoCodeGenerator::new();
    let go_code = codegen.generate(program);

    let workspace = match create_go_workspace("build", &[("main.go", &go_code)]) {
        Ok(dir) => dir,
        Err(err) => {
            eprintln!("Failed to prepare Go build workspace: {}", err);
            return None;
        }
    };

    println!(
        "-> Compiling Native Executable ({}) using go build",
        output_path.display()
    );
    let mut command = Command::new("go");
    command
        .current_dir(&workspace)
        .arg("build")
        .arg("-o")
        .arg(output_path)
        .arg(".");
    if let Some(goos) = &go_target.goos {
        command.env("GOOS", goos);
    }
    if let Some(goarch) = &go_target.goarch {
        command.env("GOARCH", goarch);
    }

    let output = command.output();
    let _ = fs::remove_dir_all(&workspace);

    match output {
        Ok(out) => {
            if !out.status.success() {
                eprintln!("Go Compiler Error:");
                eprintln!("{}", String::from_utf8_lossy(&out.stderr));
                return None;
            }
        }
        Err(e) => {
            eprintln!("Failed to invoke `go`: {}", e);
            eprintln!("Please ensure Go is installed and available in your PATH.");
            eprintln!("Download from: https://go.dev/dl/");
            return None;
        }
    }

    println!(
        "-> Compilation Successful! Executable saved as {}",
        output_path.display()
    );
    Some(output_path.to_string_lossy().to_string())
}

fn compile_with_llvm_backend(
    program: &Program,
    uses_local_modules: bool,
    file_stem: &str,
    output_path: &Path,
) -> Option<String> {
    if let Some(message) = llvm_unsupported_feature_message(program, uses_local_modules) {
        eprintln!("{}", message);
        return None;
    }

    println!("-> Generating LLVM IR");

    let ir = match generate_llvm_ir(program, file_stem) {
        Ok(ir) => ir,
        Err(e) => {
            eprintln!("LLVM Code Generation Error: {}", e);
            return None;
        }
    };

    // Save LLVM IR to file for inspection
    let ll_file_name = format!("{}.ll", file_stem);
    println!("-> Saving LLVM IR to {}", ll_file_name);
    if let Err(e) = fs::write(&ll_file_name, ir) {
        eprintln!("Failed to write LLVM IR file: {}", e);
        return None;
    }

    println!("-> Compiling LLVM IR to native object file");

    let llc_cmd = find_command(&["llc"]);
    let clang_cmd = find_command(&["clang"]);
    let cc_cmd = find_command(&["gcc", "clang", "cc"]);

    if llc_cmd.is_none() && clang_cmd.is_none() {
        eprintln!("Missing required tool `llc` or `clang` for LLVM backend native compilation.");
        eprintln!("Generated IR is available at: {}", ll_file_name);
        eprintln!("Install LLVM tools and ensure `llc` or `clang` is in PATH.");
        eprintln!("Windows example: install LLVM and add <LLVM>/bin to PATH.");
        return None;
    }
    if cc_cmd.is_none() {
        eprintln!(
            "Missing required C compiler (`gcc`, `clang`, or `cc`) for LLVM backend linking."
        );
        eprintln!("Generated IR is available at: {}", ll_file_name);
        return None;
    }

    let cc_cmd = cc_cmd.expect("checked above");

    let obj_file_name = format!("{}.o", file_stem);
    if let Some(llc_cmd) = llc_cmd {
        let llc_output = Command::new(&llc_cmd)
            .arg(&ll_file_name)
            .arg("-filetype=obj")
            .arg("-o")
            .arg(&obj_file_name)
            .output();

        match llc_output {
            Ok(out) => {
                if !out.status.success() {
                    eprintln!("LLVM llc Error:");
                    eprintln!("{}", String::from_utf8_lossy(&out.stderr));
                    return None;
                }
            }
            Err(e) => {
                eprintln!("Failed to invoke `{}`: {}", llc_cmd, e);
                return None;
            }
        }
    } else if let Some(clang_cmd) = clang_cmd {
        println!(
            "-> `llc` not found; using `{}` to compile LLVM IR",
            clang_cmd
        );
        let mut command = Command::new(&clang_cmd);
        command
            .arg("-c")
            .arg("-x")
            .arg("ir")
            .arg(&ll_file_name)
            .arg("-o")
            .arg(&obj_file_name);
        #[cfg(windows)]
        command.arg("--target=x86_64-w64-windows-gnu");
        let clang_output = command.output();

        match clang_output {
            Ok(out) => {
                if !out.status.success() {
                    eprintln!("LLVM clang IR Compile Error:");
                    eprintln!("{}", String::from_utf8_lossy(&out.stderr));
                    return None;
                }
            }
            Err(e) => {
                eprintln!("Failed to invoke `{}` for IR compile: {}", clang_cmd, e);
                return None;
            }
        }
    }

    let runtime_c_path = current_dir_path().join("runtime_llvm.c");
    if !runtime_c_path.exists() {
        eprintln!(
            "Missing runtime file required for LLVM backend: {}",
            runtime_c_path.display()
        );
        return None;
    }

    let runtime_obj_name = format!("{}.runtime.o", file_stem);
    let mut runtime_compile_command = Command::new(&cc_cmd);
    runtime_compile_command
        .arg("-c")
        .arg(&runtime_c_path)
        .arg("-o")
        .arg(&runtime_obj_name);
    let runtime_compile = runtime_compile_command.output();

    match runtime_compile {
        Ok(out) => {
            if !out.status.success() {
                eprintln!("Runtime Compile Error:");
                eprintln!("{}", String::from_utf8_lossy(&out.stderr));
                return None;
            }
        }
        Err(e) => {
            eprintln!("Failed to invoke `{}` for runtime compile: {}", cc_cmd, e);
            return None;
        }
    }

    println!(
        "-> Linking Native Executable ({}) using {}",
        output_path.display(),
        cc_cmd
    );
    let mut link_command = Command::new(&cc_cmd);
    link_command
        .arg(&obj_file_name)
        .arg(&runtime_obj_name)
        .arg("-o")
        .arg(output_path);
    let link_output = link_command.output();

    match link_output {
        Ok(out) => {
            if !out.status.success() {
                eprintln!("Linker Error:");
                eprintln!("{}", String::from_utf8_lossy(&out.stderr));
                return None;
            }
        }
        Err(e) => {
            eprintln!("Failed to invoke linker `{}`: {}", cc_cmd, e);
            return None;
        }
    }

    println!(
        "-> Compilation Successful! Executable saved as {}",
        output_path.display()
    );
    Some(output_path.to_string_lossy().to_string())
}

fn llvm_unsupported_feature_message(program: &Program, uses_local_modules: bool) -> Option<String> {
    if uses_local_modules {
        return Some(
            "LLVM backend MVP does not support local package/module imports yet. Use `--target go` for Phase 2 module-system builds."
                .to_string(),
        );
    }

    if program_uses_phase4(program) {
        return Some(
            "LLVM backend MVP does not support Phase 4 features (`set_sae`, `baung`, context middleware, database runtime) yet. Use `--target go` for production/server features."
                .to_string(),
        );
    }

    if program_uses_phase3(program) {
        return Some(
            "LLVM backend MVP does not support Phase 3 features (`kyoe`, `naut_sone`, `laung`, server/socket runtime) yet. Use `--target go` for concurrency and networking features."
                .to_string(),
        );
    }

    for stmt in &program.statements {
        match stmt {
            Statement::Import { module, .. } => {
                if let Some(module_info) = resolve_stdlib_module(module) {
                    return Some(format!(
                        "LLVM backend does not support Go-backed stdlib module `{}` (import `{}`) yet. Use `--target go` for stdlib/server builds.",
                        module_info.mlang_name, module
                    ));
                }
            }
            Statement::PackageDecl { .. } | Statement::Export { .. } => {
                return Some(
                    "LLVM backend MVP does not support Phase 2 package/export declarations yet. Use `--target go` for module-system builds."
                        .to_string(),
                );
            }
            _ => {}
        }
    }

    None
}

fn find_command(candidates: &[&str]) -> Option<String> {
    for cmd in candidates {
        let env_key = format!("MLANG_{}_PATH", cmd.to_ascii_uppercase().replace('-', "_"));
        if let Ok(custom) = env::var(&env_key) {
            if !custom.trim().is_empty() && Command::new(&custom).arg("--version").output().is_ok()
            {
                return Some(custom);
            }
        }
    }

    #[cfg(windows)]
    {
        for cmd in candidates {
            if *cmd == "llc" {
                let common = [
                    r"C:\Program Files\LLVM\bin\llc.exe",
                    r"C:\Program Files (x86)\LLVM\bin\llc.exe",
                ];
                for path in common {
                    if Path::new(path).exists()
                        && Command::new(path).arg("--version").output().is_ok()
                    {
                        return Some(path.to_string());
                    }
                }
            }
        }
    }

    for cmd in candidates {
        if Command::new(cmd).arg("--version").output().is_ok() {
            return Some((*cmd).to_string());
        }
    }
    None
}

fn create_go_workspace(tag: &str, files: &[(&str, &str)]) -> Result<PathBuf, String> {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let dir = env::temp_dir().join(format!("mlang_{}_{}_{}", tag, std::process::id(), nanos));

    fs::create_dir_all(&dir).map_err(|e| format!("create_dir_all {}: {}", dir.display(), e))?;
    fs::write(dir.join("go.mod"), "module mlangtmp\n\ngo 1.22\n")
        .map_err(|e| format!("write go.mod: {}", e))?;

    let needs_tidy = files
        .iter()
        .any(|(_, src)| src.contains("\"github.com/lib/pq\""));

    for (name, source) in files {
        fs::write(dir.join(name), source).map_err(|e| format!("write {}: {}", name, e))?;
    }

    if needs_tidy {
        let output = Command::new("go")
            .current_dir(&dir)
            .arg("mod")
            .arg("tidy")
            .output()
            .map_err(|e| format!("failed to invoke `go mod tidy`: {}", e))?;
        if !output.status.success() {
            return Err(format!(
                "go mod tidy failed:\n{}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    }

    Ok(dir)
}

fn output_file_name(file_stem: &str, target: &str, go_target: &GoTargetOptions) -> String {
    match target {
        "go" => {
            let (goos, goarch) = resolved_go_target(go_target);
            let (host_goos, host_goarch) = host_go_target();
            let is_cross = goos != host_goos || goarch != host_goarch;
            if is_cross {
                format!(
                    "{}_{}_{}{}",
                    file_stem,
                    goos,
                    goarch,
                    go_binary_suffix(&goos)
                )
            } else {
                format!("{}{}", file_stem, go_binary_suffix(&goos))
            }
        }
        "c" => format!("{}{}", file_stem, native_binary_suffix()),
        _ => format!("{}{}", file_stem, native_binary_suffix()),
    }
}

fn native_binary_suffix() -> &'static str {
    if cfg!(windows) {
        ".exe"
    } else {
        ""
    }
}

fn go_binary_suffix(goos: &str) -> &'static str {
    if goos == "windows" {
        ".exe"
    } else {
        ""
    }
}

fn host_go_target() -> (String, String) {
    let goos = match env::consts::OS {
        "macos" => "darwin",
        other => other,
    };
    let goarch = match env::consts::ARCH {
        "x86_64" => "amd64",
        "aarch64" => "arm64",
        "x86" => "386",
        other => other,
    };
    (goos.to_string(), goarch.to_string())
}

fn resolved_go_target(opts: &GoTargetOptions) -> (String, String) {
    let (host_goos, host_goarch) = host_go_target();
    (
        opts.goos.clone().unwrap_or(host_goos),
        opts.goarch.clone().unwrap_or(host_goarch),
    )
}

fn is_cross_target(opts: &GoTargetOptions) -> bool {
    let (goos, goarch) = resolved_go_target(opts);
    let (host_goos, host_goarch) = host_go_target();
    goos != host_goos || goarch != host_goarch
}

fn current_dir_path() -> PathBuf {
    env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

fn generate_test_runner_source(test_names: &[String]) -> String {
    let mut output = String::new();
    output.push_str("package main\n\n");
    output.push_str("import (\n");
    output.push_str("\t\"fmt\"\n");
    output.push_str("\t\"os\"\n");
    output.push_str(")\n\n");

    output.push_str("type mlangTestCase struct {\n");
    output.push_str("\tname string\n");
    output.push_str("\trun func() error\n");
    output.push_str("}\n\n");

    output.push_str("func main() {\n");
    output.push_str("\ttests := []mlangTestCase{\n");
    for name in test_names {
        output.push_str(&format!(
            "\t\t{{name: {}, run: {}}},\n",
            serde_json::to_string(name).unwrap_or_else(|_| "\"<invalid>\"".to_string()),
            go_test_function_name(name)
        ));
    }
    output.push_str("\t}\n\n");
    output.push_str("\tfailed := 0\n");
    output.push_str("\tfor _, tc := range tests {\n");
    output.push_str("\t\tif err := tc.run(); err != nil {\n");
    output.push_str("\t\t\tfmt.Printf(\"[FAIL] %s: %v\\n\", tc.name, err)\n");
    output.push_str("\t\t\tfailed++\n");
    output.push_str("\t\t} else {\n");
    output.push_str("\t\t\tfmt.Printf(\"[PASS] %s\\n\", tc.name)\n");
    output.push_str("\t\t}\n");
    output.push_str("\t}\n\n");
    output.push_str("\tif failed > 0 {\n");
    output.push_str("\t\tfmt.Printf(\"Failed %d/%d tests\\n\", failed, len(tests))\n");
    output.push_str("\t\tos.Exit(1)\n");
    output.push_str("\t}\n");
    output.push_str("\tfmt.Printf(\"Passed %d/%d tests\\n\", len(tests), len(tests))\n");
    output.push_str("}\n");
    output
}

fn go_test_function_name(name: &str) -> String {
    let go_ident = clean_go_identifier(name);
    go_exported_name(&format!("mlang_test_{}", go_ident))
}

fn clean_go_identifier(name: &str) -> String {
    let go_keywords = [
        "break",
        "case",
        "chan",
        "const",
        "continue",
        "default",
        "defer",
        "else",
        "fallthrough",
        "for",
        "func",
        "go",
        "goto",
        "if",
        "import",
        "interface",
        "map",
        "package",
        "range",
        "return",
        "select",
        "struct",
        "switch",
        "type",
        "var",
    ];
    if go_keywords.contains(&name) {
        format!("ml_{}", name)
    } else {
        name.to_string()
    }
}

fn go_exported_name(value: &str) -> String {
    value
        .split('_')
        .filter(|part| !part.is_empty())
        .map(capitalize_first)
        .collect::<Vec<String>>()
        .join("")
}

fn capitalize_first(value: &str) -> String {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

fn install_dependency(args: &GetArgs) -> Result<(), String> {
    let cwd = current_dir_path();
    let project_root = find_project_root_for_lock(&cwd);
    let entry = normalize_entry_path(&args.entry)?;
    let commit = resolve_git_commit(&args.git_url, &args.git_ref)?;

    let cache_rel = PathBuf::from(".mlang")
        .join("deps")
        .join(commit.to_lowercase());
    let cache_abs = project_root.join(&cache_rel);
    if !cache_abs.exists() {
        if let Some(parent) = cache_abs.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create deps dir {}: {}", parent.display(), e))?;
        }

        let clone_output = Command::new("git")
            .arg("clone")
            .arg("--filter=blob:none")
            .arg("--no-checkout")
            .arg(&args.git_url)
            .arg(&cache_abs)
            .output()
            .map_err(|e| format!("Failed to invoke `git clone`: {}", e))?;
        if !clone_output.status.success() {
            return Err(format!(
                "git clone failed:\n{}",
                String::from_utf8_lossy(&clone_output.stderr)
            ));
        }

        let checkout_output = Command::new("git")
            .arg("-C")
            .arg(&cache_abs)
            .arg("checkout")
            .arg("--detach")
            .arg(&commit)
            .output()
            .map_err(|e| format!("Failed to invoke `git checkout`: {}", e))?;
        if !checkout_output.status.success() {
            return Err(format!(
                "git checkout {} failed:\n{}",
                commit,
                String::from_utf8_lossy(&checkout_output.stderr)
            ));
        }
    }

    let entry_file = cache_abs.join(&entry);
    if !entry_file.exists() {
        return Err(format!(
            "Entry file `{}` not found in dependency cache {}",
            entry,
            cache_abs.display()
        ));
    }

    let lock_path = project_root.join("mlang.lock");
    let mut lock = load_lock_file(&lock_path)?;
    if lock.version == 0 {
        lock.version = LOCK_VERSION;
    }
    lock.version = LOCK_VERSION;

    let dep = LockDependency {
        import: args.import_path.clone(),
        git: args.git_url.clone(),
        git_ref: args.git_ref.clone(),
        commit: commit.clone(),
        entry: entry.clone(),
        cache_dir: cache_rel.to_string_lossy().replace('\\', "/"),
    };

    if let Some(existing) = lock.deps.iter_mut().find(|d| d.import == dep.import) {
        *existing = dep;
    } else {
        lock.deps.push(dep);
    }
    lock.deps.sort_by(|a, b| a.import.cmp(&b.import));

    let serialized =
        serde_json::to_string_pretty(&lock).map_err(|e| format!("serialize lockfile: {}", e))?;
    fs::write(&lock_path, format!("{}\n", serialized))
        .map_err(|e| format!("write {}: {}", lock_path.display(), e))?;

    println!("-> Installed `{}` at commit {}", args.import_path, commit);
    println!("-> Updated lockfile {}", lock_path.display());
    Ok(())
}

fn load_lock_file(lock_path: &Path) -> Result<LockFile, String> {
    if !lock_path.exists() {
        return Ok(LockFile {
            version: LOCK_VERSION,
            deps: Vec::new(),
        });
    }
    let content = fs::read_to_string(lock_path)
        .map_err(|e| format!("Failed to read {}: {}", lock_path.display(), e))?;
    serde_json::from_str(&content)
        .map_err(|e| format!("Invalid lockfile {}: {}", lock_path.display(), e))
}

fn normalize_entry_path(entry: &str) -> Result<String, String> {
    let normalized = entry.trim().replace('\\', "/");
    if normalized.is_empty() {
        return Err("`--entry` cannot be empty".to_string());
    }
    if Path::new(&normalized).is_absolute() {
        return Err("`--entry` must be a relative path".to_string());
    }
    let path = Path::new(&normalized);
    if path
        .components()
        .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        return Err("`--entry` cannot contain `..` path traversal".to_string());
    }
    Ok(normalized)
}

fn find_project_root_for_lock(start_dir: &Path) -> PathBuf {
    let mut dir = start_dir.to_path_buf();
    loop {
        if dir.join("mlang.lock").exists() {
            return dir;
        }
        if !dir.pop() {
            return start_dir.to_path_buf();
        }
    }
}

fn resolve_git_commit(git_url: &str, git_ref: &str) -> Result<String, String> {
    for candidate in [
        git_ref.to_string(),
        format!("refs/heads/{}", git_ref),
        format!("refs/tags/{}", git_ref),
        format!("refs/tags/{}^{{}}", git_ref),
    ] {
        let output = Command::new("git")
            .arg("ls-remote")
            .arg(git_url)
            .arg(&candidate)
            .output()
            .map_err(|e| format!("Failed to invoke `git ls-remote`: {}", e))?;
        if !output.status.success() {
            return Err(format!(
                "git ls-remote failed:\n{}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        if let Some(commit) = parse_ls_remote_commit(&output.stdout) {
            return Ok(commit);
        }
    }

    if is_hex_commit(git_ref) {
        return Ok(git_ref.to_lowercase());
    }

    Err(format!(
        "Unable to resolve ref `{}` for repository `{}`",
        git_ref, git_url
    ))
}

fn parse_ls_remote_commit(stdout: &[u8]) -> Option<String> {
    let text = String::from_utf8_lossy(stdout);
    let line = text.lines().next()?;
    let commit = line.split_whitespace().next()?;
    if is_hex_commit(commit) {
        Some(commit.to_lowercase())
    } else {
        None
    }
}

fn is_hex_commit(value: &str) -> bool {
    let len = value.len();
    len >= 7 && len <= 40 && value.chars().all(|ch| ch.is_ascii_hexdigit())
}

fn program_uses_phase3(program: &Program) -> bool {
    program.statements.iter().any(statement_uses_phase3)
}

fn block_uses_phase3(block: &BlockStatement) -> bool {
    block.statements.iter().any(statement_uses_phase3)
}

fn statement_uses_phase3(stmt: &Statement) -> bool {
    match stmt {
        Statement::Go { .. } | Statement::Defer { .. } => true,
        Statement::Let { ty, value, .. } => type_uses_phase3(ty) || expression_uses_phase3(value),
        Statement::LetDestructured { names, value } => {
            names.iter().any(|(_, ty, _)| type_uses_phase3(ty)) || expression_uses_phase3(value)
        }
        Statement::Assign { value, .. } => expression_uses_phase3(value),
        Statement::FieldAssign { value, .. } => expression_uses_phase3(value),
        Statement::IndexAssign {
            object,
            index,
            value,
            ..
        } => {
            expression_uses_phase3(object)
                || expression_uses_phase3(index)
                || expression_uses_phase3(value)
        }
        Statement::If {
            condition,
            consequence,
            alternative,
        } => {
            expression_uses_phase3(condition)
                || block_uses_phase3(consequence)
                || alternative
                    .as_ref()
                    .map(|alt| match alt {
                        mlang::ast::IfAlternative::Else(block) => block_uses_phase3(block),
                        mlang::ast::IfAlternative::ElseIf(stmt) => statement_uses_phase3(stmt),
                    })
                    .unwrap_or(false)
        }
        Statement::While { condition, body } => {
            expression_uses_phase3(condition) || block_uses_phase3(body)
        }
        Statement::ForIn {
            collection, body, ..
        } => expression_uses_phase3(collection) || block_uses_phase3(body),
        Statement::ForClassic {
            init,
            condition,
            post,
            body,
        } => {
            init.as_ref()
                .map(|s| statement_uses_phase3(s))
                .unwrap_or(false)
                || condition
                    .as_ref()
                    .map(expression_uses_phase3)
                    .unwrap_or(false)
                || post
                    .as_ref()
                    .map(|s| statement_uses_phase3(s))
                    .unwrap_or(false)
                || block_uses_phase3(body)
        }
        Statement::FunctionDecl {
            parameters,
            return_type,
            body,
            ..
        } => {
            parameters.iter().any(|(_, ty, _)| type_uses_phase3(ty))
                || type_uses_phase3(return_type)
                || block_uses_phase3(body)
        }
        Statement::TestDecl { body, .. } => block_uses_phase3(body),
        Statement::Return { value } => expression_uses_phase3(value),
        Statement::Print { value } => expression_uses_phase3(value),
        Statement::Import { module, .. } => module.trim_matches('"') == "kainn",
        Statement::ExpressionStatement(expr) => expression_uses_phase3(expr),
        Statement::StructDecl { fields, .. } => fields.iter().any(|(_, ty)| type_uses_phase3(ty)),
        Statement::MethodDecl {
            parameters,
            return_type,
            body,
            ..
        } => {
            parameters.iter().any(|(_, ty, _)| type_uses_phase3(ty))
                || type_uses_phase3(return_type)
                || block_uses_phase3(body)
        }
        Statement::InterfaceDecl { methods, .. } => methods.iter().any(|(_, params, ret)| {
            params.iter().any(|(_, ty)| type_uses_phase3(ty)) || type_uses_phase3(ret)
        }),
        Statement::Export { statement, .. } => statement_uses_phase3(statement),
        Statement::PackageDecl { .. } | Statement::Break | Statement::Continue => false,
    }
}

fn expression_uses_phase3(expr: &Expression) -> bool {
    match expr {
        Expression::ChannelMake { .. } => true,
        Expression::FunctionCall { arguments, .. } => arguments.iter().any(expression_uses_phase3),
        Expression::MethodCall {
            object,
            method,
            arguments,
        } => {
            (matches!(object.as_ref(), Expression::Identifier(name) if name == "http")
                && matches!(method.as_str(), "handle" | "listen"))
                || matches!(method.as_str(), "send" | "recv" | "close")
                || expression_uses_phase3(object)
                || arguments.iter().any(expression_uses_phase3)
        }
        Expression::Binary { left, right, .. } => {
            expression_uses_phase3(left) || expression_uses_phase3(right)
        }
        Expression::ArrayLiteral { elements } => elements.iter().any(expression_uses_phase3),
        Expression::HashLiteral { pairs } => pairs
            .iter()
            .any(|(k, v)| expression_uses_phase3(k) || expression_uses_phase3(v)),
        Expression::IndexExpression { left, index } => {
            expression_uses_phase3(left) || expression_uses_phase3(index)
        }
        Expression::SliceExpression { left, low, high } => {
            expression_uses_phase3(left)
                || low
                    .as_ref()
                    .map(|e| expression_uses_phase3(e))
                    .unwrap_or(false)
                || high
                    .as_ref()
                    .map(|e| expression_uses_phase3(e))
                    .unwrap_or(false)
        }
        Expression::ReadInput { prompt } => expression_uses_phase3(prompt),
        Expression::TypeConversion { argument, .. } => expression_uses_phase3(argument),
        Expression::FieldAccess { object, .. } => expression_uses_phase3(object),
        Expression::StructLiteral { fields, .. } => fields
            .iter()
            .any(|(_, value)| expression_uses_phase3(value)),
        Expression::ClosureLiteral {
            parameters,
            return_type,
            body,
        } => {
            parameters.iter().any(|(_, ty, _)| type_uses_phase3(ty))
                || type_uses_phase3(return_type)
                || block_uses_phase3(body)
        }
        Expression::ErrorCreate { message } => expression_uses_phase3(message),
        Expression::TupleLiteral { elements } => elements.iter().any(expression_uses_phase3),
        Expression::BaungCreate { timeout_ms } => expression_uses_phase3(timeout_ms),
        Expression::IntegerLiteral(_)
        | Expression::FloatLiteral(_)
        | Expression::StringLiteral(_)
        | Expression::BooleanLiteral(_)
        | Expression::NilLiteral
        | Expression::Identifier(_) => false,
    }
}

fn type_uses_phase3(ty: &Type) -> bool {
    match ty {
        Type::Channel(_) => true,
        Type::Array(inner) => type_uses_phase3(inner),
        Type::Map(key, value) => type_uses_phase3(key) || type_uses_phase3(value),
        Type::Tuple(types) => types.iter().any(type_uses_phase3),
        Type::Function {
            params,
            return_type,
        } => params.iter().any(type_uses_phase3) || type_uses_phase3(return_type),
        Type::Kain
        | Type::Sar
        | Type::Sit
        | Type::DaTha
        | Type::Baung
        | Type::Nil
        | Type::Error
        | Type::Struct(_)
        | Type::Interface(_) => false,
    }
}

fn program_uses_phase4(program: &Program) -> bool {
    program.statements.iter().any(statement_uses_phase4)
}

fn block_uses_phase4(block: &BlockStatement) -> bool {
    block.statements.iter().any(statement_uses_phase4)
}

fn statement_uses_phase4(stmt: &Statement) -> bool {
    match stmt {
        Statement::TestDecl { .. } => true,
        Statement::Let { ty, value, .. } => type_uses_phase4(ty) || expression_uses_phase4(value),
        Statement::LetDestructured { names, value } => {
            names.iter().any(|(_, ty, _)| type_uses_phase4(ty)) || expression_uses_phase4(value)
        }
        Statement::Assign { value, .. } => expression_uses_phase4(value),
        Statement::FieldAssign { value, .. } => expression_uses_phase4(value),
        Statement::IndexAssign {
            object,
            index,
            value,
            ..
        } => {
            expression_uses_phase4(object)
                || expression_uses_phase4(index)
                || expression_uses_phase4(value)
        }
        Statement::If {
            condition,
            consequence,
            alternative,
        } => {
            expression_uses_phase4(condition)
                || block_uses_phase4(consequence)
                || alternative
                    .as_ref()
                    .map(|alt| match alt {
                        mlang::ast::IfAlternative::Else(block) => block_uses_phase4(block),
                        mlang::ast::IfAlternative::ElseIf(stmt) => statement_uses_phase4(stmt),
                    })
                    .unwrap_or(false)
        }
        Statement::While { condition, body } => {
            expression_uses_phase4(condition) || block_uses_phase4(body)
        }
        Statement::ForIn {
            collection, body, ..
        } => expression_uses_phase4(collection) || block_uses_phase4(body),
        Statement::ForClassic {
            init,
            condition,
            post,
            body,
        } => {
            init.as_ref()
                .map(|s| statement_uses_phase4(s))
                .unwrap_or(false)
                || condition
                    .as_ref()
                    .map(expression_uses_phase4)
                    .unwrap_or(false)
                || post
                    .as_ref()
                    .map(|s| statement_uses_phase4(s))
                    .unwrap_or(false)
                || block_uses_phase4(body)
        }
        Statement::FunctionDecl {
            parameters,
            return_type,
            body,
            ..
        } => {
            parameters.iter().any(|(_, ty, _)| type_uses_phase4(ty))
                || type_uses_phase4(return_type)
                || block_uses_phase4(body)
        }
        Statement::MethodDecl {
            parameters,
            return_type,
            body,
            ..
        } => {
            parameters.iter().any(|(_, ty, _)| type_uses_phase4(ty))
                || type_uses_phase4(return_type)
                || block_uses_phase4(body)
        }
        Statement::Return { value } => expression_uses_phase4(value),
        Statement::Print { value } => expression_uses_phase4(value),
        Statement::Import { module, .. } => module.trim_matches('"') == "database",
        Statement::ExpressionStatement(expr) => expression_uses_phase4(expr),
        Statement::StructDecl { fields, .. } => fields.iter().any(|(_, ty)| type_uses_phase4(ty)),
        Statement::InterfaceDecl { methods, .. } => methods.iter().any(|(_, params, ret)| {
            params.iter().any(|(_, ty)| type_uses_phase4(ty)) || type_uses_phase4(ret)
        }),
        Statement::Export { statement, .. } => statement_uses_phase4(statement),
        Statement::Go { call } | Statement::Defer { call } => expression_uses_phase4(call),
        Statement::PackageDecl { .. } | Statement::Break | Statement::Continue => false,
    }
}

fn expression_uses_phase4(expr: &Expression) -> bool {
    match expr {
        Expression::BaungCreate { .. } => true,
        Expression::MethodCall {
            object,
            method,
            arguments,
        } => {
            (matches!(object.as_ref(), Expression::Identifier(name) if name == "http")
                && matches!(method.as_str(), "handle_ctx" | "handle_timeout"))
                || expression_uses_phase4(object)
                || arguments.iter().any(expression_uses_phase4)
        }
        Expression::FunctionCall { arguments, .. } => arguments.iter().any(expression_uses_phase4),
        Expression::Binary { left, right, .. } => {
            expression_uses_phase4(left) || expression_uses_phase4(right)
        }
        Expression::ArrayLiteral { elements } => elements.iter().any(expression_uses_phase4),
        Expression::HashLiteral { pairs } => pairs
            .iter()
            .any(|(k, v)| expression_uses_phase4(k) || expression_uses_phase4(v)),
        Expression::IndexExpression { left, index } => {
            expression_uses_phase4(left) || expression_uses_phase4(index)
        }
        Expression::SliceExpression { left, low, high } => {
            expression_uses_phase4(left)
                || low
                    .as_ref()
                    .map(|e| expression_uses_phase4(e))
                    .unwrap_or(false)
                || high
                    .as_ref()
                    .map(|e| expression_uses_phase4(e))
                    .unwrap_or(false)
        }
        Expression::ReadInput { prompt } => expression_uses_phase4(prompt),
        Expression::TypeConversion { argument, .. } => expression_uses_phase4(argument),
        Expression::FieldAccess { object, .. } => expression_uses_phase4(object),
        Expression::StructLiteral { fields, .. } => fields
            .iter()
            .any(|(_, value)| expression_uses_phase4(value)),
        Expression::ClosureLiteral {
            parameters,
            return_type,
            body,
        } => {
            parameters.iter().any(|(_, ty, _)| type_uses_phase4(ty))
                || type_uses_phase4(return_type)
                || block_uses_phase4(body)
        }
        Expression::ErrorCreate { message } => expression_uses_phase4(message),
        Expression::TupleLiteral { elements } => elements.iter().any(expression_uses_phase4),
        Expression::ChannelMake { capacity, .. } => capacity
            .as_ref()
            .map(|c| expression_uses_phase4(c))
            .unwrap_or(false),
        Expression::IntegerLiteral(_)
        | Expression::FloatLiteral(_)
        | Expression::StringLiteral(_)
        | Expression::BooleanLiteral(_)
        | Expression::NilLiteral
        | Expression::Identifier(_) => false,
    }
}

fn type_uses_phase4(ty: &Type) -> bool {
    match ty {
        Type::Baung => true,
        Type::Array(inner) | Type::Channel(inner) => type_uses_phase4(inner),
        Type::Map(key, value) => type_uses_phase4(key) || type_uses_phase4(value),
        Type::Tuple(types) => types.iter().any(type_uses_phase4),
        Type::Function {
            params,
            return_type,
        } => params.iter().any(type_uses_phase4) || type_uses_phase4(return_type),
        Type::Kain
        | Type::Sar
        | Type::Sit
        | Type::DaTha
        | Type::Nil
        | Type::Error
        | Type::Struct(_)
        | Type::Interface(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mlang::lexer::Lexer;
    use mlang::parser::Parser;

    fn parse_program(input: &str) -> Program {
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);
        let program = parser.parse_program().unwrap();
        assert!(
            parser.errors.is_empty(),
            "Parse errors: {:?}",
            parser.errors
        );
        program
    }

    #[test]
    fn test_parse_build_args_with_cross_flags() {
        let args = vec![
            "--target".to_string(),
            "go".to_string(),
            "--goos".to_string(),
            "linux".to_string(),
            "--goarch".to_string(),
            "arm64".to_string(),
            "app.ml".to_string(),
        ];
        let parsed = parse_build_args(&args).expect("parse build args");
        assert_eq!(parsed.target, "go");
        assert_eq!(parsed.file_path.as_deref(), Some("app.ml"));
        assert_eq!(parsed.go_target.goos.as_deref(), Some("linux"));
        assert_eq!(parsed.go_target.goarch.as_deref(), Some("arm64"));
        assert!(is_cross_target(&parsed.go_target));
    }

    #[test]
    fn test_parse_build_args_defaults_to_llvm() {
        let args = vec!["app.ml".to_string()];
        let parsed = parse_build_args(&args).expect("parse build args");
        assert_eq!(parsed.target, "llvm");
        assert_eq!(parsed.file_path.as_deref(), Some("app.ml"));
    }

    #[test]
    fn test_parse_build_args_rejects_go_flags_for_c() {
        let args = vec![
            "--target".to_string(),
            "c".to_string(),
            "--goos".to_string(),
            "linux".to_string(),
            "app.ml".to_string(),
        ];
        let err = parse_build_args(&args).unwrap_err();
        assert!(err.contains("--goos/--goarch"));
    }

    #[test]
    fn test_parse_build_args_rejects_go_flags_for_default_llvm() {
        let args = vec![
            "--goos".to_string(),
            "linux".to_string(),
            "app.ml".to_string(),
        ];
        let err = parse_build_args(&args).unwrap_err();
        assert!(err.contains("--goos/--goarch are only supported with `--target go`"));
    }

    #[test]
    fn test_llvm_backend_reports_go_only_stdlib_feature() {
        let program = parse_program(
            r#"
            yu "json";

            loke main() -> kain {
                pyan 0;
            }
            "#,
        );

        let err = llvm_unsupported_feature_message(&program, false).unwrap();
        assert!(err.contains("LLVM backend does not support Go-backed stdlib module `json`"));
        assert!(err.contains("Use `--target go`"));
    }

    #[test]
    fn test_llvm_backend_reports_phase3_feature() {
        let program = parse_program(
            r#"
            loke main() -> kain {
                laung<kain> ch = laung<kain>(1);
                kyoe worker(ch);
                pyan 0;
            }

            loke worker(laung<kain> ch) -> kain {
                ch.send(1);
                pyan 0;
            }
            "#,
        );

        let err = llvm_unsupported_feature_message(&program, false).unwrap();
        assert!(err.contains("LLVM backend MVP does not support Phase 3"));
        assert!(err.contains("Use `--target go`"));
    }

    #[test]
    fn test_program_uses_phase3_detection() {
        let phase3_program = parse_program(
            r#"
            loke main() -> kain {
                laung<kain> ch = laung<kain>();
                kyoe run(ch);
                pyan 0;
            }

            loke run(laung<kain> ch) -> kain {
                naut_sone ch.close();
                ch.send(1);
                pyan ch.recv();
            }
            "#,
        );
        assert!(program_uses_phase3(&phase3_program));

        let non_phase3_program = parse_program(
            r#"
            loke main() -> kain {
                su<kain> nums = [1, 2, 3];
                pya(ashay(nums));
                pyan 0;
            }
            "#,
        );
        assert!(!program_uses_phase3(&non_phase3_program));
    }

    #[test]
    fn test_program_uses_phase4_detection() {
        let phase4_program = parse_program(
            r#"
            set_sae timeout_guard {
                baung ctx = baung(5000);
                pyan ctx.close();
            }
            "#,
        );
        assert!(program_uses_phase4(&phase4_program));

        let non_phase4_program = parse_program(
            r#"
            loke main() -> kain {
                pyan 0;
            }
            "#,
        );
        assert!(!program_uses_phase4(&non_phase4_program));
    }

    #[test]
    fn test_c_backend_rejects_phase3_program() {
        let program = parse_program(
            r#"
            loke main() -> kain {
                laung<kain> ch = laung<kain>(1);
                kyoe worker(ch);
                pyan 0;
            }

            loke worker(laung<kain> ch) -> kain {
                naut_sone ch.close();
                ch.send(1);
                pyan 0;
            }
            "#,
        );

        let out = PathBuf::from("phase3_guard.exe");
        let compile = compile_with_c_backend(&program, "phase3_guard", &out);
        assert!(compile.is_none());
    }

    #[test]
    fn test_c_backend_rejects_phase4_program() {
        let program = parse_program(
            r#"
            set_sae smoke {
                baung ctx = baung(1000);
                pyan ctx.close();
            }
            "#,
        );

        let out = PathBuf::from("phase4_guard.exe");
        let compile = compile_with_c_backend(&program, "phase4_guard", &out);
        assert!(compile.is_none());
    }

    #[test]
    fn test_go_test_function_name_matches_codegen_shape() {
        assert_eq!(
            go_test_function_name("timeout_guard"),
            "MlangTestTimeoutGuard"
        );
    }
}
