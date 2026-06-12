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

#[allow(unused)]
fn hsl_to_rgb(h: f32, s: f32, l: f32) -> Color {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;
    
    let (r, g, b) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    
    Color::new(r + m, g + m, b + m, 1.0)
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
// Corners are recessed to make the box appear rounded and avoid overdrawing transparent corner pixels.
fn draw_pixel_rect_lines(x: f32, y: f32, w: f32, h: f32, thickness: f32, color: Color) {
    let x = x.round();
    let y = y.round();
    let w = w.round();
    let h = h.round();
    let t = thickness.round().max(1.0);

    // Top border (shortened by t on both ends)
    draw_rectangle(x + t, y, w - 2.0 * t, t, color);
    // Bottom border (shortened by t on both ends)
    draw_rectangle(x + t, y + h - t, w - 2.0 * t, t, color);
    // Left border (shortened by t on both ends)
    draw_rectangle(x, y + t, t, h - 2.0 * t, color);
    // Right border (shortened by t on both ends)
    draw_rectangle(x + w - t, y + t, t, h - 2.0 * t, color);
}

// Helper to draw a pixel-art double note icon
fn draw_music_note_icon(x: f32, y: f32, size: f32, color: Color) {
    let u = size / 8.0;
    // Left note head
    draw_rectangle(x + 1.0 * u, y + 5.0 * u, 2.0 * u, 2.0 * u, color);
    // Right note head
    draw_rectangle(x + 5.0 * u, y + 4.0 * u, 2.0 * u, 2.0 * u, color);
    // Left stem
    draw_rectangle(x + 2.0 * u, y + 1.0 * u, 1.0 * u, 4.0 * u, color);
    // Right stem
    draw_rectangle(x + 6.0 * u, y + 0.0 * u, 1.0 * u, 4.0 * u, color);
    // Beam (slanted)
    draw_rectangle(x + 2.0 * u, y + 1.0 * u, 3.0 * u, 1.0 * u, color);
    draw_rectangle(x + 5.0 * u, y + 0.0 * u, 2.0 * u, 1.0 * u, color);
}

// Helper to draw a retro hollow square fullscreen icon (sharp rectangle outline)
fn draw_fullscreen_icon(x: f32, y: f32, size: f32, color: Color) {
    let x = x.round();
    let y = y.round();
    let size = size.round();
    let t = (size * 0.15).round().max(1.0);

    // Top
    draw_rectangle(x, y, size, t, color);
    // Bottom
    draw_rectangle(x, y + size - t, size, t, color);
    // Left (shortened to avoid overlapping top/bottom)
    draw_rectangle(x, y + t, t, size - 2.0 * t, color);
    // Right (shortened to avoid overlapping top/bottom)
    draw_rectangle(x + size - t, y + t, t, size - 2.0 * t, color);
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

    // Create RenderTarget for off-screen low-res rendering
    let render_target = render_target_ex(
        WIDTH as u32,
        HEIGHT as u32,
        RenderTargetParams {
            sample_count: 0,
            depth: false,
        },
    );
    render_target.texture.set_filter(FilterMode::Nearest);

    // Font selection (Default fallback, style elements drawn with rectangles/lines)
    let mut is_game_over = false;
    let mut is_bankrupt = false;



    loop {
        // Frame delta time (capped to prevent physics explosions on lag)
        let dt = get_frame_time().min(0.08);

        let screen_w = screen_width();
        let screen_h = screen_height();

        let virtual_w = WIDTH as f32;
        let virtual_h = HEIGHT as f32;

        // Scale UI based on screen resolution for high-res sharpness
        let ui_scale = (screen_w / 800.0).max(screen_h / 600.0).max(1.0) * 1.5;

        // Screen shake offsets (high-resolution space)
        let mut shake_x = 0.0;
        let mut shake_y = 0.0;
        if state.screen_shake > 0.0 {
            shake_x = ( (get_time() * 120.0).sin() as f32 ) * state.screen_shake * 25.0 * ui_scale;
            shake_y = ( (get_time() * 140.0).cos() as f32 ) * state.screen_shake * 25.0 * ui_scale;
        }

        let view_x = shake_x;
        let view_y = shake_y;
        let view_w = screen_w;
        let view_h = screen_h;

        let cx = view_x + screen_w / 2.0;

        // Mouse position in screen space
        let (mx, my) = mouse_position();

        // Button dimensions and positions for input and rendering (smaller size)
        let btn_font_size = 7.5 * ui_scale;
        let play_text = "ENFORCE CIVIC DIRECTIVES";
        let highscore_text = "UNIT LEADERBOARD";
        let level_text = "SECTOR SELECTION";

        let play_dim = measure_text(play_text, Some(&font), btn_font_size as u16, 1.0);
        let highscore_dim = measure_text(highscore_text, Some(&font), btn_font_size as u16, 1.0);
        let level_dim = measure_text(level_text, Some(&font), btn_font_size as u16, 1.0);

        let max_btn_w = play_dim.width.max(highscore_dim.width).max(level_dim.width) + 20.0 * ui_scale;
        let btn_h = 22.0 * ui_scale;

        // Button positions: stack with a fixed gap so they never overlap at any resolution
        let btn_gap = 6.0 * ui_scale;
        let p_bx = cx - max_btn_w / 2.0;
        let p_by = view_y + view_h * 0.43 - btn_h / 2.0;

        // Button 2 (Highscore) position
        let h_bx = cx - max_btn_w / 2.0;
        let h_by = p_by + btn_h + btn_gap;

        // Button 3 (Level Select) position
        let l_bx = cx - max_btn_w / 2.0;
        let l_by = h_by + btn_h + btn_gap;

        // ==========================================
        // INPUT PROCESSING
        // ==========================================
        let mut switch_lane_left = false;
        let mut switch_lane_right = false;

        if state.show_leaderboard {
            let trigger = is_key_pressed(KeyCode::R) || 
                          is_mouse_button_pressed(MouseButton::Left) || 
                          (game::is_mobile() && game::js_get_trigger_fire());
            if trigger {
                if is_game_over || is_bankrupt {
                    state = GameState::new();
                    is_game_over = false;
                    is_bankrupt = false;
                } else {
                    // Return to menu from leaderboard
                    state.show_leaderboard = false;
                    state.is_in_menu = true;
                    state.menu_timer = 0.0;
                    state.slogan_chars_played = 0;
                    state.menu_title_landed = false;
                    state.menu_star_played = false;
                    state.menu_particles.clear();
                    state.menu_shockwaves.clear();
                    game::update_menu_active_js(true);
                    game::play_sound("laser");
                }
            }
        } else if state.is_showing_summary {
            // Summary Screen Input - block click and R/Space/Enter interactions for 0.5 seconds
            if state.summary_timer >= 0.5 {
                let trigger = is_key_pressed(KeyCode::R) || is_key_pressed(KeyCode::Space) || 
                              is_key_pressed(KeyCode::Enter) || is_mouse_button_pressed(MouseButton::Left) ||
                              (game::is_mobile() && game::js_get_trigger_fire());
                if trigger {
                    if state.summary_stage < 4 {
                        state.summary_skip_buildup = true;
                        state.summary_stage = 4;
                        game::play_sound("menu_pling");
                    } else {
                        // Transition to highscore entry (if qualified) or directly to leaderboard (if not)
                        is_game_over = state.time_left <= 0.0;
                        is_bankrupt = false;
                        
                        let qualifies_for_leaderboard = {
                            let scores = state.load_leaderboard_rust();
                            if scores.len() < 10 {
                                true
                            } else if let Some(lowest) = scores.last() {
                                state.player.credits > lowest.1
                            } else {
                                true
                            }
                        };

                        if qualifies_for_leaderboard && !is_bankrupt {
                            state.is_entering_highscore = true;
                            state.highscore_name = String::new();
                            state.highscore_input_delay = 0.5;
                            game::set_entering_highscore(true);
                            state.is_showing_summary = false;
                            game::play_sound("laser");
                        } else {
                            state.leaderboard_data = state.load_leaderboard_rust();
                            state.new_rank = None;
                            state.is_entering_highscore = false;
                            state.show_leaderboard = true;
                            game::set_entering_highscore(false);
                            state.is_showing_summary = false;
                            game::play_sound("laser");
                        }
                        
                        while let Some(_) = get_char_pressed() {}
                    }
                }
            } else {
                // Drain any inputs during lockout to avoid buffer spillover
                while let Some(_) = get_char_pressed() {}
            }
        } else if !is_game_over && !is_bankrupt {
            if state.is_in_menu {
                // Input lockout of 1.17 seconds to let title land and buttons start appearing
                if state.menu_timer >= 1.17 {
                    // Keyboard Navigation
                    let mut select_changed = false;
                    if is_key_pressed(KeyCode::W) || is_key_pressed(KeyCode::Up) {
                        state.menu_selected_idx = if state.menu_selected_idx == 0 { 2 } else { state.menu_selected_idx - 1 };
                        select_changed = true;
                    }
                    if is_key_pressed(KeyCode::S) || is_key_pressed(KeyCode::Down) {
                        state.menu_selected_idx = if state.menu_selected_idx == 2 { 0 } else { state.menu_selected_idx + 1 };
                        select_changed = true;
                    }
                    if select_changed {
                        game::play_sound("laser"); // tick sound
                    }

                    // Mouse Hover Navigation
                    let hover_play = mx >= p_bx && mx <= p_bx + max_btn_w && my >= p_by && my <= p_by + btn_h;
                    let hover_highscore = mx >= h_bx && mx <= h_bx + max_btn_w && my >= h_by && my <= h_by + btn_h;
                    let hover_level = mx >= l_bx && mx <= l_bx + max_btn_w && my >= l_by && my <= l_by + btn_h;

                    let bottom_btn_size = 24.0 * ui_scale;
                    let margin_left = 20.0 * ui_scale;
                    let margin_bottom = 20.0 * ui_scale;
                    let margin_right = 20.0 * ui_scale;
                    let info_bx = view_x + margin_left;
                    let info_by = view_y + view_h - margin_bottom - bottom_btn_size;
                    let music_bx = info_bx + bottom_btn_size + 10.0 * ui_scale;
                    let music_by = info_by;

                    let fs_bx = view_x + view_w - margin_right - bottom_btn_size;
                    let fs_by = info_by;
                    let help_bx = fs_bx - bottom_btn_size - 10.0 * ui_scale;
                    let help_by = info_by;

                    let hover_info = mx >= info_bx && mx <= info_bx + bottom_btn_size && my >= info_by && my <= info_by + bottom_btn_size;
                    let hover_music = mx >= music_bx && mx <= music_bx + bottom_btn_size && my >= music_by && my <= music_by + bottom_btn_size;
                    let hover_help = mx >= help_bx && mx <= help_bx + bottom_btn_size && my >= help_by && my <= help_by + bottom_btn_size;
                    let hover_fs = mx >= fs_bx && mx <= fs_bx + bottom_btn_size && my >= fs_by && my <= fs_by + bottom_btn_size;

                    if hover_play {
                        if state.menu_selected_idx != 0 {
                            state.menu_selected_idx = 0;
                            game::play_sound("laser");
                        }
                    } else if hover_highscore {
                        if state.menu_selected_idx != 1 {
                            state.menu_selected_idx = 1;
                            game::play_sound("laser");
                        }
                    } else if hover_level {
                        if state.menu_selected_idx != 2 {
                            state.menu_selected_idx = 2;
                            game::play_sound("laser");
                        }
                    }

                    // Trigger Selection
                    let mouse_clicked = is_mouse_button_pressed(MouseButton::Left);
                    let trigger_select = is_key_pressed(KeyCode::Space) || is_key_pressed(KeyCode::Enter) || mouse_clicked;

                    if trigger_select {
                        if mouse_clicked && hover_info {
                            game::play_sound("laser");
                            game::open_privacy_modal();
                        } else if mouse_clicked && hover_music {
                            game::play_sound("laser");
                            game::open_music_modal();
                        } else if mouse_clicked && hover_help {
                            game::play_sound("laser");
                            game::toggle_help();
                        } else if mouse_clicked && hover_fs {
                            game::play_sound("laser");
                            game::toggle_fullscreen();
                        } else {
                            let mut select_btn = false;
                            if hover_play {
                                state.menu_selected_idx = 0;
                                select_btn = true;
                            } else if hover_highscore {
                                state.menu_selected_idx = 1;
                                select_btn = true;
                            } else if hover_level {
                                state.menu_selected_idx = 2;
                                select_btn = true;
                            }

                            // Only proceed if it was a keyboard trigger or we clicked a valid button
                            if !mouse_clicked || select_btn {
                                if state.menu_selected_idx == 0 {
                                    state.is_in_menu = false;
                                    state.menu_timer = 0.0; // reset menu timer to 0 on exit
                                    state.menu_shockwaves.clear();
                                    game::update_menu_active_js(false);
                                    game::play_sound("explosion"); // Boom play start sound
                                } else if state.menu_selected_idx == 1 {
                                    state.leaderboard_data = state.load_leaderboard_rust();
                                    state.is_entering_highscore = false;
                                    state.show_leaderboard = true;
                                    state.is_in_menu = false; // deactivate menu to show leaderboard overlay
                                    state.menu_timer = 0.0; // reset menu timer to 0 on exit
                                    state.menu_shockwaves.clear();
                                    game::play_sound("explosion");
                                } else {
                                    game::play_sound("collateral"); // Error buzz
                                }
                            }
                        }
                    }
                }
            } else {
                if game::is_mobile() {
                    // Mobile controls via JS touch listeners
                    if game::js_get_switch_lane_left() {
                        switch_lane_left = true;
                    }
                    if game::js_get_switch_lane_right() {
                        switch_lane_right = true;
                    }
                    if game::js_get_trigger_fire() {
                        state.trigger_fire();
                    }
                    if game::js_get_trigger_missile() {
                        state.trigger_missile_salvo();
                    }
                } else {
                    // Desktop keyboard controls
                    if is_key_pressed(KeyCode::A) {
                        switch_lane_left = true;
                    }
                    if is_key_pressed(KeyCode::D) {
                        switch_lane_right = true;
                    }

                    // Desktop mouse click controls
                    if is_mouse_button_pressed(MouseButton::Left) {
                        let font_value_size = 8.0 * ui_scale;
                        let val_str = format!("{} CR", state.player.credits);
                        let val_dim = measure_text(&val_str, Some(&font), (font_value_size * 1.3) as u16, 1.0);
                        let panel_h = (val_dim.height + 12.0 * ui_scale).round();
                        
                        let px_val = (view_x + 15.0 * ui_scale).round();
                        let py_val = (view_y + view_h - panel_h - 15.0 * ui_scale).round();

                        let rx_val = px_val + 4.0 * ui_scale;
                        let ry_val = py_val - 6.0 * ui_scale;
                        let rocket_dim = measure_text("[R]OCKET", Some(&font), (font_value_size * 1.1) as u16, 1.0);

                        let (mx, my) = mouse_position();
                        let rocket_touched = !state.missile_used
                            && mx >= (rx_val - 15.0 * ui_scale)
                            && mx <= (rx_val + rocket_dim.width + 15.0 * ui_scale)
                            && my >= (ry_val - rocket_dim.height - 15.0 * ui_scale)
                            && my <= (ry_val + 15.0 * ui_scale);

                        if rocket_touched {
                            state.trigger_missile_salvo();
                        } else {
                            state.trigger_fire();
                        }
                    }
                }

                // Calculate speed multiplier based on elapsed time (start at 0, scale to 1 in 1s, then linearly to 2.5x by 30s)
                let elapsed = (30.0 - state.time_left).max(0.0);
                let multiplier = if elapsed < 1.0 {
                    elapsed
                } else {
                    1.0 + (elapsed - 1.0) / 29.0 * 1.5
                };

                // Player base speed is constant 3.0
                let base_speed = 3.0;
                state.player.speed = base_speed * multiplier;

                // Shooting: Spacebar only
                if is_key_pressed(KeyCode::Space) {
                    state.trigger_fire();
                }

                // Guided missile salvo: R key
                if is_key_pressed(KeyCode::R) {
                    state.trigger_missile_salvo();
                }
            }
        }

        // ==========================================
        // GAME STATE UPDATE
        // ==========================================
        if !is_game_over && !is_bankrupt && !state.show_leaderboard {
            if !state.is_showing_summary {
                state.move_player(switch_lane_left, switch_lane_right);
                state.update(dt);

                // Tick countdown
                if !state.is_in_menu {
                    state.time_left -= dt;
                    if state.time_left <= 0.0 {
                        state.time_left = 0.0;
                    }

                    // Countdown beeps for the last 5 seconds (pitched by remaining seconds)
                    let current_sec = state.time_left.ceil() as i32;
                    if current_sec <= 5 && current_sec > 0 && current_sec < state.last_beep_second {
                        state.last_beep_second = current_sec;
                        let beep_name = format!("countdown_beep_{}", current_sec);
                        game::play_sound(&beep_name);
                    }
                }

                // Check loss conditions (trigger summary debrief instead of immediate game over)
                let should_end = !state.is_in_menu && state.time_left <= 0.0;
                if should_end {
                    state.is_showing_summary = true;
                    state.summary_timer = 0.0;
                    state.summary_stage = 0;
                    state.summary_count_anim = 0.0;
                    state.summary_skip_buildup = false;
                    state.menu_particles.clear();
                    state.menu_shockwaves.clear();
                    game::play_sound("time_over");

                    // Drain any buffered characters pressed during gameplay
                    while let Some(_) = get_char_pressed() {}
                }
            } else {
                // Update summary screen animations
                let summary_dt = get_frame_time().min(0.08);
                state.summary_timer += summary_dt;

                // Decay screen shake
                if state.screen_shake > 0.0 {
                    state.screen_shake -= summary_dt * 4.0;
                }

                // Update menu particles & shockwaves during summary screen
                {
                    let dt = summary_dt;
                    for p in state.menu_particles.iter_mut() {
                        p.lifetime += dt;
                        p.x += p.vx * dt;
                        p.y += p.vy * dt;
                        p.vy += 120.0 * ui_scale * dt; // gravity
                        p.vx *= 0.98; // drag
                    }
                    state.menu_particles.retain(|p| p.lifetime < p.max_lifetime);

                    for sw in state.menu_shockwaves.iter_mut() {
                        sw.lifetime += dt;
                        sw.radius += sw.speed * dt;
                    }
                    state.menu_shockwaves.retain(|sw| sw.lifetime < sw.max_lifetime);
                }

                if !state.summary_skip_buildup {
                    if state.summary_timer >= 0.5 {
                        let elapsed = state.summary_timer - 0.5;
                        let new_stage = (elapsed / 0.75) as usize;

                        if new_stage > state.summary_stage && state.summary_stage < 4 {
                            // Spawn landing/credits explosion at the center of the finished stage's credits text!
                            let target_val = match state.summary_stage {
                                0 => state.offenders_killed_laser as f32 * 1000.0,
                                1 => state.offenders_killed_rocket as f32 * 750.0,
                                2 => state.collateral_damage_kills as f32 * -1250.0,
                                _ => state.player.credits as f32,
                            };

                            if target_val != 0.0 {
                                 let panel_w = (440.0 * ui_scale).min(view_w - 30.0 * ui_scale);
                                 let panel_h = 180.0 * ui_scale;
                                 let panel_x = cx - panel_w / 2.0;
                                 let panel_y = (view_y + (view_h - panel_h) / 2.0 - 15.0 * ui_scale).max(5.0 * ui_scale);
                                let credit_x = panel_x + panel_w - 30.0 * ui_scale;
                                let start_y = panel_y + 75.0 * ui_scale;
                                let row_gap = 25.0 * ui_scale;
                                let size_row = 9.0 * ui_scale;

                                let current_y = start_y + state.summary_stage as f32 * row_gap;
                                let credits_str = match state.summary_stage {
                                    0 => format!("{:+} CR", state.offenders_killed_laser * 1000),
                                    1 => format!("{:+} CR", state.offenders_killed_rocket * 750),
                                    2 => format!("{:+} CR", state.collateral_damage_kills as i32 * -1250),
                                    _ => format!("{} CR", state.player.credits),
                                };
                                let credits_dim = measure_text(&credits_str, Some(&font), size_row as u16, 1.0);
                                let ex = credit_x - credits_dim.width / 2.0;
                                let ey = current_y + credits_dim.height / 2.0;

                                game::play_sound("menu_explosion");
                                state.screen_shake = 0.6;

                                // Spawn burst of particles at the credits text center
                                let num_particles = 80;
                                let (r, g, b) = if target_val > 0.0 {
                                    (57u8, 255u8, 20u8)   // Neon Green
                                } else {
                                    (255u8, 0u8, 127u8)   // Neon Pink
                                };

                                for i in 0..num_particles {
                                    let angle = (i as f32 / num_particles as f32) * std::f32::consts::TAU
                                        + (i as f32 * 2.399);
                                    let speed = 80.0 * ui_scale + (i as f32 % 6.0) * 40.0 * ui_scale;
                                    let vx = angle.cos() * speed;
                                    let vy = angle.sin() * speed;

                                    state.menu_particles.push(crate::game::MenuParticle {
                                        x: ex,
                                        y: ey,
                                        vx,
                                        vy,
                                        lifetime: 0.0,
                                        max_lifetime: 0.5 + (i as f32 % 4.0) * 0.12,
                                        size: 2.0 * ui_scale + (i as f32 % 4.0) * ui_scale,
                                        color_r: r,
                                        color_g: g,
                                        color_b: b,
                                    });
                                }

                                // Spawn 2 shockwaves in the same color
                                state.menu_shockwaves.push(crate::game::MenuShockwave {
                                    x: ex,
                                    y: ey,
                                    radius: 0.0,
                                    max_radius: 120.0 * ui_scale,
                                    speed: 350.0 * ui_scale,
                                    lifetime: 0.0,
                                    max_lifetime: 0.4,
                                    thickness: 5.0 * ui_scale,
                                    color_r: r,
                                    color_g: g,
                                    color_b: b,
                                });
                                state.menu_shockwaves.push(crate::game::MenuShockwave {
                                    x: ex,
                                    y: ey,
                                    radius: 0.0,
                                    max_radius: 90.0 * ui_scale,
                                    speed: 280.0 * ui_scale,
                                    lifetime: 0.0,
                                    max_lifetime: 0.4,
                                    thickness: 3.0 * ui_scale,
                                    color_r: r,
                                    color_g: g,
                                    color_b: b,
                                });
                            }

                            game::play_sound("menu_pling");
                            state.summary_stage = new_stage.min(4);
                            state.summary_count_anim = 0.0;
                        }

                        if state.summary_stage < 4 {
                            let target_val = match state.summary_stage {
                                0 => state.offenders_killed_laser as f32 * 1000.0,
                                1 => state.offenders_killed_rocket as f32 * 750.0,
                                2 => state.collateral_damage_kills as f32 * -1250.0,
                                _ => state.player.credits as f32,
                            };

                            let stage_elapsed = elapsed - (state.summary_stage as f32 * 0.75);
                            let progress = (stage_elapsed / 0.75).min(1.0).max(0.0);
                            let prev_anim = state.summary_count_anim;
                            state.summary_count_anim = target_val * progress;

                            // Play click sounds on change
                            if state.summary_count_anim != prev_anim {
                                let c_abs = target_val.abs() as i32;
                                let divisor = (c_abs / 20).max(25);
                                if (state.summary_count_anim as i32 / divisor) != (prev_anim as i32 / divisor) {
                                    game::play_sound("scan_tick");
                                }
                            }
                        }
                    }
                }
            }
        } else {
            if state.is_entering_highscore {
                if state.highscore_input_delay > 0.0 {
                    state.highscore_input_delay -= dt;
                    // Drain any key presses while blocked
                    while let Some(_) = get_char_pressed() {}
                } else {
                    if is_bankrupt {
                        // Skip name input, press R to continue
                        let trigger_confirm = is_key_pressed(KeyCode::R) || 
                            (touches().is_empty() == false) ||
                            is_mouse_button_pressed(MouseButton::Left) ||
                            (game::is_mobile() && game::js_get_trigger_fire());
                        if trigger_confirm {
                            state.leaderboard_data = state.load_leaderboard_rust();
                            state.new_rank = None;
                            state.is_entering_highscore = false;
                            state.show_leaderboard = true;
                            game::set_entering_highscore(false);
                            game::play_sound("rank_fail");
                        }
                    } else {
                        // Poll mobile highscore initials from JS
                        let mut mobile_name_buf = [0u8; 16];
                        let mobile_name_len = game::get_mobile_highscore_name(&mut mobile_name_buf);
                        if mobile_name_len > 0 {
                            if let Ok(name_str) = std::str::from_utf8(&mobile_name_buf[..mobile_name_len]) {
                                state.highscore_name = name_str.trim().to_ascii_uppercase();
                            }
                        }

                        // Check if mobile submit was triggered
                        let mobile_submit = game::is_mobile_highscore_submitted();
                        if mobile_submit {
                            game::clear_mobile_highscore_submit();
                        }

                        while let Some(c) = get_char_pressed() {
                            if c.is_ascii_alphabetic() && state.highscore_name.len() < 3 {
                                state.highscore_name.push(c.to_ascii_uppercase());
                                game::play_sound("laser");
                            }
                        }
                        if is_key_pressed(KeyCode::Backspace) {
                            state.highscore_name.pop();
                            game::play_sound("laser");
                        }

                        let submit_triggered = (is_key_pressed(KeyCode::R) && state.highscore_name.len() == 3) || (mobile_submit && state.highscore_name.len() == 3);
                        if submit_triggered {
                            state.save_highscore_rust(&state.highscore_name, state.player.credits);
                            state.leaderboard_data = state.load_leaderboard_rust();
                            state.new_rank = None;
                            for (idx, (n, s)) in state.leaderboard_data.iter().enumerate() {
                                if n == &state.highscore_name && *s == state.player.credits {
                                    state.new_rank = Some(idx);
                                    break;
                                }
                            }
                            state.is_entering_highscore = false;
                            state.show_leaderboard = true;
                            game::set_entering_highscore(false);
                            if state.new_rank.is_some() {
                                game::play_sound("rank_top10");
                            } else {
                                game::play_sound("rank_fail");
                            }
                        }
                    }
                }
            } else if state.show_leaderboard {
                // Update particles and shockwaves on highscore screen
                let dt = get_frame_time().min(0.08);
                for p in state.menu_particles.iter_mut() {
                    p.lifetime += dt;
                    p.x += p.vx * dt;
                    p.y += p.vy * dt;
                    p.vy += 120.0 * ui_scale * dt; // gravity
                    p.vx *= 0.98; // drag
                }
                state.menu_particles.retain(|p| p.lifetime < p.max_lifetime);

                for sw in state.menu_shockwaves.iter_mut() {
                    sw.lifetime += dt;
                    sw.radius += sw.speed * dt;
                }
                state.menu_shockwaves.retain(|sw| sw.lifetime < sw.max_lifetime);

                // Spawn fireworks if they reached the top 10!
                if state.new_rank.is_some() {
                    state.summary_timer += dt;

                    // Use state.summary_count_anim to store the current random delay target (0.2 to 0.5s)
                    let mut current_target_delay = state.summary_count_anim;
                    if current_target_delay < 0.2 || current_target_delay > 0.5 {
                        current_target_delay = 0.2 + state.random_float() * 0.3;
                        state.summary_count_anim = current_target_delay;
                    }

                    if state.summary_timer >= current_target_delay {
                        state.summary_timer = 0.0;

                        // Determine next random delay for the subsequent firework
                        let next_delay = 0.2 + state.random_float() * 0.3;
                        state.summary_count_anim = next_delay;

                        // Pseudo-random position:
                        let rx_val = state.random_float();
                        let ry_val = state.random_float();

                        let ex = view_x + 0.1 * view_w + rx_val * (view_w * 0.8);
                        let ey = view_y + 0.1 * view_h + ry_val * (view_h * 0.5); // upper 60% of screen

                        // Play fireworks sound!
                        game::play_sound("firework");

                        // Pick one specific color for this firework: yellow, green, pink, or cyan
                        let color_type = (state.next_random() >> 16) % 4;
                        let (r, g, b) = match color_type {
                            0 => (255u8, 235u8, 59u8),  // Yellow
                            1 => (57u8, 255u8, 20u8),   // Green
                            2 => (255u8, 0u8, 127u8),   // Pink
                            _ => (0u8, 240u8, 255u8),   // Cyan
                        };

                        // Spawn particles
                        let num_particles = 80;
                        for i in 0..num_particles {
                            let angle = (i as f32 / num_particles as f32) * std::f32::consts::TAU
                                + (i as f32 * 2.399);
                            let speed = 70.0 * ui_scale + (i as f32 % 5.0) * 35.0 * ui_scale;
                            let vx = angle.cos() * speed;
                            let vy = angle.sin() * speed;

                            state.menu_particles.push(crate::game::MenuParticle {
                                x: ex,
                                y: ey,
                                vx,
                                vy,
                                lifetime: 0.0,
                                max_lifetime: 0.6 + (i as f32 % 4.0) * 0.15,
                                size: 2.0 * ui_scale + (i as f32 % 4.0) * ui_scale,
                                color_r: r,
                                color_g: g,
                                color_b: b,
                            });
                        }

                        // Spawn 2 expanding shockwaves in the same color
                        state.menu_shockwaves.push(crate::game::MenuShockwave {
                            x: ex,
                            y: ey,
                            radius: 0.0,
                            max_radius: 110.0 * ui_scale,
                            speed: 320.0 * ui_scale,
                            lifetime: 0.0,
                            max_lifetime: 0.45,
                            thickness: 4.0 * ui_scale,
                            color_r: r,
                            color_g: g,
                            color_b: b,
                        });
                        state.menu_shockwaves.push(crate::game::MenuShockwave {
                            x: ex,
                            y: ey,
                            radius: 0.0,
                            max_radius: 80.0 * ui_scale,
                            speed: 250.0 * ui_scale,
                            lifetime: 0.0,
                            max_lifetime: 0.45,
                            thickness: 2.5 * ui_scale,
                            color_r: r,
                            color_g: g,
                            color_b: b,
                        });
                    }
                }
            }
        }

        // ==========================================
        // 3D RAYCASTING STEP
        // ==========================================
        raycaster.clear();
        
        // Filter decals close to player for performance (only within visibility range + buffer)
        let close_decals: Vec<crate::game::BloodDecal> = state.decals.iter()
            .filter(|decal| {
                let mut dx = decal.x - state.player.x;
                if dx > MAP_WIDTH as f32 / 2.0 { dx -= MAP_WIDTH as f32; }
                else if dx < -(MAP_WIDTH as f32 / 2.0) { dx += MAP_WIDTH as f32; }

                let mut dy = decal.y - state.player.y;
                if dy > MAP_HEIGHT as f32 / 2.0 { dy -= MAP_HEIGHT as f32; }
                else if dy < -(MAP_HEIGHT as f32 / 2.0) { dy += MAP_HEIGHT as f32; }

                dx * dx + dy * dy < 18.0 * 18.0
            })
            .copied()
            .collect();

        // 1. Cast building walls (must be cast before floor for screen-space reflections)
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

        // 2. Cast Floor & Sidewalk markings
        raycaster.cast_floor(
            state.player.x,
            state.player.y,
            state.player.dir_x,
            state.player.dir_y,
            state.player.plane_x,
            state.player.plane_y,
            &state.map,
            &close_decals,
        );

        // 3. Populate sprite list for raycasting
        let mut sprites_to_draw = Vec::new();
        for (idx, citizen) in state.citizens.iter().enumerate() {
            let is_targeted = state.player.target_idx == Some(idx);
            let target_color = if is_targeted {
                if citizen.is_visually_leftsider() {
                    0xff007fff // Neon Pink
                } else {
                    0x39ff14ff // Neon Green
                }
            } else {
                0
            };

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
                        if citizen.is_visually_leftsider() { 9 } else { 7 }
                    } else {
                        if citizen.is_visually_leftsider() { 2 } else { 0 }
                    };
                    frame_base + citizen.walk_frame
                }
                CitizenState::Exploding(t) => {
                    if t < 0.2 { 4 } else { 5 }
                }
                CitizenState::Dead => {
                    continue;
                }
            };

            sprites_to_draw.push(SpriteToRender {
                x: citizen.x,
                y: citizen.y,
                z: 0.0,
                texture_idx: tex_idx,
                is_targeted,
                target_color,
            });
        }

        // Push particles to sprite list
        for p in &state.particles {
            let tex_idx = match p.p_type {
                crate::game::ParticleType::BloodSprinkle => 11,
                crate::game::ParticleType::GoreDebris => 12,
                crate::game::ParticleType::Smoke => {
                    if p.lifetime > 0.6 {
                        14 // Hot yellow/white fire
                    } else if p.lifetime > 0.3 {
                        15 // Orange/pink spark
                    } else {
                        16 // Dark grey smoke
                    }
                }
            };
            
            sprites_to_draw.push(SpriteToRender {
                x: p.x,
                y: p.y,
                z: p.z,
                texture_idx: tex_idx,
                is_targeted: false,
                target_color: 0,
            });
        }

        // Push guided missiles to sprite list for 3D rendering
        for missile in &state.missiles {
            sprites_to_draw.push(SpriteToRender {
                x: missile.x,
                y: missile.y,
                z: missile.z,
                texture_idx: 13, // s13 Guided Missile sprite
                is_targeted: false,
                target_color: 0,
            });
        }

        // Push hover vehicles to sprite list for 3D rendering
        for vehicle in &state.vehicles {
            let hover_z = 0.15 + (get_time() as f32 * vehicle.hover_speed + vehicle.hover_offset).sin() * 0.04;
            sprites_to_draw.push(SpriteToRender {
                x: vehicle.x,
                y: vehicle.y,
                z: hover_z,
                texture_idx: vehicle.sprite_idx,
                is_targeted: false,
                target_color: 0,
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
        // 1. Render low-res 3D world to render target
        let mut camera = Camera2D::from_display_rect(Rect::new(0.0, 0.0, virtual_w, virtual_h));
        camera.render_target = Some(render_target.clone());
        set_camera(&camera);

        clear_background(Color::from_rgba(10, 11, 16, 255));

        // Draw Raycaster screen (screen shake is now applied at high-res upscale)
        draw_texture_ex(
            &screen_texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(virtual_w, virtual_h)),
                ..Default::default()
            }
        );

        // Render 3D Laser beams on top of the view inside the render target
        for laser in &state.lasers {
            let p_start = project_3d(laser.sx, laser.sy, 0.35, state.player.x, state.player.y, state.player.dir_x, state.player.dir_y, state.player.plane_x, state.player.plane_y, virtual_w, virtual_h);
            let p_end = project_3d(laser.ex, laser.ey, 0.35, state.player.x, state.player.y, state.player.dir_x, state.player.dir_y, state.player.plane_x, state.player.plane_y, virtual_w, virtual_h);

            if let (Some(s), Some(e)) = (p_start, p_end) {
                let color = if laser.is_player {
                    Color::new(0.0, 0.94, 1.0, 0.95) // Cyan
                } else {
                    Color::new(1.0, 0.0, 0.1, 0.95)  // Red
                };
                draw_line(s.0, s.1, e.0, e.1, 5.0, color);
                draw_circle(s.0, s.1, 4.0, WHITE);
                draw_circle(e.0, e.1, 4.0, WHITE);
            }
        }

        // Draw low-res screen-space rain overlay
        let rain_color = Color::new(0.55, 0.82, 0.95, 0.28); // Translucent bluish cyberpunk rain
        for drop in &state.rain_drops {
            let sx = drop.x;
            let sy = drop.y;
            let ex = sx - 1.5; // slight wind slant
            let ey = sy + drop.length;
            draw_line(sx, sy, ex, ey, 1.0, rain_color);
        }



        // Damage flash visual indicator
        if state.player.damage_flash > 0.0 {
            let opacity = (state.player.damage_flash * 4.0).min(0.65);
            draw_rectangle(0.0, 0.0, virtual_w, virtual_h, Color::new(1.0, 0.0, 0.1, opacity));
        }

        // Shoot flash screen brighten indicator
        if let WeaponState::Firing(timer) = state.player.weapon_state {
            let opacity = (timer / 0.18) * 0.35; // up to 35% opacity
            draw_rectangle(0.0, 0.0, virtual_w, virtual_h, Color::new(1.0, 1.0, 1.0, opacity));
        }

        // 2. Reset camera and upscale the render target to the screen
        set_default_camera();
        clear_background(Color::from_rgba(10, 11, 16, 255));
        draw_texture_ex(
            &render_target.texture,
            shake_x,
            shake_y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(screen_w, screen_h)),
                flip_y: true,
                ..Default::default()
            }
        );



        // 3. Render 3D Floating texts on top of the upscaled screen at native high-resolution
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

        if state.is_in_menu {
            // ==========================================
            // MAIN MENU OVERLAY & INTERFACE
            // ==========================================
            // 1. Dim background
            draw_rectangle(view_x, view_y, view_w, view_h, Color::from_rgba(10, 11, 16, 200));

            // 2. Animated text setup
            let title_text = "RIGHT SIDERS";
            let slogan_base = "go right, be right";
            let slogan_full = "go right, be right*"; // with asterisk for measurement
            let note_text = "*not in a political way";

            // ---- Title slide-in animation ----
            let slide_duration = 1.0f32; // seconds for the slide-in
            let slide_progress = (state.menu_timer / slide_duration).clamp(0.0, 1.0);
            // Ease-out cubic: fast start, smooth deceleration
            let ease = 1.0 - (1.0 - slide_progress).powi(3);
            let title_landed = slide_progress >= 1.0;

            // Spawn explosion particles once when title lands
            if title_landed && !state.menu_title_landed {
                state.menu_title_landed = true;
                game::play_sound("menu_explosion");

                // Trigger screen shake!
                state.screen_shake = 0.8;

                let title_font_size_f = 24.0 * ui_scale;
                let full_title_dim = measure_text(title_text, Some(&font), title_font_size_f as u16, 1.0);
                let char_width = full_title_dim.width / 12.0;

                // Spawn burst of particles at the collision point
                let collision_x = cx - char_width / 2.0;
                let collision_y = view_y + view_h * 0.25 - full_title_dim.height / 2.0;
                let num_particles = 130;
                for i in 0..num_particles {
                    // Deterministic spread using index
                    let angle = (i as f32 / num_particles as f32) * std::f32::consts::TAU
                        + (i as f32 * 2.399); // golden angle offset for variety
                    let speed = 120.0 * ui_scale + (i as f32 % 8.0) * 50.0 * ui_scale;
                    let vx = angle.cos() * speed;
                    let vy = angle.sin() * speed;
                    // Alternate between cyan, pink, and green particles
                    let (r, g, b) = match i % 3 {
                        0 => (0u8, 240u8, 255u8),  // Neon Cyan
                        1 => (255u8, 0u8, 127u8),  // Neon Pink
                        _ => (57u8, 255u8, 20u8),   // Neon Green
                    };
                    state.menu_particles.push(crate::game::MenuParticle {
                        x: collision_x,
                        y: collision_y,
                        vx,
                        vy,
                        lifetime: 0.0,
                        max_lifetime: 0.7 + (i as f32 % 5.0) * 0.18,
                        size: 3.0 * ui_scale + (i as f32 % 5.0) * ui_scale,
                        color_r: r,
                        color_g: g,
                        color_b: b,
                    });
                }

                // Spawn cool shockwaves (expanding neon rings)
                // Shockwave 1: Neon Cyan, expanding fast
                state.menu_shockwaves.push(crate::game::MenuShockwave {
                    x: collision_x,
                    y: collision_y,
                    radius: 0.0,
                    max_radius: 320.0 * ui_scale,
                    speed: 650.0 * ui_scale,
                    lifetime: 0.0,
                    max_lifetime: 0.5,
                    thickness: 10.0 * ui_scale,
                    color_r: 0,
                    color_g: 240,
                    color_b: 255,
                });
                // Shockwave 2: Neon Pink, expanding slightly slower
                state.menu_shockwaves.push(crate::game::MenuShockwave {
                    x: collision_x,
                    y: collision_y,
                    radius: 0.0,
                    max_radius: 260.0 * ui_scale,
                    speed: 480.0 * ui_scale,
                    lifetime: 0.0,
                    max_lifetime: 0.6,
                    thickness: 6.0 * ui_scale,
                    color_r: 255,
                    color_g: 0,
                    color_b: 127,
                });
                // Shockwave 3: Bright white/green, thin and fast
                state.menu_shockwaves.push(crate::game::MenuShockwave {
                    x: collision_x,
                    y: collision_y,
                    radius: 0.0,
                    max_radius: 200.0 * ui_scale,
                    speed: 550.0 * ui_scale,
                    lifetime: 0.0,
                    max_lifetime: 0.4,
                    thickness: 4.0 * ui_scale,
                    color_r: 57,
                    color_g: 255,
                    color_b: 20,
                });
            }

            // Update menu particles
            {
                let dt = get_frame_time();
                for p in state.menu_particles.iter_mut() {
                    p.lifetime += dt;
                    p.x += p.vx * dt;
                    p.y += p.vy * dt;
                    p.vy += 120.0 * ui_scale * dt; // gravity
                    p.vx *= 0.98; // drag
                }
                state.menu_particles.retain(|p| p.lifetime < p.max_lifetime);
            }

            // Slogan typewriter (no asterisk yet)
            let slogan_start_time = slide_duration + 0.3; // slogan starts after title lands
            let slogan_chars = ((state.menu_timer - slogan_start_time) * 15.0).max(0.0) as usize;
            let slogan_limit = slogan_chars.min(slogan_base.len());
            if slogan_limit > state.slogan_chars_played {
                if let Some(c) = slogan_base.chars().nth(state.slogan_chars_played) {
                    if !c.is_whitespace() {
                        game::play_sound("slogan_bling");
                    }
                }
                state.slogan_chars_played = slogan_limit;
            }
            let slogan_visible = &slogan_base[0..slogan_limit];
            let slogan_done = slogan_chars >= slogan_base.len();
            let mut slogan_display = slogan_visible.to_string();
            if slogan_chars > 0 && !slogan_done {
                if (get_time() * 12.0) as i32 % 2 == 0 {
                    slogan_display.push('_');
                }
            }

            // Note + asterisk: smooth fade in/out (pulsing), appears directly
            // The note fades in once the slogan is done
            let note_fade_start = slogan_start_time + (slogan_base.len() as f32) / 15.0 + 0.5;
            let note_active = state.menu_timer > note_fade_start;

            // Play a pling the very first frame the star becomes visible
            if note_active && !state.menu_star_played {
                state.menu_star_played = true;
                game::play_sound("menu_pling");
            }
            let note_alpha = if note_active {
                // Smooth pulsing fade: sine wave between 0.05 and 1.0 (9.0 instead of 3.0 for 3x speed)
                let t = (state.menu_timer - note_fade_start) as f64;
                let fade_in = (t * 2.0).min(1.0); // fade in over 0.5s
                let pulse = (t * 9.0).sin() * 0.475 + 0.525; 
                (fade_in * pulse) as f32
            } else {
                0.0f32
            };

            // Add asterisk to slogan when note is visible
            if note_active {
                slogan_display = slogan_full.to_string();
            }

            // ---- Draw Title with slide-in & glint ----
            let title_font_size_f = 24.0 * ui_scale;
            let full_title_dim = measure_text(title_text, Some(&font), title_font_size_f as u16, 1.0);
            let char_width = full_title_dim.width / 12.0;
            // Final resting position (centered)
            let title_final_x = cx - full_title_dim.width / 2.0;
            let title_y = view_y + view_h * 0.25;

            let glow_offset = 2.0 * ui_scale;
            let pulse = (get_time() * 6.0).sin() as f32 * 0.25 + 0.75;



            // Calculate slide offsets
            let slide_distance = view_w; // start fully off-screen
            let right_offset_x = -slide_distance * (1.0 - ease); // comes from left
            let siders_offset_x = slide_distance * (1.0 - ease); // comes from right

            // Shadow colors (inverted)
            let shadow_color_right = Color::from_rgba(255, 0, 127, (120.0 * pulse) as u8); // pink shadow for RIGHT
            let shadow_color_siders = Color::from_rgba(0, 240, 255, (120.0 * pulse) as u8); // cyan shadow for SIDERS

            // Text colors
            let text_color_right = Color::from_rgba(0, 240, 255, 255); // cyan
            let text_color_siders = Color::from_rgba(255, 0, 127, 255); // pink

            // Glint sweep logic
            let sweep_cycle = 3.5f32; // sweeps every 3.5 seconds
            let sweep_duration = 0.8f32; // sweep takes 0.8 seconds
            let t_mod = (get_time() as f32) % sweep_cycle;
            let glint_progress = if t_mod < sweep_duration {
                t_mod / sweep_duration
            } else {
                -1.0 // no glint active
            };
            let glint_range = 40.0 * ui_scale;

            let title_full_str = "RIGHT SIDERS";
            let mut current_char_x = title_final_x;

            for (idx, ch) in title_full_str.chars().enumerate() {
                if ch != ' ' {
                    // Base position
                    let ch_base_x = current_char_x;

                    // Apply slide offsets (RIGHT slides from left, SIDERS slides from right)
                    let ch_draw_x = if idx < 6 {
                        ch_base_x + right_offset_x
                    } else {
                        ch_base_x + siders_offset_x
                    };

                    // Determine base colors
                    let (base_color, shadow_color) = if idx < 6 {
                        (text_color_right, shadow_color_right)
                    } else {
                        (text_color_siders, shadow_color_siders)
                    };

                    // Calculate glint factor
                    let mut final_char_color = base_color;
                    let mut final_shadow_color = shadow_color;

                    if glint_progress >= 0.0 && title_landed {
                        let glint_center_x = title_final_x + glint_progress * full_title_dim.width;
                        let dist = (ch_base_x - glint_center_x).abs();
                        if dist < glint_range {
                            // Smooth glint curve
                            let factor = (1.0 - dist / glint_range).max(0.0).powi(2);
                            
                            // Blend base text to white
                            final_char_color = Color::new(
                                (base_color.r + (1.0 - base_color.r) * factor * 0.95).clamp(0.0, 1.0),
                                (base_color.g + (1.0 - base_color.g) * factor * 0.95).clamp(0.0, 1.0),
                                (base_color.b + (1.0 - base_color.b) * factor * 0.95).clamp(0.0, 1.0),
                                base_color.a,
                            );

                            // Blend shadow/glow to brighter white/cyan/pink
                            final_shadow_color = Color::new(
                                (shadow_color.r + (1.0 - shadow_color.r) * factor * 0.8).clamp(0.0, 1.0),
                                (shadow_color.g + (1.0 - shadow_color.g) * factor * 0.8).clamp(0.0, 1.0),
                                (shadow_color.b + (1.0 - shadow_color.b) * factor * 0.8).clamp(0.0, 1.0),
                                shadow_color.a,
                            );
                        }
                    }

                    // Draw shadow
                    draw_pixel_text(&ch.to_string(), ch_draw_x + glow_offset, title_y + glow_offset, title_font_size_f, final_shadow_color, &font);
                    // Draw text character
                    draw_pixel_text(&ch.to_string(), ch_draw_x, title_y, title_font_size_f, final_char_color, &font);
                }

                current_char_x += char_width;
            }

            // Draw explosion particles
            for p in &state.menu_particles {
                let alpha = 1.0 - (p.lifetime / p.max_lifetime);
                let a = (alpha * 255.0) as u8;
                let size = p.size * alpha; // shrink as they fade
                draw_rectangle(
                    p.x - size / 2.0,
                    p.y - size / 2.0,
                    size,
                    size,
                    Color::from_rgba(p.color_r, p.color_g, p.color_b, a),
                );
            }

            // Draw cool shockwaves
            for sw in &state.menu_shockwaves {
                let alpha = 1.0 - (sw.lifetime / sw.max_lifetime);
                let a = (alpha * 255.0) as u8;
                let current_color = Color::from_rgba(sw.color_r, sw.color_g, sw.color_b, a);
                draw_circle_lines(sw.x, sw.y, sw.radius, sw.thickness * alpha, current_color);
            }

            // Draw Slogan (size 10, neon green, with kinetic character typewriter)
            // Vertically centred between the title baseline and the top of the first button
            let slogan_y = (title_y + p_by) / 2.0;
            if slogan_chars > 0 || note_active {
                let slogan_font_size = 10.0 * ui_scale;
                let full_slogan_dim = measure_text(slogan_full, Some(&font), slogan_font_size as u16, 1.0);
                let slogan_x = cx - full_slogan_dim.width / 2.0;

                // Draw characters one by one
                for (i, ch) in slogan_base.chars().enumerate() {
                    let reveal_time_i = slogan_start_time + (i as f32) / 15.0;
                    let elapsed_i = state.menu_timer - reveal_time_i;

                    if elapsed_i >= 0.0 {
                        // Interpolation factor over 0.25 seconds
                        let t = (elapsed_i / 0.25).clamp(0.0, 1.0);

                        // Color: White (255, 255, 255) to Neon Green (57, 255, 20)
                        let r = (255.0 + (57.0 - 255.0) * t) as u8;
                        let g = 255u8;
                        let b = (255.0 + (20.0 - 255.0) * t) as u8;
                        let color = Color::from_rgba(r, g, b, 255);

                        // Position offset: slides down smoothly from -6.0 pixels (four times as fast as fading, over 0.0625 seconds)
                        let t_pos = (elapsed_i / 0.0625).clamp(0.0, 1.0);
                        let y_offset = -6.0 * ui_scale * (1.0 - t_pos).powi(2);
                        let char_y = slogan_y + y_offset;

                        // Measure substring to find exact X coordinate
                        let sub_dim = measure_text(&slogan_base[0..i], Some(&font), slogan_font_size as u16, 1.0);
                        let char_x = slogan_x + sub_dim.width;

                        draw_pixel_text(&ch.to_string(), char_x, char_y, slogan_font_size, color, &font);
                    }
                }

                // Draw typing cursor at the end of the currently revealed string
                let slogan_done = slogan_chars >= slogan_base.len();
                if !slogan_done && slogan_limit > 0 {
                    if (get_time() * 12.0) as i32 % 2 == 0 {
                        let sub_dim = measure_text(&slogan_base[0..slogan_limit], Some(&font), slogan_font_size as u16, 1.0);
                        let cursor_x = slogan_x + sub_dim.width;
                        draw_pixel_text("_", cursor_x, slogan_y, slogan_font_size, Color::from_rgba(57, 255, 20, 255), &font);
                    }
                }

                // If note is active, draw the asterisk with fading alpha and bright yellow glow overlay pop
                if note_active {
                    let base_dim = measure_text(slogan_base, Some(&font), slogan_font_size as u16, 1.0);
                    let asterisk_x = slogan_x + base_dim.width;
                    let star_alpha = (note_alpha * 255.0) as u8;

                    let elapsed_note = state.menu_timer - note_fade_start;

                    // Glow pop effect when the star first appears (first 0.3s)
                    if elapsed_note < 0.3 {
                        let glow_factor = 1.0 - elapsed_note / 0.3;
                        
                        // Expanding back glow circle
                        let glow_radius = 15.0 * ui_scale * (1.0 - glow_factor);
                        let glow_alpha = (glow_factor * 120.0) as u8;
                        draw_circle(
                            asterisk_x + 4.5 * ui_scale,
                            slogan_y - 4.5 * ui_scale,
                            glow_radius,
                            Color::from_rgba(255, 220, 0, glow_alpha),
                        );

                        // Fading out/shrinking text blur layers
                        let flash_alpha = (glow_factor * 255.0) as u8;
                        for &(dx, dy) in &[(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
                            draw_pixel_text(
                                "*",
                                asterisk_x + dx * 1.5 * ui_scale * glow_factor,
                                slogan_y + dy * 1.5 * ui_scale * glow_factor,
                                slogan_font_size,
                                Color::from_rgba(255, 255, 200, flash_alpha),
                                &font,
                            );
                        }
                    }

                    draw_pixel_text("*", asterisk_x, slogan_y, slogan_font_size, Color::from_rgba(255, 220, 0, star_alpha), &font);
                }
            }

            // Draw Buttons (smooth fade-in after title lands)
            let buttons_start = slide_duration + 0.17; // 3x faster delay (0.5 / 3 ≈ 0.17)
            let buttons_alpha = ((state.menu_timer - buttons_start) * 7.5).clamp(0.0, 1.0); // 3x faster fade-in (2.5 * 3 = 7.5)
            if buttons_alpha > 0.01 {
                // Determine hover states for button background brightness
                let hover_play = mx >= p_bx && mx <= p_bx + max_btn_w && my >= p_by && my <= p_by + btn_h;
                let hover_highscore = mx >= h_bx && mx <= h_bx + max_btn_w && my >= h_by && my <= h_by + btn_h;
                let hover_level = mx >= l_bx && mx <= l_bx + max_btn_w && my >= l_by && my <= l_by + btn_h;

                // Subtle neon/cyberpunk text shake parameters for highlighted button
                let shake_speed_x = 75.0;
                let shake_speed_y = 93.0;
                let shake_amp = 0.7 * ui_scale;

                // Button 1 (Play)
                let play_bg_col = if state.menu_selected_idx == 0 || hover_play {
                    Color::from_rgba(0, 240, 255, (80.0 * buttons_alpha) as u8)
                } else {
                    Color::from_rgba(10, 15, 25, (180.0 * buttons_alpha) as u8)
                };
                let play_border_col = if state.menu_selected_idx == 0 {
                    Color::from_rgba(0, 240, 255, (255.0 * buttons_alpha) as u8)
                } else {
                    Color::from_rgba(0, 240, 255, (60.0 * buttons_alpha) as u8)
                };
                let play_text_col = if state.menu_selected_idx == 0 || hover_play {
                    WHITE
                } else {
                    Color::from_rgba(180, 200, 220, (180.0 * buttons_alpha) as u8)
                };

                let play_shake_x = if state.menu_selected_idx == 0 {
                    (get_time() * shake_speed_x).sin() as f32 * shake_amp
                } else {
                    0.0
                };
                let play_shake_y = if state.menu_selected_idx == 0 {
                    (get_time() * shake_speed_y).cos() as f32 * shake_amp
                } else {
                    0.0
                };

                draw_rectangle(p_bx, p_by, max_btn_w, btn_h, play_bg_col);
                draw_pixel_rect_lines(p_bx, p_by, max_btn_w, btn_h, 2.0 * ui_scale, play_border_col);
                draw_pixel_text(
                    play_text,
                    cx - play_dim.width / 2.0 + play_shake_x,
                    p_by + btn_h / 2.0 + play_dim.height / 2.0 - 0.5 * ui_scale + play_shake_y,
                    btn_font_size,
                    play_text_col,
                    &font,
                );

                // Button 2 (Highscore)
                let highscore_bg_col = if state.menu_selected_idx == 1 || hover_highscore {
                    Color::from_rgba(0, 240, 255, (80.0 * buttons_alpha) as u8)
                } else {
                    Color::from_rgba(10, 15, 25, (180.0 * buttons_alpha) as u8)
                };
                let highscore_border_col = if state.menu_selected_idx == 1 {
                    Color::from_rgba(0, 240, 255, (255.0 * buttons_alpha) as u8)
                } else {
                    Color::from_rgba(0, 240, 255, (60.0 * buttons_alpha) as u8)
                };
                let highscore_text_col = if state.menu_selected_idx == 1 || hover_highscore {
                    WHITE
                } else {
                    Color::from_rgba(180, 200, 220, (180.0 * buttons_alpha) as u8)
                };

                let hs_shake_x = if state.menu_selected_idx == 1 {
                    (get_time() * shake_speed_x).sin() as f32 * shake_amp
                } else {
                    0.0
                };
                let hs_shake_y = if state.menu_selected_idx == 1 {
                    (get_time() * shake_speed_y).cos() as f32 * shake_amp
                } else {
                    0.0
                };

                draw_rectangle(h_bx, h_by, max_btn_w, btn_h, highscore_bg_col);
                draw_pixel_rect_lines(h_bx, h_by, max_btn_w, btn_h, 2.0 * ui_scale, highscore_border_col);
                draw_pixel_text(
                    highscore_text,
                    cx - highscore_dim.width / 2.0 + hs_shake_x,
                    h_by + btn_h / 2.0 + highscore_dim.height / 2.0 - 0.5 * ui_scale + hs_shake_y,
                    btn_font_size,
                    highscore_text_col,
                    &font,
                );

                // Button 3 (Level Select) - Grayed Out / Unavailable
                let level_bg_col = if state.menu_selected_idx == 2 || hover_level {
                    Color::from_rgba(25, 25, 30, (120.0 * buttons_alpha) as u8)
                } else {
                    Color::from_rgba(10, 10, 12, (150.0 * buttons_alpha) as u8)
                };
                let level_border_col = if state.menu_selected_idx == 2 {
                    Color::from_rgba(140, 140, 150, (200.0 * buttons_alpha) as u8)
                } else {
                    Color::from_rgba(70, 70, 75, (100.0 * buttons_alpha) as u8)
                };
                let level_text_col = if state.menu_selected_idx == 2 || hover_level {
                    Color::from_rgba(150, 150, 160, (225.0 * buttons_alpha) as u8)
                } else {
                    Color::from_rgba(90, 90, 100, (150.0 * buttons_alpha) as u8)
                };

                let lvl_shake_x = if state.menu_selected_idx == 2 {
                    (get_time() * shake_speed_x).sin() as f32 * shake_amp
                } else {
                    0.0
                };
                let lvl_shake_y = if state.menu_selected_idx == 2 {
                    (get_time() * shake_speed_y).cos() as f32 * shake_amp
                } else {
                    0.0
                };

                draw_rectangle(l_bx, l_by, max_btn_w, btn_h, level_bg_col);
                draw_pixel_rect_lines(l_bx, l_by, max_btn_w, btn_h, 2.0 * ui_scale, level_border_col);
                draw_pixel_text(
                    level_text,
                    cx - level_dim.width / 2.0 + lvl_shake_x,
                    l_by + btn_h / 2.0 + level_dim.height / 2.0 - 0.5 * ui_scale + lvl_shake_y,
                    btn_font_size,
                    level_text_col,
                    &font,
                );

                // Draw Info & Music buttons in the bottom left
                let bottom_btn_size = 24.0 * ui_scale;
                let margin_left = 20.0 * ui_scale;
                let margin_bottom = 20.0 * ui_scale;
                let margin_right = 20.0 * ui_scale;
                let info_bx = view_x + margin_left;
                let info_by = view_y + view_h - margin_bottom - bottom_btn_size;
                let music_bx = info_bx + bottom_btn_size + 10.0 * ui_scale;
                let music_by = info_by;

                let fs_bx = view_x + view_w - margin_right - bottom_btn_size;
                let fs_by = info_by;
                let help_bx = fs_bx - bottom_btn_size - 10.0 * ui_scale;
                let help_by = info_by;

                let hover_info = mx >= info_bx && mx <= info_bx + bottom_btn_size && my >= info_by && my <= info_by + bottom_btn_size;
                let hover_music = mx >= music_bx && mx <= music_bx + bottom_btn_size && my >= music_by && my <= music_by + bottom_btn_size;
                let hover_help = mx >= help_bx && mx <= help_bx + bottom_btn_size && my >= help_by && my <= help_by + bottom_btn_size;
                let hover_fs = mx >= fs_bx && mx <= fs_bx + bottom_btn_size && my >= fs_by && my <= fs_by + bottom_btn_size;

                // Draw Info Button [I]
                let info_bg_col = if hover_info {
                    Color::from_rgba(0, 240, 255, (80.0 * buttons_alpha) as u8)
                } else {
                    Color::from_rgba(10, 15, 25, (180.0 * buttons_alpha) as u8)
                };
                let info_border_col = if hover_info {
                    Color::from_rgba(0, 240, 255, (255.0 * buttons_alpha) as u8)
                } else {
                    Color::from_rgba(0, 240, 255, (60.0 * buttons_alpha) as u8)
                };
                let info_text_col = if hover_info {
                    WHITE
                } else {
                    Color::from_rgba(180, 200, 220, (180.0 * buttons_alpha) as u8)
                };

                draw_rectangle(info_bx, info_by, bottom_btn_size, bottom_btn_size, info_bg_col);
                draw_pixel_rect_lines(info_bx, info_by, bottom_btn_size, bottom_btn_size, 2.0 * ui_scale, info_border_col);

                let info_text = "I";
                let info_dim = measure_text(info_text, Some(&font), btn_font_size as u16, 1.0);
                draw_pixel_text(
                    info_text,
                    info_bx + bottom_btn_size / 2.0 - info_dim.width / 2.0,
                    info_by + bottom_btn_size / 2.0 + info_dim.height / 2.0 - 0.5 * ui_scale,
                    btn_font_size,
                    info_text_col,
                    &font,
                );

                // Draw Music Button [♫]
                let music_bg_col = if hover_music {
                    Color::from_rgba(0, 240, 255, (80.0 * buttons_alpha) as u8)
                } else {
                    Color::from_rgba(10, 15, 25, (180.0 * buttons_alpha) as u8)
                };
                let music_border_col = if hover_music {
                    Color::from_rgba(0, 240, 255, (255.0 * buttons_alpha) as u8)
                } else {
                    Color::from_rgba(0, 240, 255, (60.0 * buttons_alpha) as u8)
                };
                let music_icon_col = if hover_music {
                    WHITE
                } else {
                    Color::from_rgba(180, 200, 220, (180.0 * buttons_alpha) as u8)
                };

                draw_rectangle(music_bx, music_by, bottom_btn_size, bottom_btn_size, music_bg_col);
                draw_pixel_rect_lines(music_bx, music_by, bottom_btn_size, bottom_btn_size, 2.0 * ui_scale, music_border_col);

                let icon_size = 10.0 * ui_scale;
                draw_music_note_icon(
                    music_bx + bottom_btn_size / 2.0 - icon_size / 2.0,
                    music_by + bottom_btn_size / 2.0 - icon_size / 2.0,
                    icon_size,
                    music_icon_col,
                );

                // Draw Help Button [H]
                let help_bg_col = if hover_help {
                    Color::from_rgba(0, 240, 255, (80.0 * buttons_alpha) as u8)
                } else {
                    Color::from_rgba(10, 15, 25, (180.0 * buttons_alpha) as u8)
                };
                let help_border_col = if hover_help {
                    Color::from_rgba(0, 240, 255, (255.0 * buttons_alpha) as u8)
                } else {
                    Color::from_rgba(0, 240, 255, (60.0 * buttons_alpha) as u8)
                };
                let help_text_col = if hover_help {
                    WHITE
                } else {
                    Color::from_rgba(180, 200, 220, (180.0 * buttons_alpha) as u8)
                };

                draw_rectangle(help_bx, help_by, bottom_btn_size, bottom_btn_size, help_bg_col);
                draw_pixel_rect_lines(help_bx, help_by, bottom_btn_size, bottom_btn_size, 2.0 * ui_scale, help_border_col);

                let help_text = "H";
                let help_dim = measure_text(help_text, Some(&font), btn_font_size as u16, 1.0);
                draw_pixel_text(
                    help_text,
                    help_bx + bottom_btn_size / 2.0 - help_dim.width / 2.0,
                    help_by + bottom_btn_size / 2.0 + help_dim.height / 2.0 - 0.5 * ui_scale,
                    btn_font_size,
                    help_text_col,
                    &font,
                );

                // Draw Fullscreen Button [Square icon]
                let fs_bg_col = if hover_fs {
                    Color::from_rgba(0, 240, 255, (80.0 * buttons_alpha) as u8)
                } else {
                    Color::from_rgba(10, 15, 25, (180.0 * buttons_alpha) as u8)
                };
                let fs_border_col = if hover_fs {
                    Color::from_rgba(0, 240, 255, (255.0 * buttons_alpha) as u8)
                } else {
                    Color::from_rgba(0, 240, 255, (60.0 * buttons_alpha) as u8)
                };
                let fs_icon_col = if hover_fs {
                    WHITE
                } else {
                    Color::from_rgba(180, 200, 220, (180.0 * buttons_alpha) as u8)
                };

                draw_rectangle(fs_bx, fs_by, bottom_btn_size, bottom_btn_size, fs_bg_col);
                draw_pixel_rect_lines(fs_bx, fs_by, bottom_btn_size, bottom_btn_size, 2.0 * ui_scale, fs_border_col);

                let fs_icon_size = 10.0 * ui_scale;
                draw_fullscreen_icon(
                    fs_bx + bottom_btn_size / 2.0 - fs_icon_size / 2.0,
                    fs_by + bottom_btn_size / 2.0 - fs_icon_size / 2.0,
                    fs_icon_size,
                    fs_icon_col,
                );
            }

            // Draw Side Note at bottom (fades in/out smoothly)
            if note_active {
                let note_font_size = 7.0 * ui_scale;
                let note_dim = measure_text(note_text, Some(&font), note_font_size as u16, 1.0);
                let na = (note_alpha * 255.0) as u8;
                draw_pixel_text(note_text, cx - note_dim.width / 2.0, view_y + view_h * 0.85, note_font_size, Color::from_rgba(255, 220, 0, na), &font);
            }
        } else {
            // ==========================================
            // HUD RETICLE & CORE INTERFACES
            // ==========================================
            let cx = view_x + view_w / 2.0;
            let cy = view_y + view_h / 2.0;

            // ==========================================
            // TIMER COUNTDOWN DISPLAY (Top Right Panel)
            // ==========================================
            let timer_str = format!("{:.2} SEC", state.time_left);
            let timer_font_size = 8.0 * ui_scale;
            let timer_dim = measure_text(&timer_str, Some(&font), (timer_font_size * 1.3) as u16, 1.0);

            let timer_w = (timer_dim.width + 16.0 * ui_scale).round();
            let timer_h = (timer_dim.height + 12.0 * ui_scale).round();

            let tx = (view_x + view_w - timer_w - 15.0 * ui_scale).round();
            let ty = (view_y + 15.0 * ui_scale).round();

            draw_rectangle(tx, ty, timer_w, timer_h, Color::from_rgba(10, 15, 25, 220));
            
            let is_low_time = state.time_left < 10.0;
            let timer_border_col = if is_low_time {
                if (get_time() * 4.0) as i32 % 2 == 0 {
                    Color::from_rgba(255, 0, 127, 220) // Red-pink blink border
                } else {
                    Color::from_rgba(255, 230, 0, 220) // Yellow blink border
                }
            } else {
                Color::from_rgba(0, 240, 255, 180) // Standard cyan border
            };

            let timer_text_col = if is_low_time {
                Color::from_rgba(255, 0, 127, 255) // Red-pink text
            } else {
                Color::from_rgba(57, 255, 20, 255) // Neon green text
            };

            draw_pixel_rect_lines(tx, ty, timer_w, timer_h, 2.0 * ui_scale, timer_border_col);
            draw_pixel_text(
                &timer_str,
                tx + 8.0 * ui_scale,
                ty + timer_dim.height + 5.0 * ui_scale,
                timer_font_size * 1.3,
                timer_text_col,
                &font,
            );


            // Center holographic reticle
            let target_found = state.player.target_idx.is_some();
            let reticle_color = if target_found {
                let target = &state.citizens[state.player.target_idx.unwrap()];
                // Red for Leftsiders/rebels, green for compliant
                if target.is_visually_leftsider() {
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

            // Countdown numbers directly above reticle for the last 5 seconds
            if state.time_left <= 5.0 && state.time_left > 0.0 {
                let countdown_str = format!("{}", state.time_left.ceil() as i32);
                let countdown_font_size = 14.0 * ui_scale;
                let countdown_dim = measure_text(&countdown_str, Some(&font), countdown_font_size as u16, 1.0);

                let shake_intensity = (10.0 - state.time_left).max(0.0); // 0.0 to 10.0
                // Maximum shake offset in pixels (scaling quadratically with more intensity at the very end)
                let max_offset = 1.8 * ui_scale * (shake_intensity / 10.0).powi(2);
                
                let shake_offset_x = ((get_time() * 60.0).sin() as f32) * max_offset;
                let shake_offset_y = ((get_time() * 75.0).cos() as f32) * max_offset;

                let countdown_x = cx - countdown_dim.width / 2.0 + shake_offset_x;
                let countdown_y = cy - ch_size / 2.0 - 8.0 * ui_scale + shake_offset_y;

                // For the last 3 seconds: flash between red and yellow at ~4Hz
                let fg_color = if state.time_left <= 3.0 {
                    if (get_time() * 8.0) as i32 % 2 == 0 {
                        Color::from_rgba(255, 0, 0, 255)   // Red
                    } else {
                        Color::from_rgba(255, 220, 0, 255) // Yellow
                    }
                } else {
                    Color::from_rgba(255, 0, 0, 255) // Always red for 4-10s
                };

                // Draw background shadow
                draw_pixel_text(
                    &countdown_str,
                    countdown_x + 1.5 * ui_scale,
                    countdown_y + 1.5 * ui_scale,
                    countdown_font_size,
                    Color::from_rgba(0, 0, 0, 180),
                    &font,
                );
                // Draw foreground text with flash colour
                draw_pixel_text(
                    &countdown_str,
                    countdown_x,
                    countdown_y,
                    countdown_font_size,
                    fg_color,
                    &font,
                );
            }

            // Biometric Scanner Window (Top-Left)
            if target_found {
                let target = &state.citizens[state.player.target_idx.unwrap()];
                let is_criminal = target.is_visually_leftsider();
                let hud_theme = if is_criminal {
                    if (get_time() * 4.0) as i32 % 2 == 0 {
                        Color::from_rgba(255, 0, 127, 220) // Red/pink theme
                    } else {
                        Color::from_rgba(255, 230, 0, 220) // Yellow theme
                    }
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
                    "STATUS: TRAFFIC OFFENDER"
                } else {
                    "STATUS: COMPLIANT"
                };

                // Measure longest line to dynamically size width
                let d1 = measure_text(line1, Some(&font), font_size as u16, 1.0);
                let d2 = measure_text(&line2, Some(&font), font_size as u16, 1.0);
                let d3 = measure_text(&line3, Some(&font), font_size as u16, 1.0);
                let d4 = measure_text(&line4, Some(&font), font_size as u16, 1.0);
                let d5 = measure_text(&line5, Some(&font), font_size as u16, 1.0);

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

                // Typewriter typing progression
                let speed = 450.0; // 450 characters per second (5x faster)
                let chars_left = (state.focus_text_timer * speed) as usize;

                let process_line = |full_str: &str, chars_left: &mut usize| -> Option<String> {
                    let len = full_str.len();
                    if *chars_left == 0 {
                        None
                    } else if *chars_left >= len {
                        *chars_left -= len;
                        Some(full_str.to_string())
                    } else {
                        let show_len = *chars_left;
                        *chars_left = 0;
                        let mut visible = full_str[0..show_len].to_string();
                        if (get_time() * 12.0) as i32 % 2 == 0 {
                            visible.push('_');
                        }
                        Some(visible)
                    }
                };

                let mut c_left = chars_left;
                let draw_l1 = process_line(line1, &mut c_left);
                let draw_l2 = process_line(&line2, &mut c_left);
                let draw_l3 = process_line(&line3, &mut c_left);
                let draw_l4 = process_line(&line4, &mut c_left);
                let mut draw_l5 = process_line(&line5, &mut c_left);

                // If typing is fully completed, add a slow flashing cursor at the end of line 5
                let total_len = line1.len() + line2.len() + line3.len() + line4.len() + line5.len();
                if chars_left >= total_len {
                    if let Some(ref mut text) = draw_l5 {
                        if (get_time() * 3.0) as i32 % 2 == 0 {
                            text.push('_');
                        }
                    }
                }

                // Scanner Details Text (adjusted offsets and drawn letter-by-letter)
                let padding_x = 8.0 * ui_scale;
                let line_y = 9.0 * ui_scale;

                if let Some(text) = draw_l1 {
                    draw_pixel_text(&text, wx + padding_x, wy + 10.0 * ui_scale, font_size, hud_theme, &font);
                }
                if let Some(text) = draw_l2 {
                    draw_pixel_text(&text, wx + padding_x, wy + 10.0 * ui_scale + line_y, font_size, WHITE, &font);
                }
                if let Some(text) = draw_l3 {
                    draw_pixel_text(&text, wx + padding_x, wy + 10.0 * ui_scale + line_y * 2.0, font_size, Color::from_rgba(180, 200, 220, 255), &font);
                }
                if let Some(text) = draw_l4 {
                    draw_pixel_text(&text, wx + padding_x, wy + 10.0 * ui_scale + line_y * 3.0, font_size, Color::from_rgba(180, 200, 220, 255), &font);
                }
                if let Some(text) = draw_l5 {
                    draw_pixel_text(&text, wx + padding_x, wy + 10.0 * ui_scale + line_y * 4.0, font_size, hud_theme, &font);
                }
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

            // ==========================================
            // MISSILE SPECIAL ATTACK HUD TEXT (above Credits panel)
            // ==========================================
            if !state.is_in_menu {
                let rocket_str = "[R]OCKET";
                let rocket_font_size = font_value_size * 1.1;
                let rx = px + 4.0 * ui_scale;
                let ry = py - 6.0 * ui_scale;

                let color = if state.missile_used {
                    Color::new(0.5, 0.5, 0.5, 0.5) // gray and half transparent
                } else {
                    // flashing yellow red
                    let mix = ((get_time() * 8.0).sin() as f32 * 0.5 + 0.5).clamp(0.0, 1.0);
                    let yellow = Color::new(1.0, 0.9, 0.0, 1.0);
                    let red = Color::new(1.0, 0.0, 0.5, 1.0);
                    Color::new(
                        yellow.r + (red.r - yellow.r) * mix,
                        yellow.g + (red.g - yellow.g) * mix,
                        yellow.b + (red.b - yellow.b) * mix,
                        1.0,
                    )
                };

                // Draw background shadow
                draw_pixel_text(
                    rocket_str,
                    rx + 1.0 * ui_scale,
                    ry + 1.0 * ui_scale,
                    rocket_font_size,
                    Color::new(0.0, 0.0, 0.0, if state.missile_used { 0.2 } else { 0.6 }),
                    &font,
                );
                // Draw rocket label
                draw_pixel_text(
                    rocket_str,
                    rx,
                    ry,
                    rocket_font_size,
                    color,
                    &font,
                );
            }

            // Weapon sprite rendering removed as requested

            // ==========================================
            // PATROL DEBRIEF SUMMARY OVERLAY
            // ==========================================
            if state.is_showing_summary {
                // Dim background
                draw_rectangle(view_x, view_y, view_w, view_h, Color::from_rgba(10, 11, 16, 220));

                let panel_w = (440.0 * ui_scale).min(view_w - 30.0 * ui_scale);
                let panel_h = 180.0 * ui_scale;
                let panel_x = cx - panel_w / 2.0;
                let panel_y = (view_y + (view_h - panel_h) / 2.0 - 15.0 * ui_scale).max(5.0 * ui_scale);

                // Border and background of the debrief panel
                draw_rectangle(panel_x, panel_y, panel_w, panel_h, Color::from_rgba(15, 20, 30, 240));
                draw_pixel_rect_lines(panel_x, panel_y, panel_w, panel_h, 2.0 * ui_scale, Color::from_rgba(0, 240, 255, 180)); // Cyan border

                // Header
                let size_title = 12.0 * ui_scale;
                let size_sub = 8.0 * ui_scale;
                let size_row = 9.0 * ui_scale;

                let t_title = "[ KX-128#67 DEBRIEF ]";

                let title_dim = measure_text(t_title, Some(&font), size_title as u16, 1.0);

                draw_pixel_text(t_title, cx - title_dim.width / 2.0, panel_y + 25.0 * ui_scale, size_title, Color::from_rgba(0, 240, 255, 255), &font);

                // Table column headers
                let label_x = panel_x + 30.0 * ui_scale;
                let count_x = panel_x + panel_w - 150.0 * ui_scale;
                let credit_x = panel_x + panel_w - 30.0 * ui_scale;

                let col_y = panel_y + 45.0 * ui_scale;
                draw_pixel_text("METRIC", label_x, col_y, size_sub, Color::from_rgba(100, 115, 130, 255), &font);
                
                let count_header_dim = measure_text("QTY", Some(&font), size_sub as u16, 1.0);
                draw_pixel_text("QTY", count_x - count_header_dim.width, col_y, size_sub, Color::from_rgba(100, 115, 130, 255), &font);
                
                let credit_header_dim = measure_text("CREDITS", Some(&font), size_sub as u16, 1.0);
                draw_pixel_text("CREDITS", credit_x - credit_header_dim.width, col_y, size_sub, Color::from_rgba(100, 115, 130, 255), &font);

                // Dashed line just below the column headers (the "first line")
                draw_line(panel_x + 20.0 * ui_scale, panel_y + 55.0 * ui_scale, panel_x + panel_w - 20.0 * ui_scale, panel_y + 55.0 * ui_scale, 1.0 * ui_scale, Color::from_rgba(0, 240, 255, 100));

                // Row values logic
                let stage = state.summary_stage;
                let rows = [
                    ("EDUCATED", 0),
                    ("ROCKET", 1),
                    ("COLLATERAL", 2),
                    ("TOTAL", 3)
                ];

                let mut current_y = panel_y + 75.0 * ui_scale;
                let row_gap = 25.0 * ui_scale;

                for &(label, idx) in &rows {
                    // Decide whether this row is visible yet
                    let is_visible = if state.summary_skip_buildup {
                        true
                    } else if state.summary_timer < 0.5 {
                        false
                    } else {
                        stage >= idx
                    };

                    if !is_visible {
                        continue;
                    }

                    // Calculate row quantity and credits
                    let (qty, credits, _) = if idx == stage && !state.summary_skip_buildup {
                        match idx {
                            0 => (
                                ((state.summary_count_anim / 1000.0).abs() as i32).min(state.offenders_killed_laser as i32),
                                state.summary_count_anim as i32,
                                true
                            ),
                            1 => (
                                ((state.summary_count_anim / 750.0).abs() as i32).min(state.offenders_killed_rocket as i32),
                                state.summary_count_anim as i32,
                                true
                            ),
                            2 => (
                                ((state.summary_count_anim / -1250.0).abs() as i32).min(state.collateral_damage_kills as i32),
                                state.summary_count_anim as i32,
                                true
                            ),
                            _ => (0, state.summary_count_anim as i32, true)
                        }
                    } else {
                        match idx {
                            0 => (state.offenders_killed_laser as i32, (state.offenders_killed_laser as i32 * 1000), false),
                            1 => (state.offenders_killed_rocket as i32, (state.offenders_killed_rocket as i32 * 750), false),
                            2 => (state.collateral_damage_kills as i32, (state.collateral_damage_kills as i32 * -1250), false),
                            _ => (0, state.player.credits, false)
                        }
                    };

                    // Draw row name
                    let row_color = if idx == 3 {
                        Color::from_rgba(0, 240, 255, 255) // Cyan for total
                    } else if idx == 2 {
                        Color::from_rgba(255, 0, 127, 220) // Pink/Red for collateral
                    } else {
                        Color::from_rgba(255, 255, 255, 220) // White for others
                    };

                    // Draw line separator just above the Total balance row (the "second line") - shifted up slightly for better spacing
                    if idx == 3 {
                        draw_line(panel_x + 20.0 * ui_scale, current_y - 17.0 * ui_scale, panel_x + panel_w - 20.0 * ui_scale, current_y - 17.0 * ui_scale, 1.0 * ui_scale, Color::from_rgba(0, 240, 255, 80));
                    }

                    draw_pixel_text(label, label_x, current_y, size_row, row_color, &font);

                    // Draw quantity column (only for rows 0, 1, 2)
                    if idx < 3 {
                        let qty_str = qty.to_string();
                        let qty_dim = measure_text(&qty_str, Some(&font), size_row as u16, 1.0);
                        draw_pixel_text(&qty_str, count_x - qty_dim.width, current_y, size_row, row_color, &font);
                    }

                    // Draw credits column
                    let credits_str = if idx == 3 {
                        format!("{} CR", credits)
                    } else {
                        format!("{:+} CR", credits)
                    };
                    let cred_color = if credits > 0 {
                        Color::from_rgba(57, 255, 20, 255) // Green for rewards
                    } else if credits < 0 {
                        Color::from_rgba(255, 0, 127, 255) // Pink for deductions
                    } else {
                        row_color
                    };

                    let credits_dim = measure_text(&credits_str, Some(&font), size_row as u16, 1.0);
                    draw_pixel_text(&credits_str, credit_x - credits_dim.width, current_y, size_row, cred_color, &font);

                    current_y += row_gap;
                }

                // Help/Continue prompt at bottom
                let prompt_y = panel_y + panel_h + 15.0 * ui_scale;
                if stage < 4 && !state.summary_skip_buildup {
                    let prompt_text = if game::is_mobile() {
                        "[ TAP TO SKIP ]"
                    } else {
                        "[ CLICK OR PRESS R TO SKIP ]"
                    };
                    let prompt_dim = measure_text(prompt_text, Some(&font), size_sub as u16, 1.0);
                    // Blinking gray
                    let alpha = ( (get_time() * 5.0).sin().abs() * 120.0 + 80.0 ) as u8;
                    draw_pixel_text(prompt_text, cx - prompt_dim.width / 2.0, prompt_y, size_sub, Color::from_rgba(100, 115, 130, alpha), &font);
                } else {
                    let prompt_text = if game::is_mobile() {
                        "[ TAP TO CONTINUE ]"
                    } else {
                        "[ CLICK OR PRESS R TO CONTINUE ]"
                    };
                    let prompt_dim = measure_text(prompt_text, Some(&font), size_sub as u16, 1.0);
                    // Blinking green
                    let alpha = ( (get_time() * 8.0).sin().abs() * 155.0 + 100.0 ) as u8;
                    draw_pixel_text(prompt_text, cx - prompt_dim.width / 2.0, prompt_y, size_sub, Color::from_rgba(57, 255, 20, alpha), &font);
                }

                // Draw explosion particles on top of summary overlay
                for p in &state.menu_particles {
                    let alpha = 1.0 - (p.lifetime / p.max_lifetime);
                    let a = (alpha * 255.0) as u8;
                    let size = p.size * alpha; // shrink as they fade
                    draw_rectangle(
                        p.x - size / 2.0,
                        p.y - size / 2.0,
                        size,
                        size,
                        Color::from_rgba(p.color_r, p.color_g, p.color_b, a),
                    );
                }

                // Draw cool shockwaves on top of summary overlay
                for sw in &state.menu_shockwaves {
                    let alpha = 1.0 - (sw.lifetime / sw.max_lifetime);
                    let a = (alpha * 255.0) as u8;
                    let current_color = Color::from_rgba(sw.color_r, sw.color_g, sw.color_b, a);
                    draw_circle_lines(sw.x, sw.y, sw.radius, sw.thickness * alpha, current_color);
                }
            }

            // ==========================================
            // RUST-BASED GAME OVER & LEADERBOARD OVERLAYS
            // ==========================================
            if is_game_over || is_bankrupt || state.show_leaderboard {
                if state.is_entering_highscore {
                    // Render initials entry screen
                    draw_rectangle(view_x, view_y, view_w, view_h, Color::from_rgba(10, 11, 16, 230));
                    
                    let size_title = 12.0 * ui_scale;
                    let size_sub = 8.0 * ui_scale;
                    let size_score = 14.0 * ui_scale;
                    let size_prompt = 9.0 * ui_scale;
                    
                    // Header text
                    let t_header = if is_bankrupt {
                        "BUDGET EXCEEDED - DECOMMISSIONED"
                    } else if state.player.health <= 0.0 {
                        "KX-128#67 INTEGRITY CRITICAL // SIM TERMINATED"
                    } else {
                        "PATROL COMPLETE"
                    };
                    
                    let t_sub = if is_bankrupt {
                        ""
                    } else if state.player.health <= 0.0 {
                        "REBEL UNIT DEPLOYED LETHAL FORCE"
                    } else {
                        ""
                    };
                    
                    let dim_header = measure_text(t_header, Some(&font), size_title as u16, 1.0);
                    
                    // Draw headers
                    let header_col = if is_bankrupt || state.player.health <= 0.0 {
                        Color::from_rgba(255, 0, 127, 255) // neon pink
                    } else {
                        Color::from_rgba(0, 240, 255, 255) // neon cyan
                    };
                    
                    // Calculate a vertical baseline that groups elements tightly in the center
                    let mut current_y = view_y + view_h * 0.32;
                    
                    draw_pixel_text(t_header, view_x + (view_w - dim_header.width) / 2.0, current_y, size_title, header_col, &font);
                    current_y += 26.0 * ui_scale;
                    
                    if !t_sub.is_empty() {
                        let dim_sub = measure_text(t_sub, Some(&font), size_sub as u16, 1.0);
                        draw_pixel_text(t_sub, view_x + (view_w - dim_sub.width) / 2.0, current_y, size_sub, WHITE, &font);
                        current_y += 22.0 * ui_scale;
                    }
                    
                    // Draw Score
                    let score_str = format!("SCORE: {} CR", state.player.credits);
                    let dim_score = measure_text(&score_str, Some(&font), size_score as u16, 1.0);
                    draw_pixel_text(&score_str, view_x + (view_w - dim_score.width) / 2.0, current_y, size_score, Color::from_rgba(57, 255, 20, 255), &font);
                    current_y += 36.0 * ui_scale;
                    
                    if state.highscore_input_delay <= 0.0 {
                        if is_bankrupt {
                            let t_confirm = if game::is_mobile() {
                                "TAP TO CONTINUE"
                            } else {
                                "PRESS 'R' TO CONTINUE"
                            };
                            let size_confirm = 9.0 * ui_scale;
                            let dim_confirm = measure_text(t_confirm, Some(&font), size_confirm as u16, 1.0);
                            let pulse = (get_time() * 9.0).sin() * 0.25 + 0.75;
                            let confirm_col = Color::from_rgba(57, 255, 20, (255.0 * pulse) as u8);
                            let confirm_y = current_y + 12.0 * ui_scale;
                            draw_pixel_text(t_confirm, view_x + (view_w - dim_confirm.width) / 2.0, confirm_y, size_confirm, confirm_col, &font);
                        } else {
                            // Draw name entry prompt
                            let t_prompt = "ENTER UNIT INITIALS (3 CHARACTERS)";
                            let dim_prompt = measure_text(t_prompt, Some(&font), size_prompt as u16, 1.0);
                            draw_pixel_text(t_prompt, view_x + (view_w - dim_prompt.width) / 2.0, current_y, size_prompt, Color::from_rgba(148, 163, 184, 255), &font);
                            current_y += 18.0 * ui_scale;
                            
                            // Draw letters with boxes/lines
                            let start_x = cx - 75.0 * ui_scale;
                            let letter_w = 40.0 * ui_scale;
                            let gap = 15.0 * ui_scale;
                            let box_y = current_y;
                            let box_h = 45.0 * ui_scale;
                            
                            for i in 0..3 {
                                let bx = start_x + i as f32 * (letter_w + gap);
                                // Draw box outline
                                let box_color = if state.highscore_name.len() == i {
                                    Color::from_rgba(0, 240, 255, 255) // Active box is Cyan
                                } else {
                                    Color::from_rgba(0, 240, 255, 80) // Inactive box is dim Cyan
                                };
                                draw_pixel_rect_lines(bx, box_y, letter_w, box_h, 2.0 * ui_scale, box_color);
                                
                                // Draw character
                                if i < state.highscore_name.len() {
                                    let char_str = state.highscore_name.chars().nth(i).unwrap().to_string();
                                    let char_size = 20.0 * ui_scale;
                                    let char_dim = measure_text(&char_str, Some(&font), char_size as u16, 1.0);
                                    draw_pixel_text(
                                        &char_str,
                                        bx + (letter_w - char_dim.width) / 2.0,
                                        box_y + box_h / 2.0 + char_dim.height / 2.0 - 2.0,
                                        char_size,
                                        WHITE,
                                        &font,
                                    );
                                } else if state.highscore_name.len() == i {
                                    // Blinking cursor in active box
                                    if (get_time() * 4.0) as i32 % 2 == 0 {
                                        let cursor_str = "_";
                                        let char_size = 20.0 * ui_scale;
                                        let char_dim = measure_text(cursor_str, Some(&font), char_size as u16, 1.0);
                                        draw_pixel_text(
                                            cursor_str,
                                            bx + (letter_w - char_dim.width) / 2.0,
                                            box_y + box_h / 2.0 + char_dim.height / 2.0 - 2.0,
                                            char_size,
                                            Color::from_rgba(0, 240, 255, 255),
                                            &font,
                                        );
                                    }
                                }
                            }
                            
                            // Draw confirmation hint if name is filled
                            if state.highscore_name.len() == 3 {
                                let t_confirm = if game::is_mobile() {
                                    "TAP TO TRANSMIT DATA"
                                } else {
                                    "PRESS 'R' TO TRANSMIT DATA"
                                };
                                let size_confirm = 9.0 * ui_scale;
                                let dim_confirm = measure_text(t_confirm, Some(&font), size_confirm as u16, 1.0);
                                let pulse = (get_time() * 9.0).sin() * 0.25 + 0.75; // 3x faster blinking (matching reboot prompt)
                                let confirm_col = Color::from_rgba(57, 255, 20, (255.0 * pulse) as u8);
                                let confirm_y = box_y + box_h + 20.0 * ui_scale;
                                draw_pixel_text(t_confirm, view_x + (view_w - dim_confirm.width) / 2.0, confirm_y, size_confirm, confirm_col, &font);
                            }
                        }
                    }
                } else if state.show_leaderboard {
                    // Render Top 10 rankings table
                    draw_rectangle(view_x, view_y, view_w, view_h, Color::from_rgba(10, 11, 16, 240));
                    
                    let size_headers = 8.0 * ui_scale;
                    let size_row = 8.0 * ui_scale;
                    let size_msg = 9.0 * ui_scale;
                    let size_reboot = 9.0 * ui_scale;
                    
                    // Table configuration
                    let table_x = cx - 180.0 * ui_scale;
                    let table_y = view_y + view_h * 0.08;
                    let table_w = 360.0 * ui_scale;
                    let row_h = 14.5 * ui_scale;
                    
                    // Draw table headers
                    let th_rank = "RANK";
                    let th_agent = "UNIT";
                    let th_score = "SCORE";
                    
                    let dim_th_agent = measure_text(th_agent, Some(&font), size_headers as u16, 1.0);
                    let dim_th_score = measure_text(th_score, Some(&font), size_headers as u16, 1.0);
                    
                    let header_col = Color::from_rgba(0, 240, 255, 255);
                    draw_pixel_text(th_rank, table_x + 10.0 * ui_scale, table_y + row_h - 4.0 * ui_scale, size_headers, header_col, &font);
                    draw_pixel_text(th_agent, cx - dim_th_agent.width / 2.0, table_y + row_h - 4.0 * ui_scale, size_headers, header_col, &font);
                    draw_pixel_text(th_score, table_x + table_w - dim_th_score.width - 10.0 * ui_scale, table_y + row_h - 4.0 * ui_scale, size_headers, header_col, &font);
                    
                    // Draw header underline
                    draw_rectangle(table_x, table_y + row_h, table_w, 2.0 * ui_scale, Color::from_rgba(0, 240, 255, 120));
                    
                    // Draw rows
                    for idx in 0..10 {
                        let ry = table_y + row_h * 1.2 + idx as f32 * row_h;
                        
                        let is_new = state.new_rank == Some(idx);
                        let row_color = if is_new {
                            let pulse = (get_time() * 9.0).sin() * 0.25 + 0.75;
                            Color::new(1.0, 0.92, 0.23, pulse as f32) // Yellow blinking (3x faster)
                        } else {
                            Color::from_rgba(180, 200, 220, 255) // Slate blue
                        };
                        
                        // Rank column
                        let rank_str = format!("#{}", idx + 1);
                        draw_pixel_text(&rank_str, table_x + 10.0 * ui_scale, ry + row_h - 4.0 * ui_scale, size_row, row_color, &font);
                        
                        // Get name & score
                        if idx < state.leaderboard_data.len() {
                            let (name, score) = &state.leaderboard_data[idx];
                            
                            // Name column
                            let dim_name = measure_text(name, Some(&font), size_row as u16, 1.0);
                            draw_pixel_text(name, cx - dim_name.width / 2.0, ry + row_h - 4.0 * ui_scale, size_row, row_color, &font);
                            
                            // Score column
                            let score_str = format!("{} CR", score);
                            let dim_score = measure_text(&score_str, Some(&font), size_row as u16, 1.0);
                            draw_pixel_text(&score_str, table_x + table_w - dim_score.width - 10.0 * ui_scale, ry + row_h - 4.0 * ui_scale, size_row, row_color, &font);
                        } else {
                            // Empty row filler
                            let name = "---";
                            let dim_name = measure_text(name, Some(&font), size_row as u16, 1.0);
                            draw_pixel_text(name, cx - dim_name.width / 2.0, ry + row_h - 4.0 * ui_scale, size_row, row_color, &font);
                            
                            let score_str = "--- CR";
                            let dim_score = measure_text(score_str, Some(&font), size_row as u16, 1.0);
                            draw_pixel_text(score_str, table_x + table_w - dim_score.width - 10.0 * ui_scale, ry + row_h - 4.0 * ui_scale, size_row, row_color, &font);
                        }
                        
                        // Thin separator line
                        draw_rectangle(table_x, ry + row_h, table_w, 1.0 * ui_scale, Color::from_rgba(255, 255, 255, 12));
                    }
                    
                    // Message below table
                    let (t_msg, msg_col) = if is_game_over || is_bankrupt {
                        if let Some(rank) = state.new_rank {
                            let color_green = Color::from_rgba(57, 255, 20, 255);
                            let color_yellow = Color::from_rgba(255, 235, 59, 255);
                            let mix_val = ((get_time() * 8.0).sin() * 0.5 + 0.5) as f32;
                            let mut color = Color::new(
                                color_green.r + (color_yellow.r - color_green.r) * mix_val,
                                color_green.g + (color_yellow.g - color_green.g) * mix_val,
                                color_green.b + (color_yellow.b - color_green.b) * mix_val,
                                1.0,
                            );
                            let pulse = (get_time() * 9.0).sin() * 0.3 + 0.7; // Blinking alpha
                            color.a = pulse as f32;
                            (format!("CONGRATULATIONS UNIT! YOU RANKED #{}!", rank + 1), color)
                        } else {
                            ("RANKING INSUFFICIENT. APPLICANT REJECTED.".to_string(), Color::from_rgba(255, 0, 127, 255))
                        }
                    } else {
                        ("UNIT LEADERBOARD RECORDINGS".to_string(), Color::from_rgba(0, 240, 255, 255))
                    };
                    
                    let dim_msg = measure_text(&t_msg, Some(&font), size_msg as u16, 1.0);
                    let msg_y = table_y + row_h * 11.5 + 15.0 * ui_scale;
                    draw_pixel_text(&t_msg, view_x + (view_w - dim_msg.width) / 2.0, msg_y, size_msg, msg_col, &font);
                    
                    // Reboot prompt
                    let t_reboot = if game::is_mobile() {
                        "TAP TO CONTINUE"
                    } else {
                        "PRESS 'R' TO CONTINUE"
                    };
                    let dim_reboot = measure_text(t_reboot, Some(&font), size_reboot as u16, 1.0);
                    let pulse = (get_time() * 9.0).sin() * 0.25 + 0.75; // 3x faster blinking
                    let reboot_col = Color::from_rgba(0, 240, 255, (255.0 * pulse) as u8);
                    let reboot_y = msg_y + 18.0 * ui_scale;
                    draw_pixel_text(t_reboot, view_x + (view_w - dim_reboot.width) / 2.0, reboot_y, size_reboot, reboot_col, &font);

                    // Draw firework particles and shockwaves on top of the leaderboard
                    for p in &state.menu_particles {
                        let alpha = 1.0 - (p.lifetime / p.max_lifetime);
                        let a = (alpha * 255.0) as u8;
                        let size = p.size * alpha; // shrink as they fade
                        draw_rectangle(
                            p.x - size / 2.0,
                            p.y - size / 2.0,
                            size,
                            size,
                            Color::from_rgba(p.color_r, p.color_g, p.color_b, a),
                        );
                    }

                    for sw in &state.menu_shockwaves {
                        let alpha = 1.0 - (sw.lifetime / sw.max_lifetime);
                        let a = (alpha * 255.0) as u8;
                        let current_color = Color::from_rgba(sw.color_r, sw.color_g, sw.color_b, a);
                        draw_circle_lines(sw.x, sw.y, sw.radius, sw.thickness * alpha, current_color);
                    }
                }
            }
        }
        
        next_frame().await;
    }
}
