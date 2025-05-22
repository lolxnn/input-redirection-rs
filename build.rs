use std::{env, fs, io::Write, path::Path};
#[cfg(windows)]
use winresource::WindowsResource;

fn main() -> std::io::Result<()> {
    // Embed ICO icon for Windows executables
    #[cfg(windows)]
    {
        WindowsResource::new()
            .set_icon("assets/app.ico")
            .compile()?;
    }

    // Convert PNG to RGBA and output as a binary blob for include_bytes!
    let out_dir = env::var("OUT_DIR").unwrap();
    let input_png = "assets/app.png";
    let output_rgba = Path::new(&out_dir).join("app_icon.rgba");

    let img = image::open(input_png).expect("Failed to open icon PNG").into_rgba8();
    let (w, h) = img.dimensions();
    let mut file = fs::File::create(&output_rgba)?;

    file.write_all(&w.to_le_bytes())?;
    file.write_all(&h.to_le_bytes())?;
    file.write_all(&img.into_raw())?;

    println!("cargo:rerun-if-changed={}", input_png);

    Ok(())
}