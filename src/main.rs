use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

use mlang::lexer::Lexer;
use mlang::parser::Parser;
use mlang::typecheck::{Environment, TypeChecker};
use mlang::codegen::CodeGenerator;
use mlang::codegen_go::GoCodeGenerator;
use mlang::formatter;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        return;
    }

    let command = &args[1];

    match command.as_str() {
        "fmt" => {
            let (check_mode, file_path) = parse_fmt_args(&args[2..]);
            match file_path {
                Some(path) => format_file(&path, check_mode),
                None => {
                    eprintln!("Error: No file specified for fmt command.");
                    print_usage();
                }
            }
        }
        "build" | "run" => {
            if args.len() < 3 {
                print_usage();
                return;
            }

            // Parse --target flag: "c" or "go" (default: "go")
            let (target, file_path) = parse_build_args(&args[2..]);
            let file_path = match file_path {
                Some(f) => f,
                None => {
                    eprintln!("Error: No file specified.");
                    print_usage();
                    return;
                }
            };

            if command == "build" {
                compile_file(&file_path, &target);
            } else {
                let exe_path = compile_file(&file_path, &target);
                if let Some(exe) = exe_path {
                    println!("--- Running {} ---", exe);
                    let status = Command::new(format!("./{}", exe))
                        .status()
                        .expect("Failed to execute compiled binary");
                    println!("--- Process exited with status: {} ---", status);
                }
            }
        }
        _ => {
            print_usage();
        }
    }
}

fn print_usage() {
    println!("M-Lang (Myanmar Language) Compiler");
    println!("Usage:");
    println!("  mlang build <file.ml>                  Build (default: Go backend)");
    println!("  mlang build --target c <file.ml>       Build with C backend");
    println!("  mlang build --target go <file.ml>      Build with Go backend");
    println!("  mlang run <file.ml>                    Build and run");
    println!("  mlang run --target c <file.ml>         Build and run with C backend");
    println!("  mlang fmt <file.ml>                    Format source file in place");
    println!("  mlang fmt --check <file.ml>            Check if file is formatted");
}

fn parse_build_args(args: &[String]) -> (String, Option<String>) {
    let mut target = "go".to_string(); // Default to Go backend
    let mut file = None;
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--target" {
            if i + 1 < args.len() {
                target = args[i + 1].to_lowercase();
                i += 2;
                continue;
            }
        } else {
            file = Some(args[i].clone());
        }
        i += 1;
    }
    (target, file)
}

fn parse_fmt_args(args: &[String]) -> (bool, Option<String>) {
    let mut check = false;
    let mut file = None;
    for arg in args {
        if arg == "--check" {
            check = true;
        } else {
            file = Some(arg.clone());
        }
    }
    (check, file)
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
    } else {
        if source == formatted {
            println!("{} is already formatted.", file_path);
        } else {
            if let Err(e) = fs::write(file_path, &formatted) {
                eprintln!("Error writing file {}: {}", file_path, e);
                std::process::exit(1);
            }
            println!("Formatted {}.", file_path);
        }
    }
}

fn compile_file(file_path: &str, target: &str) -> Option<String> {
    let source_code = match fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file {}: {}", file_path, e);
            return None;
        }
    };

    println!("-> Lexing and Parsing {}", file_path);
    let mut lexer = Lexer::new(&source_code);
    let mut parser = Parser::new(&mut lexer);

    let program = match parser.parse_program() {
        Some(p) => p,
        None => {
            eprintln!("Failed to parse program.");
            return None;
        }
    };

    if !parser.errors.is_empty() {
        eprintln!("Syntax Errors:");
        for err in &parser.errors {
            eprintln!("  {}", err);
        }
        return None;
    }

    println!("-> Type Checking {}", file_path);
    let mut type_checker = TypeChecker::new();
    let mut env = Environment::new();
    type_checker.check_program(&program, &mut env);

    if !type_checker.errors.is_empty() {
        eprintln!("Type Errors:");
        for err in &type_checker.errors {
            eprintln!("  {}", err);
        }
        return None;
    }

    let path = Path::new(file_path);
    let file_stem = path.file_stem().unwrap().to_str().unwrap();
    let exe_file_name = format!("{}.exe", file_stem);

    match target {
        "c" => compile_with_c_backend(&program, file_stem, &exe_file_name),
        "go" => compile_with_go_backend(&program, file_stem, &exe_file_name),
        _ => {
            eprintln!("Unknown target '{}'. Use 'c' or 'go'.", target);
            None
        }
    }
}

fn compile_with_c_backend(program: &mlang::ast::Program, file_stem: &str, exe_file_name: &str) -> Option<String> {
    println!("-> Generating C Code");
    let mut codegen = CodeGenerator::new();
    let c_code = codegen.generate(program);

    let c_file_name = format!("{}.c", file_stem);

    if let Err(e) = fs::write(&c_file_name, &c_code) {
        eprintln!("Failed to write intermediate C file: {}", e);
        return None;
    }

    println!("-> Compiling Native Executable ({}) using gcc", exe_file_name);
    let output = Command::new("gcc")
        .arg(&c_file_name)
        .arg("-o")
        .arg(exe_file_name)
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
            eprintln!("Please ensure `gcc` (MinGW-w64 on Windows) is installed and available in your PATH.");
            return None;
        }
    }

    println!("-> Compilation Successful! Executable saved as {}", exe_file_name);
    Some(exe_file_name.to_string())
}

fn compile_with_go_backend(program: &mlang::ast::Program, file_stem: &str, exe_file_name: &str) -> Option<String> {
    println!("-> Generating Go Code");
    let mut codegen = GoCodeGenerator::new();
    let go_code = codegen.generate(program);

    let go_file_name = format!("{}.go", file_stem);

    if let Err(e) = fs::write(&go_file_name, &go_code) {
        eprintln!("Failed to write intermediate Go file: {}", e);
        return None;
    }

    println!("-> Compiling Native Executable ({}) using go build", exe_file_name);
    let output = Command::new("go")
        .arg("build")
        .arg("-o")
        .arg(exe_file_name)
        .arg(&go_file_name)
        .output();

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

    println!("-> Compilation Successful! Executable saved as {}", exe_file_name);

    // Clean up intermediate .go file
    // let _ = fs::remove_file(&go_file_name);

    Some(exe_file_name.to_string())
}
