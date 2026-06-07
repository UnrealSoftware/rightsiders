// RIGHTSIDER // REU-99 Tactical Simulator
// Main entry point for Rust + Macroquad Web Assembly build

mod assets;
mod map;
mod raycaster;
mod game;

use macroquad::prelude::*;
use assets::{generate_assets, SpriteTexture};
use raycaster::{Raycaster, WIDTH, HEIGHT, SpriteToRender};
use game::{GameState, WeaponState, CitizenState};
use map::{TileType, MAP_WIDTH, MAP_HEIGHT};

// Configuration for Macroquad window
fn window_conf() -> Conf {
    Conf {
        window_title: "RIGHTSIDERS // REU-99 Tactical Simulator".to_owned(),
        window_width: 800,
        window_height: 600,
        high_dpi: true,
        fullscreen: false,
        ..Default::default()
    }
}

// Convert procedural CPU texture to GPU Texture2D
fn upload_texture(sprite: &SpriteTexture) -> Texture2D {
    let mut bytes = vec![0u8; sprite.width * sprite.height * 4];
    for i in 0..(sprite.width * sprite.height) {
        let pixel = sprite.pixels[i];
        bytes[i * 4]     = ((pixel >> 24) & 0xff) as u8; // R
        bytes[i * 4 + 1] = ((pixel >> 16) & 0xff) as u8; // G
        bytes[i * 4 + 2] = ((pixel >> 8) & 0xff) as u8;  // B
        bytes[i * 4 + 3] = (pixel & 0xff) as u8;         // A
    }
    let img = Image {
        bytes,
        width: sprite.width as u16,
        height: sprite.height as u16,
    };
    let texture = Texture2D::from_image(&img);
    texture.set_filter(FilterMode::Nearest); // Retro pixelated scaling
    texture
}

// Project a 3D world coordinate (x, y, z) to 2D screen coordinates (with torus wrapping)
fn project_3d(
    x: f32,
    y: f32,
    z: f32,
    player_x: f32,
    player_y: f32,
    dir_x: f32,
    dir_y: f32,
    plane_x: f32,
    plane_y: f32,
    screen_w: f32,
    screen_h: f32,
) -> Option<(f32, f32)> {
    let mut dx = x - player_x;
    if dx > MAP_WIDTH as f32 / 2.0 { dx -= MAP_WIDTH as f32; }
    else if dx < -(MAP_WIDTH as f32 / 2.0) { dx += MAP_WIDTH as f32; }

    let mut dy = y - player_y;
    if dy > MAP_HEIGHT as f32 / 2.0 { dy -= MAP_HEIGHT as f32; }
    else if dy < -(MAP_HEIGHT as f32 / 2.0) { dy += MAP_HEIGHT as f32; }

    // Inverse camera determinant
    let inv_det = 1.0 / (plane_x * dir_y - dir_x * plane_y);
    let transform_x = inv_det * (dir_y * dx - dir_x * dy);
    let transform_y = inv_det * (-plane_y * dx + plane_x * dy); // depth

    // Draw only if in front of player
    if transform_y > 0.08 {
        let sx = (screen_w / 2.0) * (1.0 + transform_x / transform_y);
        let sy = -(screen_h / transform_y) * (z - 0.4) + screen_h / 2.0;
        Some((sx, sy))
    } else {
        None
    }
}

// Helper function to draw text using the pixel font
fn draw_pixel_text(text: &str, x: f32, y: f32, size: f32, color: Color, font: &Font) {
    draw_text_ex(
        text,
        x,
        y,
        TextParams {
            font: Some(font),
            font_size: size as u16,
            color,
            ..Default::default()
        },
    );
}

// Helper to draw a pixelated rectangle outline using sharp block rectangles
fn draw_pixel_rect_lines(x: f32, y: f32, w: f32, h: f32, thickness: f32, color: Color) {
    let x = x.round();
    let y = y.round();
    let w = w.round();
    let h = h.round();
    let t = thickness.round().max(1.0);

    // Top border
    draw_rectangle(x, y, w, t, color);
    // Bottom border
    draw_rectangle(x, y + h - t, w, t, color);
    // Left border
    draw_rectangle(x, y, t, h, color);
    // Right border
    draw_rectangle(x + w - t, y, t, h, color);
}

