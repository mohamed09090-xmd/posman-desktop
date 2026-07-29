use std::{env, io, path::PathBuf};

fn main() {
    if let Err(error) = build() {
        panic!("failed to run the POSMAN build script: {error:#}");
    }
}

fn build() -> tauri_build::Result<()> {
    let target_os = env::var("CARGO_CFG_TARGET_OS").map_err(|error| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!("CARGO_CFG_TARGET_OS is unavailable: {error}"),
        )
    })?;
    let target_env = env::var("CARGO_CFG_TARGET_ENV").map_err(|error| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!("CARGO_CFG_TARGET_ENV is unavailable: {error}"),
        )
    })?;
    let is_windows_msvc = target_os == "windows" && target_env == "msvc";

    let attributes = if is_windows_msvc {
        tauri_build::Attributes::new()
            .windows_attributes(tauri_build::WindowsAttributes::new_without_app_manifest())
    } else {
        tauri_build::Attributes::new()
    };

    tauri_build::try_build(attributes)?;

    if is_windows_msvc {
        emit_windows_manifest_linker_args()?;
    }

    Ok(())
}

fn emit_windows_manifest_linker_args() -> tauri_build::Result<()> {
    let manifest_dir = env::var_os("CARGO_MANIFEST_DIR").ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, "CARGO_MANIFEST_DIR is unavailable")
    })?;
    let manifest_path = PathBuf::from(manifest_dir).join("windows-app-manifest.xml");
    let manifest_path = manifest_path.canonicalize().map_err(|error| {
        io::Error::new(
            error.kind(),
            format!(
                "failed to resolve Windows application manifest at {}: {error}",
                manifest_path.display()
            ),
        )
    })?;
    let manifest_path = manifest_path.to_str().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "Windows application manifest path is not valid UTF-8: {}",
                manifest_path.display()
            ),
        )
    })?;

    println!("cargo::rerun-if-changed={manifest_path}");
    println!("cargo::rustc-link-arg=/MANIFEST:EMBED");
    println!("cargo::rustc-link-arg=/MANIFESTINPUT:{manifest_path}");
    println!("cargo::rustc-link-arg=/WX");

    Ok(())
}
