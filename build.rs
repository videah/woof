use std::{
    path::Path,
    process::Command,
};

fn main() {
    let dir = "woof-passkey-login"; // update to your directory
    println!("cargo:rerun-if-changed={}/", dir);

    let dest_path = Path::new(&dir).join("pkg");
    let output = Command::new("wasm-pack")
        .args(&["build", "--target", "web"])
        .arg(dir)
        .output()
        .expect("To build wasm files successfully");

    if !output.status.success() {
        panic!(
            "Error while compiling:\n{}",
            String::from_utf8_lossy(&output.stdout)
        );
    }

    let js_file = dest_path.join("woof_passkey_login.js");
    let wasm_file = dest_path.join("woof_passkey_login_bg.wasm");

    for file in &[&js_file, &wasm_file] {
        let file = std::fs::metadata(file).expect("file to exist");
        assert!(file.is_file());
    }

    println!("cargo:rustc-env=PROJECT_NAME_JS={}", js_file.display());
    println!("cargo:rustc-env=PROJECT_NAME_WASM={}", wasm_file.display());

    // Copy the files to the static directory
    let static_dir = Path::new("static");
    std::fs::create_dir_all(&static_dir).expect("to create static directory");
    std::fs::copy(&js_file, static_dir.join("woof_passkey_login.js")).expect("to copy js file");
    std::fs::copy(&wasm_file, static_dir.join("woof_passkey_login_bg.wasm"))
        .expect("to copy wasm file");
}