// Helper to draw an outline circle on a SpriteTexture (midpoint circle algorithm)
fn draw_circle_outline(sprite: &mut SpriteTexture, cx: i32, cy: i32, radius: i32, color: u32) {
    let mut x = radius;
    let mut y = 0;
    let mut err = 1 - radius;

    while x >= y {
        sprite.set_pixel(cx + x, cy + y, color);
        sprite.set_pixel(cx + y, cy + x, color);
        sprite.set_pixel(cx - y, cy + x, color);
        sprite.set_pixel(cx - x, cy + y, color);
        sprite.set_pixel(cx - x, cy - y, color);
        sprite.set_pixel(cx - y, cy - x, color);
        sprite.set_pixel(cx + y, cy - x, color);
        sprite.set_pixel(cx + x, cy - y, color);

        y += 1;
        if err < 0 {
            err += 2 * y + 1;
        } else {
            x -= 1;
            err += 2 * (y - x) + 1;
        }
    }
}

// Procedurally generate a 32x32 pixel-art crosshair texture
fn generate_crosshair_texture() -> Texture2D {
    let mut sprite = SpriteTexture::new(32, 32, 0x00000000); // transparent background
    let color = 0xffffffff; // white color (tintable by draw color)

    // Circle outline of radius 5 in the center (16, 16)
    draw_circle_outline(&mut sprite, 16, 16, 5, color);

    // 1-pixel thick lines from edges towards center
    sprite.draw_line(2, 16, 9, 16, color);
    sprite.draw_line(23, 16, 30, 16, color);
    sprite.draw_line(16, 2, 16, 9, color);
    sprite.draw_line(16, 23, 16, 30, color);

    upload_texture(&sprite)
}

