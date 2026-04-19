fn main() {
    let build_target = format_build_target();

    println!("cargo:rustc-env=SUON_BUILD_TARGET={build_target}");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut resource = winresource::WindowsResource::new();
        resource.set_icon("assets/suon.ico");
        resource.compile().unwrap();
    }
}

fn format_build_target() -> String {
    let arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_else(|_| "unknown".to_string());
    let os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_else(|_| "unknown".to_string());
    format!("{arch}-{os}")
}
