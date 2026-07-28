use std::env;

fn main() {
    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let processor_architecture = match env::var("CARGO_CFG_TARGET_ARCH").as_deref() {
            Ok("x86_64") => "amd64",
            Ok("x86") => "x86",
            Ok("aarch64") => "arm64",
            Ok(architecture) => panic!("unsupported Windows target architecture: {architecture}"),
            Err(error) => panic!("CARGO_CFG_TARGET_ARCH is unavailable: {error}"),
        };

        println!(
            "cargo::rustc-link-arg-tests=/MANIFESTDEPENDENCY:\"type='win32' name='Microsoft.Windows.Common-Controls' version='6.0.0.0' processorArchitecture='{processor_architecture}' publicKeyToken='6595b64144ccf1df' language='*'\""
        );
    }

    tauri_build::build();
}
