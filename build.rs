use std::env;
use std::path::PathBuf;

fn main() {
    // Note: Cargo does not natively execute build scripts placed at the root of a virtual workspace.
    // This script is placed here per architectural requirements. If it is run (e.g. via a custom runner),
    // it will generate the bootimage-usya.bin using the boot binary.

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap_or_else(|_| "target".into()));
    
    // Resolve the compiled boot binary.
    let kernel_path = env::var("CARGO_BIN_FILE_BOOT_BOOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("target/x86_64-unknown-none/debug/boot"));

    if kernel_path.exists() {
        let uefi_path = out_dir.join("bootimage-usya-uefi.efi");
        let bios_path = out_dir.join("bootimage-usya-bios.bin");

        // Construct UEFI image
        bootloader::UefiBoot::new(&kernel_path)
            .create_disk_image(&uefi_path)
            .expect("Failed to create UEFI boot image");

        // Construct BIOS image
        bootloader::BiosBoot::new(&kernel_path)
            .create_disk_image(&bios_path)
            .expect("Failed to create BIOS boot image");

        println!("cargo:rerun-if-changed={}", kernel_path.display());
    } else {
        println!("cargo:warning=Kernel binary not found at {}. Please compile the 'boot' crate first.", kernel_path.display());
    }
}
