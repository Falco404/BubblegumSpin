//! Headless screenshot: render one frame to `out/m1.png` with no window.

use bubblegum_spin::gfx::{fb::Fb, palette_card};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut fb = Fb::new();
    palette_card(&mut fb);

    std::fs::create_dir_all("out")?;
    fb.to_png("out/m1.png")?;
    println!("wrote out/m1.png");
    Ok(())
}
