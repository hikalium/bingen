extern crate proc_macro;
use proc_macro::TokenStream;
use std::env;
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

#[derive(Debug)]
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
    let supported_versions = (6..=13).rev();
    let mut clang_versions: Vec<String> = vec!["clang".to_string()];
    let mut objcopy_versions: Vec<String> = vec!["llvm-objcopy".to_string()];
    for v in supported_versions {
        clang_versions.extend_from_slice(&[
            format!("clang-{}", v),
            format!("clang-{}.0", v),
            format!("clang-{}0", v),
        ]);
        objcopy_versions.extend_from_slice(&[
            format!("llvm-objcopy-{}", v),
            format!("llvm-objcopy-{}.0", v),
            format!("llvm-objcopy-{}0", v),
        ]);
    }

    let clang_results = clang_versions.iter().map(|x| which::which(x));
    let objcopy_results = objcopy_versions.iter().map(|x| which::which(x));
    let results = clang_results.zip(objcopy_results);
    let mut results = results.filter(|x| x.0.is_ok() && x.1.is_ok());

    let path = results
        .nth(0)
        .unwrap_or_else(|| panic!("clang or llvm-objcopy not found"));

    LLVMPath {
        clang: path.0.unwrap().to_string_lossy().to_string(),
        llvm_objcopy: path.1.unwrap().to_string_lossy().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn print_llvm_path() {
        let llvm_path = get_llvm_path();
        println!("{:?}", llvm_path);
    }
}

#[proc_macro]
pub fn bingen(input: TokenStream) -> TokenStream {
    let Args { arch, asm } = parse_macro_input!(input as Args);

    let env_clang_path = env::var("BINGEN_CLANG_PATH");
    let env_objcopy_path = env::var("BINGEN_OBJCOPY_PATH");
    let LLVMPath {
        clang,
        llvm_objcopy,
    } = if env_clang_path.is_err() && env_objcopy_path.is_err() {
        get_llvm_path()
    } else {
        LLVMPath {
            clang: env_clang_path
                .as_ref()
                .expect("BINGEN_CLANG_PATH is not set")
                .to_string(),
            llvm_objcopy: env_objcopy_path
                .as_ref()
                .expect("BINGEN_OBJCOPY_PATH is not set")
                .to_string(),
        }
    };

    let dir = tempdir().expect("Failed to create a temp dir");

    let mut input = File::create(dir.path().join("bingen.S")).unwrap();
    input.write_all(asm.value().as_bytes()).unwrap();

    let result = Command::new(clang.clone())
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
    assert!(
        result.status.success(),
        "{} returned {:?}. stderr:\n{}",
        clang,
        result.status.code().expect("exit code not set"),
        String::from_utf8_lossy(&result.stderr)
    );

    let result = Command::new(llvm_objcopy.clone())
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

    assert!(
        result.status.success(),
        "{} returned {:?}. stderr:\n{}",
        llvm_objcopy,
        result.status.code().expect("exit code not set"),
        String::from_utf8_lossy(&result.stderr)
    );

    format!(
        "{:?}",
        fs::read(dir.path().join("bingen.bin")).expect("Failed to open /tmp/bingen.bin")
    )
    .parse()
    .unwrap()
}
