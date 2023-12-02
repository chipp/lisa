fn main() {
    if cfg!(target_os = "linux") {
        pkg_config::Config::new()
            .statik(true)
            .probe("bluez")
            .unwrap();
    }
}
