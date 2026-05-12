// Build script that, when the target is Windows, converts the shared
// `exhaleColorGradient512.png` icon to a multi-resolution `.ico` and
// embeds it as a resource in the compiled `.exe` so Explorer / Start
// menu / Alt-Tab show the proper exhale icon instead of the generic
// Rust executable placeholder.
//
// On non-Windows targets this build script is a no-op.  On Windows
// targets, if the host machine lacks a resource compiler (`rc.exe` /
// `windres`), the build still succeeds — the `.exe` just falls back
// to the generic icon and we emit a `cargo:warning` so the cause is
// visible in build logs.

fn main() {
    // `CARGO_CFG_TARGET_OS` is set by Cargo for build scripts, telling
    // us what OS we're compiling FOR (the host OS comes from
    // `std::env::consts::OS`, which is different).
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os != "windows" { return; }

    let src_png = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..").join("..").join("..")
        .join("swift").join("exhale").join("Assets.xcassets")
        .join("AppIcon.appiconset")
        .join("exhaleColorGradient512.png");
    if !src_png.exists() {
        println!("cargo:warning=icon source PNG not found at {}; skipping embed",
            src_png.display());
        return;
    }
    println!("cargo:rerun-if-changed={}", src_png.display());

    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR set by cargo");
    let ico_path = std::path::PathBuf::from(&out_dir).join("exhale.ico");

    if let Err(e) = png_to_ico(&src_png, &ico_path) {
        println!("cargo:warning=PNG→ICO conversion failed: {e}; skipping embed");
        return;
    }

    let mut res = winresource::WindowsResource::new();
    res.set_icon(ico_path.to_str().expect("ICO path is UTF-8"));
    if let Err(e) = res.compile() {
        // Most common cause: building from a non-Windows host without
        // `windres` (from `mingw-w64`) installed.  Not fatal — the
        // `.exe` just gets the generic icon.  CI's Windows runner
        // ships RC tools by default and always succeeds here.
        println!("cargo:warning=icon resource embed failed: {e}; \
                  binary will have the default Windows icon");
    }
}

fn png_to_ico(
    src: &std::path::Path,
    dst: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let src_img = image::open(src)?.to_rgba8();
    let mut icon_dir = ico::IconDir::new(ico::ResourceType::Icon);
    // Standard Windows icon sizes — Explorer / Start use whichever
    // one matches the display DPI best.  256 is the modern HiDPI
    // tile; 16/32/48 are the small/medium/large taskbar/notification
    // sizes; 64/128 cover intermediate scales.
    for size in [16u32, 32, 48, 64, 128, 256] {
        let resized = image::imageops::resize(
            &src_img, size, size, image::imageops::FilterType::Lanczos3,
        );
        let entry = ico::IconImage::from_rgba_data(size, size, resized.into_raw());
        icon_dir.add_entry(ico::IconDirEntry::encode(&entry)?);
    }
    let f = std::fs::File::create(dst)?;
    icon_dir.write(f)?;
    Ok(())
}
