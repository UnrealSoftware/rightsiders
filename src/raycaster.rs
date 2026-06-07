// 3D Raycasting Engine for Rightsiders

use crate::assets::{GameAssets, TEX_SIZE};
use crate::map::{CityMap, TileType, MAP_WIDTH, MAP_HEIGHT};

pub const WIDTH: usize = 400;
pub const HEIGHT: usize = 300;
pub const VISIBILITY_DIST: f32 = 16.0;

pub struct Raycaster {
    pub pixels: Vec<u32>,   // RGBA buffer (0xRRGGBBAA)
    pub z_buffer: Vec<f32>, // Z-buffer for occlusion
}

pub struct SpriteToRender {
    pub x: f32,
    pub y: f32,
    pub texture_idx: usize, // Index in assets.sprites
}

impl Raycaster {
    pub fn new() -> Self {
        Self {
            pixels: vec![0; WIDTH * HEIGHT],
            z_buffer: vec![0.0; WIDTH],
        }
    }

    /// Clear the screen with a futuristic vertical gradient for the sky
    pub fn clear(&mut self) {
        // Ceiling: Deep cyber-purple to black gradient at horizon
        for y in 0..HEIGHT/2 {
            let t = y as f32 / (HEIGHT as f32 / 2.0);
            // Interpolate color from 0x120822ff (top) to 0x030206ff (horizon)
            let r = (18.0 * (1.0 - t) + 3.0 * t) as u32;
            let g = (8.0 * (1.0 - t) + 2.0 * t) as u32;
            let b = (34.0 * (1.0 - t) + 6.0 * t) as u32;
            let color = (r << 24) | (g << 16) | (b << 8) | 0xff;

            for x in 0..WIDTH {
                self.pixels[y * WIDTH + x] = color;
            }
        }
        
        // Floor: Initialized to very dark gray, will be filled by floor caster
        let floor_color = 0x08090dff;
        for y in HEIGHT/2..HEIGHT {
            for x in 0..WIDTH {
                self.pixels[y * WIDTH + x] = floor_color;
            }
        }
    }

    /// Perspective-correct floor casting to render roads, sidewalks, and lane markings
    pub fn cast_floor(&mut self, player_x: f32, player_y: f32, dir_x: f32, dir_y: f32, plane_x: f32, plane_y: f32, map: &CityMap) {
        // Dir vectors for leftmost and rightmost rays on screen
        let ray_dir_x0 = dir_x - plane_x;
        let ray_dir_y0 = dir_y - plane_y;
        let ray_dir_x1 = dir_x + plane_x;
        let ray_dir_y1 = dir_y + plane_y;

        for y in (HEIGHT/2 + 1)..HEIGHT {
            // Current y position relative to the center of the screen
            let p = y as f32 - (HEIGHT as f32 / 2.0);
            // Camera height (lowered to 0.4)
            let pos_z = 0.4;
            // Vertical distance from camera to floor row
            let row_distance = pos_z * (HEIGHT as f32) / p;

            // Real world step coordinates for each floor pixel across the row
            let floor_step_x = row_distance * (ray_dir_x1 - ray_dir_x0) / (WIDTH as f32);
            let floor_step_y = row_distance * (ray_dir_y1 - ray_dir_y0) / (WIDTH as f32);

            // Starting real world coordinates for the leftmost pixel in the row
            let mut floor_x = player_x + row_distance * ray_dir_x0;
            let mut floor_y = player_y + row_distance * ray_dir_y0;

            // Fog scaling based on row distance
            let fog = (1.0 - (row_distance / VISIBILITY_DIST)).clamp(0.0, 1.0);

            for x in 0..WIDTH {
                // Determine tile coordinate (wrapping on torus)
                let tx = (floor_x.floor() as i32).rem_euclid(MAP_WIDTH as i32) as usize;
                let ty = (floor_y.floor() as i32).rem_euclid(MAP_HEIGHT as i32) as usize;

                if tx < MAP_WIDTH && ty < MAP_HEIGHT {
                    let tile = map.grid[tx][ty];

                    // Base color for different tile types
                    let mut color = match tile {
                        TileType::Wall(_) => 0x222222ff, // Fallback
                        TileType::Road => 0x0f1013ff,    // Dark asphalt
                        TileType::Intersection => 0x1c1e22ff, // Dark gray paving
                        TileType::SidewalkVert | TileType::SidewalkHoriz => {
                            // Sidewalk pattern
                            let cx = tx as f32 + 0.5;
                            let cy = ty as f32 + 0.5;
                            let dx = floor_x - cx;
                            let dy = floor_y - cy;

                            let mut base_col = 0x242830ff; // Sleek grey metal grid

                            // Neon Cyan border at sidewalk edges (near walls)
                            let edge_limit = 0.44;
                            if tile == TileType::SidewalkVert {
                                if dx.abs() > edge_limit {
                                    base_col = 0x00d0ffff; // Cyan neon edge
                                } else {
                                    // Add grid lines along Y
                                    let fract_y = (floor_y * 4.0).fract();
                                    if fract_y < 0.08 || dx.abs() < 0.01 {
                                        base_col = 0x181a20ff;
                                    }
                                }
                            } else {
                                if dy.abs() > edge_limit {
                                    base_col = 0x00d0ffff; // Cyan neon edge
                                } else {
                                    // Add grid lines along X
                                    let fract_x = (floor_x * 4.0).fract();
                                    if fract_x < 0.08 || dy.abs() < 0.01 {
                                        base_col = 0x181a20ff;
                                    }
                                }
                            }
                            
                            // Centerline warning markings (dashed yellow)
                            if tile == TileType::SidewalkVert {
                                if dx.abs() < 0.03 {
                                    let is_dash = (floor_y * 3.0).fract() > 0.4;
                                    if is_dash {
                                        base_col = 0xffa500ff; // Orange dashed centerline
                                    }
                                }
                            } else {
                                if dy.abs() < 0.03 {
                                    let is_dash = (floor_x * 3.0).fract() > 0.4;
                                    if is_dash {
                                        base_col = 0xffa500ff;
                                    }
                                }
                            }

                            base_col
                        }
                    };

                    // Apply distance fog to the pixel
                    if fog < 1.0 {
                        let r = (((color >> 24) & 0xff) as f32 * fog) as u32;
                        let g = (((color >> 16) & 0xff) as f32 * fog) as u32;
                        let b = (((color >> 8) & 0xff) as f32 * fog) as u32;
                        color = (r << 24) | (g << 16) | (b << 8) | 0xff;
                    }

                    self.pixels[y * WIDTH + x] = color;
                }

                floor_x += floor_step_x;
                floor_y += floor_step_y;
            }
        }
    }

