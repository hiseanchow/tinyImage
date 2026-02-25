fn main() {
    #[cfg(target_os = "macos")]
    {
        cc::Build::new()
            .file("src/service_handler.m")
            .flag("-fobjc-arc")
            .compile("service_handler");
        println!("cargo:rustc-link-lib=framework=AppKit");
        println!("cargo:rustc-link-lib=framework=Foundation");
    }

    tauri_build::build()
}
