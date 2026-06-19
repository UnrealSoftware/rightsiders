#[cfg(not(target_arch = "wasm32"))]
#[path = "../assets.rs"]
mod assets;

#[cfg(not(target_arch = "wasm32"))]
use std::path::Path;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    println!("Generating procedural assets...");
    let game_assets = assets::generate_assets();

    let wall_size = 64;
    let sprite_size = 64;

    // 1. Export walls (arranged in a grid depending on count)
    let num_walls = game_assets.walls.len();
    let walls_cols = if num_walls <= 4 { 2 } else { 4 };
    let walls_rows = if num_walls <= 4 { 2 } else { 4 };
    let walls_width = wall_size * walls_cols;
    let walls_height = wall_size * walls_rows;
    let mut walls_pixels = vec![0u8; walls_width * walls_height * 4];

    for w_idx in 0..num_walls {
        let wall = &game_assets.walls[w_idx];
        let grid_col = w_idx % walls_cols;
        let grid_row = w_idx / walls_cols;
        
        for y in 0..wall_size {
            for x in 0..wall_size {
                let pixel = wall.pixels[y * wall_size + x];
                let out_x = grid_col * wall_size + x;
                let out_y = grid_row * wall_size + y;
                let idx = (out_y * walls_width + out_x) * 4;
                walls_pixels[idx]     = ((pixel >> 24) & 0xff) as u8; // R
                walls_pixels[idx + 1] = ((pixel >> 16) & 0xff) as u8; // G
                walls_pixels[idx + 2] = ((pixel >> 8) & 0xff) as u8;  // B
                walls_pixels[idx + 3] = (pixel & 0xff) as u8;         // A
            }
        }
    }

    let walls_path = Path::new("src/assets/walls.png");
    image::save_buffer(
        walls_path,
        &walls_pixels,
        walls_width as u32,
        walls_height as u32,
        image::ColorType::Rgba8,
    ).expect("Failed to save walls.png");
    println!("Saved {} walls in {}x{} grid to {:?}", num_walls, walls_cols, walls_rows, walls_path);

    // 2. Export sprites (25 sprites, arranged in a 5x5 grid: 320x320 pixels)
    let num_sprites = game_assets.sprites.len();
    let sprites_cols = 5;
    let sprites_rows = 5;
    let sprites_width = sprite_size * sprites_cols;
    let sprites_height = sprite_size * sprites_rows;
    let mut sprites_pixels = vec![0u8; sprites_width * sprites_height * 4];

    for s_idx in 0..num_sprites {
        let sprite = &game_assets.sprites[s_idx];
        let grid_col = s_idx % sprites_cols;
        let grid_row = s_idx / sprites_cols;
        
        for y in 0..sprite_size {
            for x in 0..sprite_size {
                let pixel = sprite.pixels[y * sprite_size + x];
                let out_x = grid_col * sprite_size + x;
                let out_y = grid_row * sprite_size + y;
                let idx = (out_y * sprites_width + out_x) * 4;
                sprites_pixels[idx]     = ((pixel >> 24) & 0xff) as u8; // R
                sprites_pixels[idx + 1] = ((pixel >> 16) & 0xff) as u8; // G
                sprites_pixels[idx + 2] = ((pixel >> 8) & 0xff) as u8;  // B
                sprites_pixels[idx + 3] = (pixel & 0xff) as u8;         // A
            }
        }
    }

    let sprites_path = Path::new("src/assets/sprites.png");
    image::save_buffer(
        sprites_path,
        &sprites_pixels,
        sprites_width as u32,
        sprites_height as u32,
        image::ColorType::Rgba8,
    ).expect("Failed to save sprites.png");
    println!("Saved {} sprites in 5x5 grid to {:?}", num_sprites, sprites_path);

    println!("Asset export complete!");
}

#[cfg(target_arch = "wasm32")]
fn main() {}