    /// DDA Wall Raycasting
    pub fn cast_walls(&mut self, player_x: f32, player_y: f32, dir_x: f32, dir_y: f32, plane_x: f32, plane_y: f32, map: &CityMap, assets: &GameAssets) {
        for x in 0..WIDTH {
            // Ray direction vector
            let camera_x = 2.0 * (x as f32) / (WIDTH as f32) - 1.0;
            let ray_dir_x = dir_x + plane_x * camera_x;
            let ray_dir_y = dir_y + plane_y * camera_x;

            // Grid cell coordinates
            let mut map_x = player_x as i32;
            let mut map_y = player_y as i32;

            // Distance to next X or Y grid boundary
            let mut side_dist_x: f32;
            let mut side_dist_y: f32;

            // Travel length along ray between grid crossings
            let delta_dist_x = if ray_dir_x == 0.0 { f32::MAX } else { (1.0 / ray_dir_x).abs() };
            let delta_dist_y = if ray_dir_y == 0.0 { f32::MAX } else { (1.0 / ray_dir_y).abs() };

            let step_x: i32;
            let step_y: i32;

            // Calculate step and initial side distance
            if ray_dir_x < 0.0 {
                step_x = -1;
                side_dist_x = (player_x - map_x as f32) * delta_dist_x;
            } else {
                step_x = 1;
                side_dist_x = (map_x as f32 + 1.0 - player_x) * delta_dist_x;
            }
            if ray_dir_y < 0.0 {
                step_y = -1;
                side_dist_y = (player_y - map_y as f32) * delta_dist_y;
            } else {
                step_y = 1;
                side_dist_y = (map_y as f32 + 1.0 - player_y) * delta_dist_y;
            }

            // Perform DDA
            let mut hit = false;
            let mut side = 0; // 0 for X, 1 for Y
            let mut wall_style = 0;

            let max_dda_steps = 18;
            let mut steps = 0;
            while !hit && steps < max_dda_steps {
                if side_dist_x < side_dist_y {
                    side_dist_x += delta_dist_x;
                    map_x += step_x;
                    side = 0;
                } else {
                    side_dist_y += delta_dist_y;
                    map_y += step_y;
                    side = 1;
                }

                // Check wall collision (wrapping on torus)
                let wx = map_x.rem_euclid(MAP_WIDTH as i32) as usize;
                let wy = map_y.rem_euclid(MAP_HEIGHT as i32) as usize;
                if let TileType::Wall(style) = map.grid[wx][wy] {
                    hit = true;
                    wall_style = style;
                }
                steps += 1;
            }

            if !hit {
                self.z_buffer[x] = f32::MAX;
                continue;
            }

            // Calculate perp wall distance to avoid fish-eye
            let perp_wall_dist = if side == 0 {
                side_dist_x - delta_dist_x
            } else {
                side_dist_y - delta_dist_y
            };
            
            // Protect against division by zero
            let perp_wall_dist = if perp_wall_dist < 0.01 { 0.01 } else { perp_wall_dist };
            self.z_buffer[x] = perp_wall_dist;

            // Calculate height of wall strip to draw
            let line_height = (HEIGHT as f32 / perp_wall_dist) as i32;

            // Offset based on camera height pos_z = 0.4
            let pos_z = 0.4_f32;
            let draw_start = -( (1.0 - pos_z) * line_height as f32 ) as i32 + HEIGHT as i32 / 2;
            let draw_end = ( pos_z * line_height as f32 ) as i32 + HEIGHT as i32 / 2;

            // Clamp vertical lines to screen space
            let draw_start_clamped = draw_start.clamp(0, HEIGHT as i32 - 1) as usize;
            let draw_end_clamped = draw_end.clamp(0, HEIGHT as i32 - 1) as usize;

            // Calculate wall texture coordinate (X)
            let mut wall_x = if side == 0 {
                player_y + perp_wall_dist * ray_dir_y
            } else {
                player_x + perp_wall_dist * ray_dir_x
            };
            wall_x -= wall_x.floor();

            let mut tex_x = (wall_x * (TEX_SIZE as f32)) as i32;
            if side == 0 && ray_dir_x > 0.0 {
                tex_x = TEX_SIZE as i32 - 1 - tex_x;
            }
            if side == 1 && ray_dir_y < 0.0 {
                tex_x = TEX_SIZE as i32 - 1 - tex_x;
            }
            let tex_x = tex_x.clamp(0, TEX_SIZE as i32 - 1) as usize;

            // Fetch texture
            let texture = &assets.walls[wall_style as usize];

            // Shading & Fog factors
            let side_shading = if side == 1 { 0.70 } else { 1.0 }; // Darken Y walls
            let fog = (1.0 - (perp_wall_dist / VISIBILITY_DIST)).clamp(0.0, 1.0);
            let intensity = side_shading * fog;

            // Draw the vertical strip of wall
            for y in draw_start_clamped..draw_end_clamped {
                // Calculate texture coordinate (Y)
                let d = y as i32 * 256 - HEIGHT as i32 * 128 + line_height * 128;
                let tex_y = (((d * TEX_SIZE as i32) / line_height) / 256).clamp(0, TEX_SIZE as i32 - 1) as usize;

                let mut pixel = texture.pixels[tex_y * TEX_SIZE + tex_x];

                // Shade pixel color components (RGBA format)
                if intensity < 1.0 {
                    let r = (((pixel >> 24) & 0xff) as f32 * intensity) as u32;
                    let g = (((pixel >> 16) & 0xff) as f32 * intensity) as u32;
                    let b = (((pixel >> 8) & 0xff) as f32 * intensity) as u32;
                    pixel = (r << 24) | (g << 16) | (b << 8) | 0xff;
                }

                self.pixels[y * WIDTH + x] = pixel;
            }
        }
    }

