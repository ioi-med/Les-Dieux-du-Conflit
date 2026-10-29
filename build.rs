fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        let mut res = winres::WindowsResource::new();
        res.set_icon("windows/icon.ico");
        if cfg!(not(windows)) { res.set_windres_path("x86_64-w64-mingw32-windres"); res.set_ar_path("x86_64-w64-mingw32-ar"); }
        res.set("ProductName", "Les Dieux du Conflit");
        res.set("FileDescription", "Les Dieux du Conflit - jeu d'action");
        res.compile().expect("Impossible de compiler l'icône Windows");
        if std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default() == "gnu" {
            println!("cargo:rustc-link-arg={}/resource.o", std::env::var("OUT_DIR").unwrap());
        }
    }
}
