extern crate proc_macro;
use proc_macro::TokenStream;
use std::io::Write;
use std::process::{Command, Stdio};
use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::{parse_macro_input, LitStr};

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

#[proc_macro]
pub fn bingen(input: TokenStream) -> TokenStream {
    let Args { arch, asm } = parse_macro_input!(input as Args);

    let brew = Command::new("brew")
        .args(["--prefix", "llvm"])
        .output()
        .expect("Failed to run brew command");
    if !brew.status.success() {
        panic!(
            "Failed to get llvm path from brew: {}",
            String::from_utf8_lossy(&brew.stderr)
        );
    }
    let mut clang_path = String::from_utf8_lossy(&brew.stdout).to_string();
    if clang_path.ends_with('\n') {
        clang_path.pop();
        if clang_path.ends_with('\r') {
            clang_path.pop();
        }
    }
    let mut clang = Command::new(clang_path.clone() + &"/bin/clang".to_string())
        .args([
            "-target",
            &arch.value(),
            "-xassembler-with-cpp",
            "-o",
            "-",
            "-c",
            "-",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to run clang");

    let stdin = clang.stdin.as_mut().unwrap();
    stdin.write_all(asm.value().as_bytes()).unwrap();

    if let Some(clang_output) = clang.stdout.take() {
        let objcopy = Command::new(clang_path + &"/bin/llvm-objcopy".to_string())
            .args(["-O", "binary", "-"])
            .stdin(clang_output)
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        drop(clang);
        let output = objcopy.wait_with_output().unwrap();
        format!("{:?}", output.stdout).parse().unwrap()
    } else {
        panic!("failed to take stdout of clang");
    }
}