#[macroquad::main(window_conf)]
async fn main() {
    // 1. Generate procedural assets
    let assets_data = generate_assets();
    
    // (Weapon textures upload removed as weapon is hidden)

    // 3. Load and configure pixel font
    let font_bytes = include_bytes!("assets/PressStart2P-Regular.ttf");
    let mut font = load_ttf_font_from_bytes(font_bytes).unwrap();
    font.set_filter(FilterMode::Nearest);

    // Generate low-res pixel crosshair texture
    let crosshair_tex = generate_crosshair_texture();

    // 4. Initialize game state and raycaster
    let mut state = GameState::new();
    let mut raycaster = Raycaster::new();

    // Create CPU pixel buffer and matching GPU texture for raycasting display
    let mut screen_image = Image {
        bytes: vec![0u8; WIDTH * HEIGHT * 4],
        width: WIDTH as u16,
        height: HEIGHT as u16,
    };
    let screen_texture = Texture2D::from_image(&screen_image);
    screen_texture.set_filter(FilterMode::Nearest);

    // Font selection (Default fallback, style elements drawn with rectangles/lines)
    let mut is_game_over = false;
    let mut is_bankrupt = false;

    loop {
        // Frame delta time (capped to prevent physics explosions on lag)
        let dt = get_frame_time().min(0.08);

        // ==========================================
        // INPUT PROCESSING
        // ==========================================
        let mut switch_lane_left = false;
        let mut switch_lane_right = false;

        if !is_game_over && !is_bankrupt {
            // A / D to switch lane
            if is_key_pressed(KeyCode::A) {
                switch_lane_left = true;
            }
            if is_key_pressed(KeyCode::D) {
                switch_lane_right = true;
            }

            // Shooting: Spacebar only
            if is_key_pressed(KeyCode::Space) {
                state.trigger_fire();
            }
        } else {
            // Press R to restart simulation
            if is_key_pressed(KeyCode::R) {
                state = GameState::new();
                is_game_over = false;
                is_bankrupt = false;
            }
        }

        // ==========================================
        // GAME STATE UPDATE
        // ==========================================
        if !is_game_over && !is_bankrupt {
            state.move_player(switch_lane_left, switch_lane_right);
            state.update(dt);

            // Check loss conditions
            if state.player.health <= 0.0 {
                is_game_over = true;
            }
            if state.player.credits <= -1000 {
                is_bankrupt = true;
            }
        }

        // ==========================================
        // 3D RAYCASTING STEP
        // ==========================================
        raycaster.clear();
        
        // 1. Cast Floor & Sidewalk markings
        raycaster.cast_floor(
            state.player.x,
            state.player.y,
            state.player.dir_x,
            state.player.dir_y,
            state.player.plane_x,
            state.player.plane_y,
            &state.map,
        );

        // 2. Cast building walls
        raycaster.cast_walls(
            state.player.x,
            state.player.y,
            state.player.dir_x,
            state.player.dir_y,
            state.player.plane_x,
            state.player.plane_y,
            &state.map,
            &assets_data,
        );

        // 3. Populate sprite list for raycasting
        let mut sprites_to_draw = Vec::new();
        for citizen in &state.citizens {
            let tex_idx = match citizen.state {
                CitizenState::Walking => {
                    // Vector from player to citizen
                    let dx = citizen.x - state.player.x;
                    let dy = citizen.y - state.player.y;

                    // Citizen direction vector
                    let mut c_dir_x = citizen.next_tx as f32 - citizen.tx as f32;
                    let mut c_dir_y = citizen.next_ty as f32 - citizen.ty as f32;
                    let len = (c_dir_x * c_dir_x + c_dir_y * c_dir_y).sqrt();
                    if len > 0.01 {
                        c_dir_x /= len;
                        c_dir_y /= len;
                    }

                    // Dot product to check if seen from front or back
                    let dot = dx * c_dir_x + dy * c_dir_y;
                    let seen_from_back = dot > 0.0;

                    let frame_base = if seen_from_back {
                        if citizen.is_leftsider { 9 } else { 7 }
                    } else {
                        if citizen.is_leftsider { 2 } else { 0 }
                    };
                    frame_base + citizen.walk_frame
                }
                CitizenState::Exploding(t) => {
                    if t < 0.2 { 4 } else { 5 }
                }
                CitizenState::Dead => 6,
            };

            sprites_to_draw.push(SpriteToRender {
                x: citizen.x,
                y: citizen.y,
                texture_idx: tex_idx,
            });
        }

        // 4. Render sorted sprites
        raycaster.cast_sprites(
            state.player.x,
            state.player.y,
            state.player.dir_x,
            state.player.dir_y,
            state.player.plane_x,
            state.player.plane_y,
            &sprites_to_draw,
            &assets_data,
        );

        // 5. Copy CPU pixels into GPU texture buffer
        for i in 0..(WIDTH * HEIGHT) {
            let pixel = raycaster.pixels[i];
            screen_image.bytes[i * 4]     = ((pixel >> 24) & 0xff) as u8;
            screen_image.bytes[i * 4 + 1] = ((pixel >> 16) & 0xff) as u8;
            screen_image.bytes[i * 4 + 2] = ((pixel >> 8) & 0xff) as u8;
            screen_image.bytes[i * 4 + 3] = (pixel & 0xff) as u8;
        }
        screen_texture.update(&screen_image);

        // ==========================================
        // RENDERING GPU CANVAS & HUD
        // ==========================================
        clear_background(Color::from_rgba(10, 11, 16, 255));

        let screen_w = screen_width();
        let screen_h = screen_height();

        // Scale UI based on window size
        let ui_scale = (screen_w / 800.0).max(screen_h / 600.0).max(1.0) * 1.5;

        // Sized to take up the full screen
        let view_x = 0.0;
        let view_y = 0.0;
        let view_w = screen_w;
        let view_h = screen_h;

        // Draw Raycaster screen with shake offset
        let mut shake_x = 0.0;
        let mut shake_y = 0.0;
        if state.screen_shake > 0.0 {
            // Pseudo-random shake offsets
            shake_x = ( (get_time() * 100.0).sin() as f32 ) * state.screen_shake * 15.0;
            shake_y = ( (get_time() * 120.0).cos() as f32 ) * state.screen_shake * 15.0;
        }

        draw_texture_ex(
            &screen_texture,
            view_x + shake_x,
            view_y + shake_y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(view_w, view_h)),
                ..Default::default()
            }
        );

        // Render 3D Laser beams on top of the view
        for laser in &state.lasers {
            let p_start = project_3d(laser.sx, laser.sy, 0.35, state.player.x, state.player.y, state.player.dir_x, state.player.dir_y, state.player.plane_x, state.player.plane_y, view_w, view_h);
            let p_end = project_3d(laser.ex, laser.ey, 0.35, state.player.x, state.player.y, state.player.dir_x, state.player.dir_y, state.player.plane_x, state.player.plane_y, view_w, view_h);

            if let (Some(s), Some(e)) = (p_start, p_end) {
                let color = if laser.is_player {
                    Color::new(0.0, 0.94, 1.0, 0.95) // Cyan
                } else {
                    Color::new(1.0, 0.0, 0.1, 0.95)  // Red
                };
                // Translate coordinates to match global screen viewport location
                draw_line(view_x + s.0, view_y + s.1, view_x + e.0, view_y + e.1, 5.0, color);
                draw_circle(view_x + s.0, view_y + s.1, 4.0, WHITE);
                draw_circle(view_x + e.0, view_y + e.1, 4.0, WHITE);
            }
        }

        // Render 3D Floating texts on top of the view
        for txt in &state.floating_texts {
            let proj = project_3d(txt.x, txt.y, 0.65, state.player.x, state.player.y, state.player.dir_x, state.player.dir_y, state.player.plane_x, state.player.plane_y, view_w, view_h);
            if let Some(pos) = proj {
                let r = ((txt.color >> 24) & 0xff) as u8;
                let g = ((txt.color >> 16) & 0xff) as u8;
                let b = ((txt.color >> 8) & 0xff) as u8;
                let color = Color::from_rgba(r, g, b, ( (txt.duration * 255.0).min(255.0) ) as u8);
                
                let font_size = 9.0 * ui_scale;
                let text_dim = measure_text(&txt.text, Some(&font), font_size as u16, 1.0);
                let bg_w = text_dim.width + 12.0 * ui_scale;
                let bg_h = text_dim.height + 8.0 * ui_scale;

                // Text background box
                draw_rectangle(
                    view_x + pos.0 - bg_w / 2.0,
                    view_y + pos.1 - bg_h / 2.0 - 2.0,
                    bg_w,
                    bg_h,
                    Color::from_rgba(0, 0, 0, 150),
                );
                draw_pixel_text(
                    &txt.text,
                    view_x + pos.0 - text_dim.width / 2.0,
                    view_y + pos.1 + text_dim.height / 2.0 - 4.0,
                    font_size,
                    color,
                    &font,
                );
            }
        }

        // Damage flash visual indicator
        if state.player.damage_flash > 0.0 {
            let opacity = (state.player.damage_flash * 4.0).min(0.65);
            draw_rectangle(view_x, view_y, view_w, view_h, Color::new(1.0, 0.0, 0.1, opacity));
        }

        // Shoot flash screen brighten indicator
        if let WeaponState::Firing(timer) = state.player.weapon_state {
            let opacity = (timer / 0.18) * 0.35; // up to 35% opacity
            draw_rectangle(view_x, view_y, view_w, view_h, Color::new(1.0, 1.0, 1.0, opacity));
        }

        // ==========================================
        // HUD RETICLE & CORE INTERFACES
        // ==========================================
        let cx = view_x + view_w / 2.0;
        let cy = view_y + view_h / 2.0;

        // Center holographic reticle
        let target_found = state.player.target_idx.is_some();
        let reticle_color = if target_found {
            let target = &state.citizens[state.player.target_idx.unwrap()];
            // Red for Leftsiders/rebels, green for compliant
            if target.is_leftsider || target.is_rebel {
                Color::from_rgba(255, 0, 127, 200) // Neon Pink (Criminal)
            } else {
                Color::from_rgba(57, 255, 20, 200)  // Neon Green (Compliant)
            }
        } else {
            Color::from_rgba(0, 240, 255, 150) // Cyan (Searching)
        };

        // Reticle shapes (pixel-art texture scaled up with Nearest filtering)
        let ch_size = (32.0 * ui_scale).round();
        let ch_x = (cx - ch_size / 2.0).round();
        let ch_y = (cy - ch_size / 2.0).round();

        draw_texture_ex(
            &crosshair_tex,
            ch_x,
            ch_y,
            reticle_color,
            DrawTextureParams {
                dest_size: Some(vec2(ch_size, ch_size)),
                ..Default::default()
            }
        );

        // Biometric Scanner Window (Top-Left)
        if target_found {
            let target = &state.citizens[state.player.target_idx.unwrap()];
            let is_criminal = target.is_leftsider || target.is_rebel;
            let hud_theme = if is_criminal {
                Color::from_rgba(255, 0, 127, 220) // Red/pink theme
            } else {
                Color::from_rgba(57, 255, 20, 220)  // Green theme
            };

            let font_size = 4.0 * ui_scale; // Half font size as requested

            // Define lines of text to draw
            let line1 = "BIOMETRIC SCAN ACQUIRED";
            let line2 = format!("NAME: {}", target.name);
            let line3 = format!("REG : {}", target.id_num);
            
            let tile = state.map.get_tile(target.x, target.y);
            let location_str = match tile {
                TileType::SidewalkVert => "SIDEWALK (VERT)",
                TileType::SidewalkHoriz => "SIDEWALK (HORIZ)",
                TileType::Intersection => "INTERSECTION",
                TileType::Road => "STREET ROADWAY",
                _ => "UNKNOWN ZONE",
            };
            let line4 = format!("LOC : {}", location_str);

            let line5 = if is_criminal {
                if target.is_rebel {
                    "STATUS: REBEL / SHOOT TO KILL"
                } else {
                    "STATUS: LEFT-SIDE VIOLATION"
                }
            } else {
                "STATUS: COMPLIANT CITIZEN"
            };

            // Measure longest line to dynamically size width
            let d1 = measure_text(line1, Some(&font), font_size as u16, 1.0);
            let d2 = measure_text(&line2, Some(&font), font_size as u16, 1.0);
            let d3 = measure_text(&line3, Some(&font), font_size as u16, 1.0);
            let d4 = measure_text(&line4, Some(&font), font_size as u16, 1.0);
            let d5 = measure_text(line5, Some(&font), font_size as u16, 1.0);

            let max_w = d1.width
                .max(d2.width)
                .max(d3.width)
                .max(d4.width)
                .max(d5.width);

            // Window border (tighter fit for half font size)
            let wx = view_x + 15.0 * ui_scale;
            let wy = view_y + 15.0 * ui_scale;
            let win_w = max_w + 16.0 * ui_scale;
            let win_h = 56.0 * ui_scale;

            draw_rectangle(wx, wy, win_w, win_h, Color::from_rgba(10, 15, 25, 200));
            draw_pixel_rect_lines(wx, wy, win_w, win_h, 2.0 * ui_scale, hud_theme);

            // Scanner Details Text (adjusted offsets)
            let padding_x = 8.0 * ui_scale;
            let line_y = 9.0 * ui_scale;
            draw_pixel_text(line1, wx + padding_x, wy + 10.0 * ui_scale, font_size, hud_theme, &font);
            draw_pixel_text(&line2, wx + padding_x, wy + 10.0 * ui_scale + line_y, font_size, WHITE, &font);
            draw_pixel_text(&line3, wx + padding_x, wy + 10.0 * ui_scale + line_y * 2.0, font_size, Color::from_rgba(180, 200, 220, 255), &font);
            draw_pixel_text(&line4, wx + padding_x, wy + 10.0 * ui_scale + line_y * 3.0, font_size, Color::from_rgba(180, 200, 220, 255), &font);
            draw_pixel_text(line5, wx + padding_x, wy + 10.0 * ui_scale + line_y * 4.0, font_size, hud_theme, &font);
        }

        // Firing logs / Compliance banner (Top-Center)
        if let Some((ref text, color, _)) = state.credits_flash {
            let r = ((color >> 24) & 0xff) as u8;
            let g = ((color >> 16) & 0xff) as u8;
            let b = ((color >> 8) & 0xff) as u8;
            let flash_c = Color::from_rgba(r, g, b, 255);

            let font_size = 8.0 * ui_scale;
            let text_dim = measure_text(text, Some(&font), font_size as u16, 1.0);
            let banner_w = text_dim.width + 30.0 * ui_scale;
            let banner_h = text_dim.height + 16.0 * ui_scale;
            let bx = view_x + (view_w - banner_w) / 2.0;
            let by = view_y + 20.0 * ui_scale;

            draw_rectangle(bx, by, banner_w, banner_h, Color::from_rgba(0, 0, 0, 220));
            draw_pixel_rect_lines(bx, by, banner_w, banner_h, 2.0 * ui_scale, flash_c);
            draw_pixel_text(
                text,
                bx + 15.0 * ui_scale,
                by + banner_h / 2.0 + text_dim.height / 2.0 - 2.0,
                font_size,
                flash_c,
                &font,
            );
        }

        // (Mini-map interface removed as requested)

        // ==========================================
        // PLAYER BUDGET DISPLAY (Bottom Left Panel)
        // ==========================================
        let font_value_size = 8.0 * ui_scale;

        let val_str = format!("{} CR", state.player.credits);
        let val_dim = measure_text(&val_str, Some(&font), (font_value_size * 1.3) as u16, 1.0);

        // Size the panel dynamically based on content width (tightly fit)
        let panel_w = (val_dim.width + 16.0 * ui_scale).round();
        let panel_h = (val_dim.height + 12.0 * ui_scale).round();

        let px = (view_x + 15.0 * ui_scale).round();
        let py = (view_y + view_h - panel_h - 15.0 * ui_scale).round();

        draw_rectangle(px, py, panel_w, panel_h, Color::from_rgba(10, 15, 25, 220));
        draw_pixel_rect_lines(px, py, panel_w, panel_h, 2.0 * ui_scale, Color::from_rgba(0, 240, 255, 180));

        let credits_col = if state.player.credits < 0 {
            Color::from_rgba(255, 0, 127, 255) // Red negative budget
        } else {
            Color::from_rgba(57, 255, 20, 255)  // Green positive budget
        };

        draw_pixel_text(&val_str, px + 8.0 * ui_scale, py + val_dim.height + 5.0 * ui_scale, font_value_size * 1.3, credits_col, &font);

        // Weapon sprite rendering removed as requested

        // ==========================================
        // GAME OVER / SIMULATION FAIL OVERLAY
        // ==========================================
        if is_game_over {
            draw_rectangle(view_x, view_y, view_w, view_h, Color::from_rgba(20, 0, 5, 220));

            let size1 = 12.0 * ui_scale;
            let size2 = 8.0 * ui_scale;
            let size3 = 9.0 * ui_scale;

            let t1 = "REU-99 INTEGRITY CRITICAL // SIM TERMINATED";
            let t2 = "REBEL UNIT DEPLOYED LETHAL FORCE";
            let t3 = "PRESS 'R' TO REBOOT SYSTEM AND TRY AGAIN";

            let dim1 = measure_text(t1, Some(&font), size1 as u16, 1.0);
            let dim2 = measure_text(t2, Some(&font), size2 as u16, 1.0);
            let dim3 = measure_text(t3, Some(&font), size3 as u16, 1.0);

            draw_pixel_text(t1, view_x + (view_w - dim1.width) / 2.0, view_y + view_h * 0.4, size1, Color::from_rgba(255, 0, 127, 255), &font);
            draw_pixel_text(t2, view_x + (view_w - dim2.width) / 2.0, view_y + view_h * 0.48, size2, WHITE, &font);
            draw_pixel_text(t3, view_x + (view_w - dim3.width) / 2.0, view_y + view_h * 0.6, size3, Color::from_rgba(0, 240, 255, 255), &font);
        } else if is_bankrupt {
            draw_rectangle(view_x, view_y, view_w, view_h, Color::from_rgba(20, 0, 5, 220));

            let size1 = 11.0 * ui_scale;
            let size2 = 7.0 * ui_scale;
            let size3 = 9.0 * ui_scale;

            let t1 = "BUDGET BALANCE LIMIT EXCEEDED // DECOMMISSIONED";
            let t2 = "COLLATERAL DAMAGE LIABILITIES SURPASSED POLICE FUNDS";
            let t3 = "PRESS 'R' TO REBOOT SYSTEM AND TRY AGAIN";

            let dim1 = measure_text(t1, Some(&font), size1 as u16, 1.0);
            let dim2 = measure_text(t2, Some(&font), size2 as u16, 1.0);
            let dim3 = measure_text(t3, Some(&font), size3 as u16, 1.0);

            draw_pixel_text(t1, view_x + (view_w - dim1.width) / 2.0, view_y + view_h * 0.4, size1, Color::from_rgba(255, 0, 127, 255), &font);
            draw_pixel_text(t2, view_x + (view_w - dim2.width) / 2.0, view_y + view_h * 0.48, size2, WHITE, &font);
            draw_pixel_text(t3, view_x + (view_w - dim3.width) / 2.0, view_y + view_h * 0.6, size3, Color::from_rgba(0, 240, 255, 255), &font);
        }

        next_frame().await;
    }
}
