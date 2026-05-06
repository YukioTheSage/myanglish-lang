use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

struct NativeCase {
    source: &'static str,
    stem: &'static str,
    expected_stdout: &'static str,
}

#[test]
fn phase1_examples_compile_and_run_with_llvm_backend() {
    let cases = [
        NativeCase {
            source: "examples/phase1/01_struct_and_collection_mutation.ml",
            stem: "01_struct_and_collection_mutation",
            expected_stdout: "Ko Ko\n2\n[\"tea\", \"latte\", \"cake\"]\n2200\n",
        },
        NativeCase {
            source: "examples/phase1/02_loop_control_with_index.ml",
            stem: "02_loop_control_with_index",
            expected_stdout: "6400\n",
        },
        NativeCase {
            source: "examples/phase1/03_error_handling_and_closure.ml",
            stem: "03_error_handling_and_closure",
            expected_stdout: "Cannot divide\ndivision by zero\n4750\n",
        },
        NativeCase {
            source: "examples/phase1/04_classic_for_loop.ml",
            stem: "04_classic_for_loop",
            expected_stdout: "0\n1\n2\n3\n4\n",
        },
        NativeCase {
            source: "examples/phase1/05_llvm_native_store_demo.ml",
            stem: "05_llvm_native_store_demo",
            expected_stdout: "Aye Aye\n[\"tea\", \"rice\", \"cake\"]\n2500\n540\n5640\n",
        },
    ];

    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workdir = fresh_workdir("mlang-llvm-e2e");
    fs::create_dir_all(&workdir).expect("create temp build dir");
    fs::copy(repo.join("runtime_llvm.c"), workdir.join("runtime_llvm.c"))
        .expect("copy LLVM runtime into temp build dir");

    for case in cases {
        let source = repo.join(case.source);
        let build = Command::new(env!("CARGO_BIN_EXE_mlang"))
            .current_dir(&workdir)
            .arg("build")
            .arg("--target")
            .arg("llvm")
            .arg(&source)
            .output()
            .expect("run mlang LLVM build");

        assert!(
            build.status.success(),
            "LLVM build failed for {}\nstdout:\n{}\nstderr:\n{}",
            case.source,
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );

        let run = Command::new(workdir.join(executable_name(case.stem)))
            .output()
            .expect("run generated native executable");
        assert!(
            run.status.success(),
            "native executable failed for {}\nstdout:\n{}\nstderr:\n{}",
            case.source,
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        assert_eq!(
            normalize_newlines(&String::from_utf8_lossy(&run.stdout)),
            case.expected_stdout
        );
    }

    let _ = fs::remove_dir_all(&workdir);
}

fn fresh_workdir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("{}-{}-{}", prefix, std::process::id(), nanos))
}

fn executable_name(stem: &str) -> String {
    format!("{}{}", stem, native_binary_suffix())
}

fn native_binary_suffix() -> &'static str {
    if cfg!(windows) {
        ".exe"
    } else {
        ""
    }
}

fn normalize_newlines(text: &str) -> String {
    text.replace("\r\n", "\n")
}
