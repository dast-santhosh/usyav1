fn main() {
    let bios = bootloader::BiosBoot::new(std::path::Path::new("dummy"));
    // check if create_cdimage or create_iso exists
    let _ = bios.create_iso;
}