    /// Renders sorted sprites with wall occlusion checking
    pub fn cast_sprites(&mut self, player_x: f32, player_y: f32, dir_x: f32, dir_y: f32, plane_x: f32, plane_y: f32, sprites: &[SpriteToRender], assets: &GameAssets) {
        // Sort sprites by distance descending (using torus wrapped distance)
        let mut sorted_indices: Vec<usize> = (0..sprites.len()).collect();
        sorted_indices.sort_by(|&a, &b| {
            let mut dx_a = sprites[a].x - player_x;
            if dx_a > MAP_WIDTH as f32 / 2.0 { dx_a -= MAP_WIDTH as f32; }
            else if dx_a < -(MAP_WIDTH as f32 / 2.0) { dx_a += MAP_WIDTH as f32; }

            let mut dy_a = sprites[a].y - player_y;
            if dy_a > MAP_HEIGHT as f32 / 2.0 { dy_a -= MAP_HEIGHT as f32; }
            else if dy_a < -(MAP_HEIGHT as f32 / 2.0) { dy_a += MAP_HEIGHT as f32; }

            let mut dx_b = sprites[b].x - player_x;
            if dx_b > MAP_WIDTH as f32 / 2.0 { dx_b -= MAP_WIDTH as f32; }
            else if dx_b < -(MAP_WIDTH as f32 / 2.0) { dx_b += MAP_WIDTH as f32; }

            let mut dy_b = sprites[b].y - player_y;
            if dy_b > MAP_HEIGHT as f32 / 2.0 { dy_b -= MAP_HEIGHT as f32; }
            else if dy_b < -(MAP_HEIGHT as f32 / 2.0) { dy_b += MAP_HEIGHT as f32; }

            let dist_a = dx_a*dx_a + dy_a*dy_a;
            let dist_b = dx_b*dx_b + dy_b*dy_b;
            dist_b.partial_cmp(&dist_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        for idx in sorted_indices {
            let sprite = &sprites[idx];

            // Translate relative to player (wrapping on torus)
            let mut sprite_x = sprite.x - player_x;
            if sprite_x > MAP_WIDTH as f32 / 2.0 { sprite_x -= MAP_WIDTH as f32; }
            else if sprite_x < -(MAP_WIDTH as f32 / 2.0) { sprite_x += MAP_WIDTH as f32; }

            let mut sprite_y = sprite.y - player_y;
            if sprite_y > MAP_HEIGHT as f32 / 2.0 { sprite_y -= MAP_HEIGHT as f32; }
            else if sprite_y < -(MAP_HEIGHT as f32 / 2.0) { sprite_y += MAP_HEIGHT as f32; }

            // Transform matrix inversion
            let inv_det = 1.0 / (plane_x * dir_y - dir_x * plane_y);
            let transform_x = inv_det * (dir_y * sprite_x - dir_x * sprite_y);
            let transform_y = inv_det * (-plane_y * sprite_x + plane_x * sprite_y); // Depth of sprite

            // Sprite must be in front of player
            if transform_y <= 0.08 {
                continue;
            }

            // Screen X projection
            let sprite_screen_x = ((WIDTH as f32 / 2.0) * (1.0 + transform_x / transform_y)) as i32;

            // Height/width scaling (citizens are 0.6x wall height, standing on floor)
            let pos_z = 0.4_f32; // Camera height
            let full_height = (HEIGHT as f32 / transform_y).abs() as i32;
            let sprite_height = (full_height as f32 * 0.6) as i32;
            let sprite_width = (full_height as f32 * 0.6) as i32;

            let draw_end_y_unclamped = HEIGHT as i32 / 2 + (pos_z * full_height as f32) as i32;
            let draw_start_y_unclamped = draw_end_y_unclamped - sprite_height;
            let draw_start_y = draw_start_y_unclamped.clamp(0, HEIGHT as i32 - 1);
            let draw_end_y = draw_end_y_unclamped.clamp(0, HEIGHT as i32 - 1);

            let draw_start_x = (-sprite_width / 2 + sprite_screen_x).clamp(0, WIDTH as i32 - 1);
            let draw_end_x = (sprite_width / 2 + sprite_screen_x).clamp(0, WIDTH as i32 - 1);

            let texture = &assets.sprites[sprite.texture_idx];
            let fog = (1.0 - (transform_y / VISIBILITY_DIST)).clamp(0.0, 1.0);

            // Draw the sprite column by column
            for stripe in draw_start_x..draw_end_x {
                // Check Z-Buffer: sprite is blocked by walls
                if transform_y >= self.z_buffer[stripe as usize] {
                    continue;
                }

                let tex_x = ((256 * (stripe - (-sprite_width / 2 + sprite_screen_x)) * TEX_SIZE as i32 / sprite_width) / 256)
                    .clamp(0, TEX_SIZE as i32 - 1) as usize;

                for y in draw_start_y..draw_end_y {
                    let d = (y - draw_start_y_unclamped) * 256;
                    let tex_y = (((d * TEX_SIZE as i32) / sprite_height) / 256).clamp(0, TEX_SIZE as i32 - 1) as usize;

                    let mut pixel = texture.pixels[tex_y * TEX_SIZE + tex_x];

                    // Transparent chroma-key (Black pixels 0x00000000)
                    if (pixel & 0xff) == 0 {
                        continue;
                    }

                    // Apply distance fog to sprite pixel
                    if fog < 1.0 {
                        let r = (((pixel >> 24) & 0xff) as f32 * fog) as u32;
                        let g = (((pixel >> 16) & 0xff) as f32 * fog) as u32;
                        let b = (((pixel >> 8) & 0xff) as f32 * fog) as u32;
                        pixel = (r << 24) | (g << 16) | (b << 8) | 0xff;
                    }

                    self.pixels[(y as usize) * WIDTH + (stripe as usize)] = pixel;
                }
            }
        }
    }
}
