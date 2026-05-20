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

    // ── All targets: pre-decode a 256×256 RGBA bitmap and emit raw
    // bytes to `OUT_DIR/icon.rgba`.  The runtime `include_bytes!`s
    // this and hands it to winit via `WindowAttributes::with_window_icon`
    // so the dock / Alt-Tab / taskbar shows the exhale icon for any
    // window the app creates.  Especially relevant on Linux where a
    // raw binary run from a terminal has no `.desktop` file installed
    // and the compositor otherwise falls back to a generic icon
    let rgba_path = std::path::PathBuf::from(&out_dir).join("icon.rgba");
    if let Err(e) = png_to_rgba_256(&src_png, &rgba_path) {
        println!("cargo:warning=icon PNG→RGBA conversion failed: {e}; \
                  binary windows will use the platform default icon");
    }

    // ── Windows: also produce a multi-resolution .ico and embed it
    // as an .exe resource so Explorer / Start / Alt-Tab pick it up
    // (separate from the window icon — Windows uses BOTH)
    if target_os != "windows" { return; }
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

/// Decode the source PNG, resize to 256×256, write raw RGBA bytes
/// (no headers — just `w*h*4` bytes) for the runtime to load.
fn png_to_rgba_256(
    src: &std::path::Path,
    dst: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let img = image::open(src)?.to_rgba8();
    let resized = image::imageops::resize(
        &img, 256, 256, image::imageops::FilterType::Lanczos3,
    );
    std::fs::write(dst, resized.into_raw())?;
    Ok(())
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
