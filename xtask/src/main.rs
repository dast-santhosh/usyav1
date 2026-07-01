use std::process::Command;
use std::path::{Path, PathBuf};
use std::env;

fn main() {
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    
    println!("=== Building the USYA Nano-Core ===");
    let status = Command::new(&cargo)
        .current_dir(workspace_root())
        .args(&[
            "build",
            "--package", "boot",
            "--target", "x86_64-unknown-none",
            "-Z", "build-std=core,compiler_builtins,alloc",
            "-Z", "build-std-features=compiler-builtins-mem",
        ])
        .status()
        .expect("Failed to execute cargo build");

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    println!("=== Kernel built successfully. Creating Boot Images ===");
    
    // The compiled binary will be located here:
    let kernel_binary = workspace_root().join("target/x86_64-unknown-none/debug/boot");
    let out_dir = workspace_root().join("target/bootimage");
    
    std::fs::create_dir_all(&out_dir).expect("Failed to create bootimage directory");

    let bios_path = out_dir.join("bootimage-bios.bin");
    let uefi_path = out_dir.join("bootimage-uefi.efi");

    bootloader::BiosBoot::new(&kernel_binary)
        .create_disk_image(&bios_path)
        .expect("Failed to create BIOS boot image");

    bootloader::UefiBoot::new(&kernel_binary)
        .create_disk_image(&uefi_path)
        .expect("Failed to create UEFI boot image");

    let iso_path = out_dir.join("usya.iso");
    if let Err(e) = std::fs::copy(&bios_path, &iso_path) {
        println!("Warning: Failed to copy bin to ISO: {}", e);
    }

    println!("Success! Boot images created at:");
    println!("  BIOS: {}", bios_path.display());
    println!("  UEFI: {}", uefi_path.display());
    println!("  ISO:  {}", iso_path.display());
}

fn workspace_root() -> PathBuf {
    Path::new(&env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(1)
        .unwrap()
        .to_path_buf()
}
