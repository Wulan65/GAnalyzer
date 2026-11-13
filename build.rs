fn main() {
    println!("cargo:rerun-if-changed=assets/ganalyzer.ico");

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut resources = winres::WindowsResource::new();
        resources
            .set_icon("assets/ganalyzer.ico")
            .set("FileDescription", "GAnalyzer")
            .set("ProductName", "GAnalyzer")
            .set("OriginalFilename", "ganalyzer.exe");
        resources
            .compile()
            .expect("failed to embed GAnalyzer icon and metadata");
    }
}
