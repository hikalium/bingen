extern crate proc_macro;
use proc_macro::TokenStream;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::process::{Command, Stdio};
use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::{parse_macro_input, LitStr};
use tempfile::tempdir;

struct Args {
    arch: LitStr,
    asm: LitStr,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let arch = input.parse()?;
        input.parse::<syn::Token![,]>()?;
        let asm = input.parse()?;
        Ok(Args { arch, asm })
    }
}

struct LLVMPath {
    clang: String,
    llvm_objcopy: String,
}

#[cfg(target_os = "macos")]
fn get_llvm_path() -> LLVMPath {
    let brew = Command::new("brew")
        .args(["--prefix", "llvm"])
        .output()
        .expect("Failed to run brew command");
    assert!(
        brew.status.success(),
        "Failed to get llvm path from brew: {}",
        String::from_utf8_lossy(&brew.stderr)
    );
    let mut base_path = String::from_utf8_lossy(&brew.stdout).to_string();
    if base_path.ends_with('\n') {
        base_path.pop();
        if base_path.ends_with('\r') {
            base_path.pop();
        }
    }
    LLVMPath {
        clang: base_path.clone() + &"/bin/clang".to_string(),
        llvm_objcopy: base_path + &"/bin/llvm-objcopy".to_string(),
    }
}

#[cfg(not(target_os = "macos"))]
fn get_llvm_path() -> LLVMPath {
    use std::path::Path;
    let which = Command::new("which")
        .args(["clang-8"])
        .output()
        .expect("Failed to run which command");
    assert!(
        which.status.success(),
        "Failed to get llvm path from which: {}",
        String::from_utf8_lossy(&which.stderr)
    );
    let clang_path = String::from_utf8_lossy(&which.stdout).to_string();
    let base_path = Path::new(&clang_path)
        .parent()
        .expect("Failed to take a parent path")
        .to_str()
        .expect("Failed to convert from Path to &str")
        .to_string();
    LLVMPath {
        clang: base_path.clone() + &"/clang-8".to_string(),
        llvm_objcopy: base_path + &"/llvm-objcopy-8".to_string(),
    }
}

#[proc_macro]
pub fn bingen(input: TokenStream) -> TokenStream {
    let Args { arch, asm } = parse_macro_input!(input as Args);

    let LLVMPath {
        clang,
        llvm_objcopy,
    } = get_llvm_path();

    let dir = tempdir().expect("Failed to create a temp dir");

    let mut input = File::create(dir.path().join("bingen.S")).unwrap();
    input.write_all(asm.value().as_bytes()).unwrap();

    Command::new(clang)
        .args([
            "-target",
            &arch.value(),
            "-xassembler-with-cpp",
            "-o",
            dir.path()
                .join("bingen.o")
                .to_str()
                .expect("Failed to create a str from path"),
            "-c",
            dir.path()
                .join("bingen.S")
                .to_str()
                .expect("Failed to create a str from path"),
        ])
        .output()
        .expect("Failed to run clang");

    Command::new(llvm_objcopy)
        .args([
            "-O",
            "binary",
            dir.path()
                .join("bingen.o")
                .to_str()
                .expect("Failed to create a str from path"),
            dir.path()
                .join("bingen.bin")
                .to_str()
                .expect("Failed to create a str from path"),
        ])
        .stdout(Stdio::piped())
        .output()
        .expect("Failed to run objcopy");

    format!(
        "{:?}",
        fs::read(dir.path().join("bingen.bin")).expect("Failed to open /tmp/bingen.bin")
    )
    .parse()
    .unwrap()
}
