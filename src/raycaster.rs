// 3D Raycasting Engine for Rightsiders

use crate::assets::{GameAssets, TEX_SIZE};
use crate::map::{CityMap, TileType, MAP_WIDTH, MAP_HEIGHT};
use crate::game::BloodDecal;
use macroquad::prelude::get_time;

pub const WIDTH: usize = 480;
pub const HEIGHT: usize = 270;
pub const VISIBILITY_DIST: f32 = 16.0;

pub struct Raycaster {
    pub pixels: Vec<u32>,   // RGBA buffer (0xRRGGBBAA)
    pub z_buffer: Vec<f32>, // Z-buffer for occlusion
    pub decal_grid: Vec<Vec<usize>>, // Grid mapping tile index to decal indices
    pub light_grid: Vec<Vec<usize>>, // Grid mapping tile index to light indices
    row_distances: Vec<f32>,
    row_fogs: Vec<f32>,
    pub wall_frames: Vec<[u8; 32]>, // Precomputed wall texture frame sequence
}

#[derive(Clone, Copy)]
pub struct Spotlight {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub dir_x: f32,
    pub dir_y: f32,
    pub dir_z: f32,
    pub cos_cutoff: f32,
    pub range: f32,
    pub color_r: f32,
    pub color_g: f32,
    pub color_b: f32,
}

pub struct SpriteToRender {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub texture_idx: usize, // Index in assets.sprites
    pub is_targeted: bool,
    pub target_color: u32,
    pub angle: f32, // Rotation angle in radians
}

impl Raycaster {
    pub fn new() -> Self {
        let mut row_distances = Vec::with_capacity(HEIGHT);
        let mut row_fogs = Vec::with_capacity(HEIGHT);
        for y in 0..HEIGHT {
            if y > HEIGHT / 2 {
                let p = y as f32 - (HEIGHT as f32 / 2.0);
                let pos_z = 0.4;
                let row_dist = pos_z * (HEIGHT as f32) / p;
                let fog = (1.0 - (row_dist / VISIBILITY_DIST)).clamp(0.0, 1.0);
                row_distances.push(row_dist);
                row_fogs.push(fog);
            } else {
                row_distances.push(0.0);
                row_fogs.push(0.0);
            }
        }

        Self {
            pixels: vec![0; WIDTH * HEIGHT],
            z_buffer: vec![0.0; WIDTH],
            decal_grid: vec![Vec::new(); MAP_WIDTH * MAP_HEIGHT],
            light_grid: vec![Vec::new(); MAP_WIDTH * MAP_HEIGHT],
            row_distances,
            row_fogs,
            wall_frames: vec![[0u8; 32]; MAP_WIDTH * MAP_HEIGHT],
        }
    }

    pub fn populate_light_grid(&mut self, lights: &[Spotlight]) {
        for cell in &mut self.light_grid {
            cell.clear();
        }
        for (idx, light) in lights.iter().enumerate() {
            let range = light.range;
            let min_tx = (light.x - range).floor() as i32;
            let max_tx = (light.x + range).floor() as i32;
            let min_ty = (light.y - range).floor() as i32;
            let max_ty = (light.y + range).floor() as i32;

            for cx in min_tx..=max_tx {
                let tx = cx.rem_euclid(MAP_WIDTH as i32) as usize;
                for cy in min_ty..=max_ty {
                    let ty = cy.rem_euclid(MAP_HEIGHT as i32) as usize;
                    self.light_grid[tx * MAP_HEIGHT + ty].push(idx);
                }
            }
        }
    }

    /// Precompute wall texture selections for the static map
    pub fn precompute_wall_frames(&mut self, map: &CityMap, num_walls: usize) {
        for wx in 0..MAP_WIDTH {
            for wy in 0..MAP_HEIGHT {
                let cell_idx = wx * MAP_HEIGHT + wy;
                if let TileType::Wall(wall_style) = map.grid[wx][wy] {
                    let wall_h = match wall_style {
                        3 => 11.0_f32,
                        2 => 5.0_f32 + (((wx * 11 + wy * 19) % 3) as f32),
                        1 => 6.0_f32 + (((wx * 7 + wy * 13) % 3) as f32),
                        _ => 7.0_f32 + (((wx * 17 + wy * 23) % 4) as f32),
                    };
                    let max_tile_y = wall_h as i32 - 1;
                    let mut has_transitioned = false;
                    for tile_y in 0..=max_tile_y {
                        let idx = tile_y as usize;
                        if idx >= 32 { break; }

                        if tile_y == 0 {
                            let frame = if num_walls >= 8 {
                                let hash = (wx as u32).wrapping_mul(73856093) ^ (wy as u32).wrapping_mul(19349663) ^ 0x9e3779b9;
                                (hash as usize) % 8
                            } else if num_walls >= 2 {
                                let hash = (wx as u32).wrapping_mul(73856093) ^ (wy as u32).wrapping_mul(19349663) ^ 0x9e3779b9;
                                (hash as usize) % 2
                            } else {
                                0
                            };
                            self.wall_frames[cell_idx][idx] = frame as u8;
                        } else if (tile_y == 1 || tile_y == 2) && !has_transitioned {
                            let hash = (wx as u32).wrapping_mul(73856093)
                                ^ (wy as u32).wrapping_mul(19349663)
                                ^ (tile_y as u32).wrapping_mul(83492791)
                                ^ 0xabcdef;
                            let choice = (hash as usize) % num_walls;
                            let is_first_group = if num_walls >= 16 {
                                choice < 8
                            } else {
                                choice < 2
                            };
                            if is_first_group {
                                self.wall_frames[cell_idx][idx] = choice as u8;
                            } else {
                                self.wall_frames[cell_idx][idx] = choice as u8;
                                has_transitioned = true;
                            }
                        } else {
                            let frame = if num_walls >= 16 {
                                let hash = (wx as u32).wrapping_mul(73856093)
                                    ^ (wy as u32).wrapping_mul(19349663)
                                    ^ (tile_y as u32).wrapping_mul(83492791)
                                    ^ 0xabcdef;
                                8 + (hash as usize) % 8
                            } else if num_walls > 2 {
                                let remaining = num_walls - 2;
                                let hash = (wx as u32).wrapping_mul(73856093)
                                    ^ (wy as u32).wrapping_mul(19349663)
                                    ^ (tile_y as u32).wrapping_mul(83492791)
                                    ^ 0xabcdef;
                                2 + (hash as usize) % remaining
                            } else {
                                0
                            };
                            self.wall_frames[cell_idx][idx] = frame as u8;
                        }
                    }
                }
            }
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

    pub fn cast_floor(&mut self, player_x: f32, player_y: f32, dir_x: f32, dir_y: f32, plane_x: f32, plane_y: f32, map: &CityMap, decals: &[BloodDecal], lights: &[Spotlight]) {
        #[inline(always)]
        fn tri_wave(v: f32) -> f32 {
            let fract = (v + 1000.0).fract();
            (fract - 0.5).abs() * 4.0 - 1.0
        }

        let time = get_time() as f32;
        // Precompute draw_end for each screen column to speed up reflection math
        let mut draw_ends = [0; WIDTH];
        for x in 0..WIDTH {
            let perp_wall_dist = self.z_buffer[x];
            draw_ends[x] = if perp_wall_dist >= VISIBILITY_DIST {
                HEIGHT as i32 / 2
            } else {
                let line_height = (HEIGHT as f32 / perp_wall_dist) as i32;
                let pos_z = 0.4_f32;
                (pos_z * line_height as f32) as i32 + HEIGHT as i32 / 2
            };
        }

        // Clear the decal grid
        for cell in &mut self.decal_grid {
            cell.clear();
        }

        // Populate the decal grid with close decals
        for (idx, decal) in decals.iter().enumerate() {
            let min_tx = (decal.x - decal.radius).floor() as i32;
            let max_tx = (decal.x + decal.radius).floor() as i32;
            let min_ty = (decal.y - decal.radius).floor() as i32;
            let max_ty = (decal.y + decal.radius).floor() as i32;

            for cx in min_tx..=max_tx {
                let tx = cx.rem_euclid(MAP_WIDTH as i32) as usize;
                for cy in min_ty..=max_ty {
                    let ty = cy.rem_euclid(MAP_HEIGHT as i32) as usize;
                    self.decal_grid[tx * MAP_HEIGHT + ty].push(idx);
                }
            }
        }

        // Dir vectors for leftmost and rightmost rays on screen
        let ray_dir_x0 = dir_x - plane_x;
        let ray_dir_y0 = dir_y - plane_y;
        let ray_dir_x1 = dir_x + plane_x;
        let ray_dir_y1 = dir_y + plane_y;

        // Hoist the constant scaling factor
        let inv_width = 1.0 / (WIDTH as f32);
        let ray_diff_x = (ray_dir_x1 - ray_dir_x0) * inv_width;
        let ray_diff_y = (ray_dir_y1 - ray_dir_y0) * inv_width;

        for y in (HEIGHT/2 + 1)..HEIGHT {
            // Retrieve precalculated vertical distance and fog
            let row_distance = self.row_distances[y];
            let fog = self.row_fogs[y];
            let fog_int = (fog * 256.0) as u32;

            // Real world step coordinates for each floor pixel across the row
            let floor_step_x = row_distance * ray_diff_x;
            let floor_step_y = row_distance * ray_diff_y;

            // Starting real world coordinates for the leftmost pixel in the row
            let mut floor_x = player_x + row_distance * ray_dir_x0;
            let mut floor_y = player_y + row_distance * ray_dir_y0;

            for x in 0..WIDTH {
                // Depth occlusion check: only cast floor if it is closer than the wall in this column
                if row_distance >= self.z_buffer[x] {
                    floor_x += floor_step_x;
                    floor_y += floor_step_y;
                    continue;
                }

                // Horizon culling: far away floor pixels fade to black directly
                if row_distance >= VISIBILITY_DIST {
                    self.pixels[y * WIDTH + x] = 0x000000ff;
                    floor_x += floor_step_x;
                    floor_y += floor_step_y;
                    continue;
                }

                // Determine tile coordinate (fast positive-wrapped modulo)
                let tx = ((floor_x + 630.0) as usize) % MAP_WIDTH;
                let ty = ((floor_y + 630.0) as usize) % MAP_HEIGHT;

                if tx < MAP_WIDTH && ty < MAP_HEIGHT {
                    let tile = map.grid[tx][ty];
                    let fract_x = (floor_x + 630.0).fract();
                    let fract_y = (floor_y + 630.0).fract();
                    let dx = fract_x - 0.5;
                    let dy = fract_y - 0.5;

                    // Base color for different tile types
                    let mut color = match tile {
                        TileType::Wall(_) => 0x222222ff, // Fallback
                        TileType::Road => 0x0f1013ff,    // Dark asphalt
                        TileType::Intersection => 0x1c1e22ff, // Dark gray paving
                        TileType::SidewalkVert | TileType::SidewalkHoriz => {
                            let mut base_col = 0x242830ff; // Sleek grey metal grid

                            // Neon Cyan border at sidewalk edges (near walls)
                            let edge_limit = 0.44;
                            if tile == TileType::SidewalkVert {
                                if dx.abs() > edge_limit {
                                    base_col = 0x00d0ffff; // Cyan neon edge
                                } else {
                                    // Add grid lines along Y
                                    let fract_y_grid = (floor_y * 4.0).fract();
                                    if fract_y_grid < 0.08 || dx.abs() < 0.01 {
                                        base_col = 0x181a20ff;
                                    }
                                }
                            } else {
                                if dy.abs() > edge_limit {
                                    base_col = 0x00d0ffff; // Cyan neon edge
                                } else {
                                    // Add grid lines along X
                                    let fract_x_grid = (floor_x * 4.0).fract();
                                    if fract_x_grid < 0.08 || dy.abs() < 0.01 {
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

                    // Procedural metallic drain grate
                    let is_drain_tile = match tile {
                        TileType::Wall(_) => false,
                        _ => (tx * 23 + ty * 37) % 11 == 0,
                    };
                    if is_drain_tile {
                        if dx.abs() <= 0.13 && dy.abs() <= 0.13 {
                            let is_rim = dx.abs() > 0.11 || dy.abs() > 0.11;
                            let is_grill = (dx * 25.0).fract().abs() < 0.3;
                            if is_rim || is_grill {
                                color = 0x4f525cff; // Metallic steel color
                            } else {
                                color = 0x050508ff; // Dark slit under grate
                            }
                        }
                    }

                    // Check if it's a puddle on the road or intersection
                    let mut is_puddle = false;
                    if tile == TileType::Road || tile == TileType::Intersection {
                        let tile_hash = ((tx * 13) + (ty * 37)) % 5 == 0;
                        if tile_hash {
                            if dx * dx + dy * dy < 0.12 {
                                is_puddle = true;
                            }
                        }
                    }

                    // Blend blood decals (optimized tile-based mask blending)
                    let mut is_blood = false;
                    let cell_idx = tx * MAP_HEIGHT + ty;
                    let wrapped_x = tx as f32 + fract_x;
                    let wrapped_y = ty as f32 + fract_y;
                    for &decal_idx in &self.decal_grid[cell_idx] {
                        let decal = &decals[decal_idx];
                        let mut bdx = wrapped_x - decal.x;
                        if bdx > 31.5 { bdx -= 63.0; }
                        else if bdx < -31.5 { bdx += 63.0; }

                        let mut bdy = wrapped_y - decal.y;
                        if bdy > 31.5 { bdy -= 63.0; }
                        else if bdy < -31.5 { bdy += 63.0; }

                        if bdx * bdx + bdy * bdy < decal.radius * decal.radius {
                            is_blood = true;
                            break;
                        }
                    }

                    if is_blood {
                        color = 0x8a0303ff; // Set base color to blood red before reflection
                    }

                    // Determine wet reflectiveness percentage
                    let reflect_pct = if is_blood {
                        50
                    } else if is_puddle {
                        60
                    } else {
                        20
                    };

                    // Reconstruct draw_end for this column (using precomputed draw_ends)
                    let draw_end = draw_ends[x];
                    let y_reflected = 2 * draw_end - y as i32;

                    // Ripple waves based on fast triangle wave approximation
                    let ripple_mult = if reflect_pct == 60 { 0.015 } else if reflect_pct == 50 { 0.012 } else { 0.005 };
                    let ripple = ripple_mult * (tri_wave(floor_x * 1.91 + time * 0.95) + tri_wave(floor_y * 1.91 - time * 0.8 + 0.25));
                    let ref_x = (x as f32 + ripple * 45.0) as i32;
                    let ref_y = (y_reflected as f32 + ripple * 25.0) as i32;

                    let ref_x = ref_x.clamp(0, WIDTH as i32 - 1) as usize;
                    let ref_y = ref_y.clamp(0, HEIGHT as i32 - 1) as usize;

                    // Fetch the reflected screen color (which already has walls/sky)
                    let reflected_color = self.pixels[ref_y * WIDTH + ref_x];

                    let r_ref = (reflected_color >> 24) & 0xff;
                    let g_ref = (reflected_color >> 16) & 0xff;
                    let b_ref = (reflected_color >> 8) & 0xff;

                    let r_base = (color >> 24) & 0xff;
                    let g_base = (color >> 16) & 0xff;
                    let b_base = (color >> 8) & 0xff;

                    let mut r = (r_base * (100 - reflect_pct) + r_ref * reflect_pct) / 100;
                    let mut g = (g_base * (100 - reflect_pct) + g_ref * reflect_pct) / 100;
                    let mut b = (b_base * (100 - reflect_pct) + b_ref * reflect_pct) / 100;

                    // Cyberpunk color boost for deep puddles (using fast shifts)
                    if reflect_pct == 60 {
                        r = (r * 218) >> 8;
                        g = (g * 243) >> 8;
                        b = (b * 294) >> 8;
                    }

                    let mut light_r = 0.0;
                    let mut light_g = 0.0;
                    let mut light_b = 0.0;

                    let cell_idx = tx * MAP_HEIGHT + ty;
                    for &light_idx in &self.light_grid[cell_idx] {
                        let light = &lights[light_idx];
                        let mut dx = floor_x - light.x;
                        if dx > 31.5 { dx -= 63.0; }
                        else if dx < -31.5 { dx += 63.0; }

                        let mut dy = floor_y - light.y;
                        if dy > 31.5 { dy -= 63.0; }
                        else if dy < -31.5 { dy += 63.0; }

                        let dz = 0.0 - light.z;

                        let dist_sq = dx*dx + dy*dy + dz*dz;
                        let range = light.range;
                        if dist_sq < range * range {
                            if light.cos_cutoff <= -0.99 {
                                // Fast circular point light path (no sqrt, no divisions, no dot product!)
                                let intensity = 1.0 - dist_sq / (range * range);
                                light_r += light.color_r * intensity;
                                light_g += light.color_g * intensity;
                                light_b += light.color_b * intensity;
                            } else {
                                let dist = dist_sq.sqrt();
                                if dist > 0.01 {
                                    let ux = dx / dist;
                                    let uy = dy / dist;
                                    let uz = dz / dist;

                                    let cos_theta = ux * light.dir_x + uy * light.dir_y + uz * light.dir_z;
                                    if cos_theta > light.cos_cutoff {
                                        let dist_factor = 1.0 - dist / range;
                                        let cone_factor = (cos_theta - light.cos_cutoff) / (1.0 - light.cos_cutoff);
                                        let intensity = dist_factor * cone_factor;

                                        light_r += light.color_r * intensity;
                                        light_g += light.color_g * intensity;
                                        light_b += light.color_b * intensity;
                                    }
                                }
                            }
                        }
                    }

                    r = (r as f32 + light_r).min(255.0) as u32;
                    g = (g as f32 + light_g).min(255.0) as u32;
                    b = (b as f32 + light_b).min(255.0) as u32;

                    // Apply distance fog to the pixel directly
                    if fog_int < 256 {
                        r = (r * fog_int) >> 8;
                        g = (g * fog_int) >> 8;
                        b = (b * fog_int) >> 8;
                    }

                    self.pixels[y * WIDTH + x] = (r.min(255) << 24) | (g.min(255) << 16) | (b.min(255) << 8) | 0xff;
                }

                floor_x += floor_step_x;
                floor_y += floor_step_y;
            }
        }
    }

    /// DDA Wall Raycasting
    pub fn cast_walls(&mut self, player_x: f32, player_y: f32, dir_x: f32, dir_y: f32, plane_x: f32, plane_y: f32, map: &CityMap, assets: &GameAssets, lights: &[Spotlight]) {
        // Pre-wrap player coordinates to avoid negative coords inside DDA and step calculations
        let player_x_wrapped = player_x.rem_euclid(MAP_WIDTH as f32);
        let player_y_wrapped = player_y.rem_euclid(MAP_HEIGHT as f32);

        for x in 0..WIDTH {
            // Ray direction vector
            let camera_x = 2.0 * (x as f32) / (WIDTH as f32) - 1.0;
            let ray_dir_x = dir_x + plane_x * camera_x;
            let ray_dir_y = dir_y + plane_y * camera_x;

            // Grid cell coordinates
            let mut map_x = player_x_wrapped as i32;
            let mut map_y = player_y_wrapped as i32;

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
                side_dist_x = (player_x_wrapped - map_x as f32) * delta_dist_x;
            } else {
                step_x = 1;
                side_dist_x = (map_x as f32 + 1.0 - player_x_wrapped) * delta_dist_x;
            }
            if ray_dir_y < 0.0 {
                step_y = -1;
                side_dist_y = (player_y_wrapped - map_y as f32) * delta_dist_y;
            } else {
                step_y = 1;
                side_dist_y = (map_y as f32 + 1.0 - player_y_wrapped) * delta_dist_y;
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
                    if map_x < 0 {
                        map_x += MAP_WIDTH as i32;
                    } else if map_x >= MAP_WIDTH as i32 {
                        map_x -= MAP_WIDTH as i32;
                    }
                    side = 0;
                } else {
                    side_dist_y += delta_dist_y;
                    map_y += step_y;
                    if map_y < 0 {
                        map_y += MAP_HEIGHT as i32;
                    } else if map_y >= MAP_HEIGHT as i32 {
                        map_y -= MAP_HEIGHT as i32;
                    }
                    side = 1;
                }

                // Check wall collision (wrapped coords maintained dynamically)
                if let TileType::Wall(style) = map.grid[map_x as usize][map_y as usize] {
                    hit = true;
                    wall_style = style;
                }
                steps += 1;
            }

             if !hit {
                 self.z_buffer[x] = f32::MAX;
                 continue;
             }

             let wx = map_x as usize;
             let wy = map_y as usize;

             // Calculate perp wall distance to avoid fish-eye
             let perp_wall_dist = if side == 0 {
                 side_dist_x - delta_dist_x
             } else {
                 side_dist_y - delta_dist_y
             };
             
              // Protect against division by zero
              let perp_wall_dist = if perp_wall_dist < 0.01 { 0.01 } else { perp_wall_dist };
              self.z_buffer[x] = perp_wall_dist;

              let wall_world_x = player_x_wrapped + perp_wall_dist * ray_dir_x;
              let wall_world_y = player_y_wrapped + perp_wall_dist * ray_dir_y;

              // Calculate height of wall strip to draw (height = 1.0 unit)
             let line_height = (HEIGHT as f32 / perp_wall_dist) as i32;

             // Offset based on camera height pos_z = 0.4
             let pos_z = 0.4_f32;

             // Determine wall height based on coordinates for a dynamic skyline (skyscrapers)
             let wall_h = match wall_style {
                 3 => 11.0_f32, // Police HQ is a massive skyscraper
                2 => {
                    // Billboard buildings (varies between 5.0 and 7.0)
                    5.0_f32 + (((wx * 11 + wy * 19) % 3) as f32)
                }
                1 => {
                    // Tech buildings (varies between 6.0 and 8.0)
                    6.0_f32 + (((wx * 7 + wy * 13) % 3) as f32)
                }
                _ => {
                    // Neon Grid skyscrapers (varies between 7.0 and 10.0)
                    7.0_f32 + (((wx * 17 + wy * 23) % 4) as f32)
                }
            };

            let draw_start = -( (wall_h - pos_z) * line_height as f32 ) as i32 + HEIGHT as i32 / 2;
            let draw_end = ( pos_z * line_height as f32 ) as i32 + HEIGHT as i32 / 2;

            // Clamp vertical lines to screen space
            let draw_start_clamped = draw_start.clamp(0, HEIGHT as i32 - 1) as usize;
            let draw_end_clamped = draw_end.clamp(0, HEIGHT as i32 - 1) as usize;

            // Calculate wall texture coordinate (X)
            let mut wall_x = if side == 0 {
                player_y_wrapped + perp_wall_dist * ray_dir_y
            } else {
                player_x_wrapped + perp_wall_dist * ray_dir_x
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

            // Shading & Fog factors
            let side_shading = if side == 1 { 0.70 } else { 1.0 }; // Darken Y walls
            let fog = (1.0 - (perp_wall_dist / VISIBILITY_DIST)).clamp(0.0, 1.0);
            let intensity = side_shading * fog;
            let intensity_int = (intensity * 256.0) as u32;

            // Draw the vertical strip of wall
            let step = (TEX_SIZE as f32) / (line_height as f32);
            let mut tex_y_fp = (draw_start_clamped as i32 - draw_start) as f32 * step;

            let max_tile_y = wall_h as i32 - 1;

            // Retrieve precomputed column frames from cache
            let cell_idx = wx * MAP_HEIGHT + wy;
            let column_frames = &self.wall_frames[cell_idx];

            let mut current_tile_y = -1;
            let mut texture = &assets.walls[0]; // dummy initial value

            // Precalculate reflection angle norms once per column for side == 0 and side == 1
            let angle_norm_0 = {
                let rx = -ray_dir_x;
                let ry = ray_dir_y;
                let angle = ry.atan2(rx);
                (angle + std::f32::consts::PI) / (2.0 * std::f32::consts::PI)
            };
            let angle_norm_1 = {
                let rx = ray_dir_x;
                let ry = -ray_dir_y;
                let angle = ry.atan2(rx);
                (angle + std::f32::consts::PI) / (2.0 * std::f32::consts::PI)
            };

            for y in draw_start_clamped..draw_end_clamped {
                // tex_y_fp is always positive; use fast bitwise & 63 (for TEX_SIZE = 64)
                let tex_y = (tex_y_fp as usize) & (TEX_SIZE - 1);
                let tile_y = (max_tile_y - (tex_y_fp as i32 / TEX_SIZE as i32)).clamp(0, max_tile_y);

                if tile_y != current_tile_y {
                    current_tile_y = tile_y;
                    let frame_idx = column_frames[(tile_y as usize).min(31)] as usize;
                    texture = &assets.walls[frame_idx];
                }

                // tex_y * TEX_SIZE is tex_y << 6 since TEX_SIZE is 64
                let mut pixel = texture.pixels[(tex_y << 6) + tex_x];

                // Detect reflective window key color (Neon Magenta: 0xFF00FFFF)
                if pixel == 0xFF00FFFF {
                    let angle_norm = if side == 0 { angle_norm_0 } else { angle_norm_1 };
                    
                    // Identify individual window panes using sub-tile texture coordinates (spaced approx 16-18 pixels)
                    let sub_x = tex_x >> 4;
                    let sub_y = tex_y >> 4;
                    
                    // Generate a pseudo-random hash unique to this specific window pane on the map
                    let win_hash = wx.wrapping_mul(73856093)
                        ^ wy.wrapping_mul(19349663)
                        ^ sub_x.wrapping_mul(83492791)
                        ^ sub_y.wrapping_mul(43853923);
                    
                    let rx_offset = (win_hash as usize) % 256;
                    let ry_offset = (win_hash >> 8) as usize % 12; // slight vertical shift (up to 12 pixels)
                    
                    // Reflection texture is 256x64
                    let rx = ((angle_norm * 255.0) as usize + rx_offset) % 256;
                    let ry = (tex_y + ry_offset) % 64;
                    
                    let ref_pixel = assets.reflection.pixels[(ry << 8) + rx]; // ry * 256 is ry << 8
                    
                    let r_ref = (ref_pixel >> 24) & 0xff;
                    let g_ref = (ref_pixel >> 16) & 0xff;
                    let b_ref = (ref_pixel >> 8) & 0xff;
                    
                    // Blend: 70% reflection + 30% deep blue glass (0x0A1C47FF)
                    let r_glass = 10u32;
                    let g_glass = 28u32;
                    let b_glass = 71u32;
                    
                    // Division-free blending using fast shifts (dividing by 128)
                    let r_blend = (r_ref * 90 + r_glass * 38) >> 7;
                    let g_blend = (g_ref * 90 + g_glass * 38) >> 7;
                    let b_blend = (b_ref * 90 + b_glass * 38) >> 7;
                    
                    pixel = (r_blend << 24) | (g_blend << 16) | (b_blend << 8) | 0xff;
                }

                let wall_world_z = wall_h - (tex_y_fp / 64.0);
                let mut light_r = 0.0;
                let mut light_g = 0.0;
                let mut light_b = 0.0;

                let cell_idx = wx * MAP_HEIGHT + wy;
                for &light_idx in &self.light_grid[cell_idx] {
                    let light = &lights[light_idx];
                    let mut dx = wall_world_x - light.x;
                    if dx > 31.5 { dx -= 63.0; }
                    else if dx < -31.5 { dx += 63.0; }

                    let mut dy = wall_world_y - light.y;
                    if dy > 31.5 { dy -= 63.0; }
                    else if dy < -31.5 { dy += 63.0; }

                    let dz = wall_world_z - light.z;

                    let dist_sq = dx*dx + dy*dy + dz*dz;
                    let range = light.range;
                    if dist_sq < range * range {
                        if light.cos_cutoff <= -0.99 {
                            // Fast circular point light path (no sqrt, no divisions, no dot product!)
                            let intensity = 1.0 - dist_sq / (range * range);
                            light_r += light.color_r * intensity;
                            light_g += light.color_g * intensity;
                            light_b += light.color_b * intensity;
                        } else {
                            let dist = dist_sq.sqrt();
                            if dist > 0.01 {
                                let ux = dx / dist;
                                let uy = dy / dist;
                                let uz = dz / dist;

                                let cos_theta = ux * light.dir_x + uy * light.dir_y + uz * light.dir_z;
                                if cos_theta > light.cos_cutoff {
                                    let dist_factor = 1.0 - dist / range;
                                    let cone_factor = (cos_theta - light.cos_cutoff) / (1.0 - light.cos_cutoff);
                                    let intensity = dist_factor * cone_factor;

                                    light_r += light.color_r * intensity;
                                    light_g += light.color_g * intensity;
                                    light_b += light.color_b * intensity;
                                }
                            }
                        }
                    }
                }

                let mut r = ((pixel >> 24) & 0xff) as f32;
                let mut g = ((pixel >> 16) & 0xff) as f32;
                let mut b = ((pixel >> 8) & 0xff) as f32;

                r = (r + light_r).min(255.0);
                g = (g + light_g).min(255.0);
                b = (b + light_b).min(255.0);

                pixel = ((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | 0xff;

                // Shade pixel color components (RGBA format using optimized integer math)
                if intensity_int < 256 {
                    let r = (((pixel >> 24) & 0xff) * intensity_int) >> 8;
                    let g = (((pixel >> 16) & 0xff) * intensity_int) >> 8;
                    let b = (((pixel >> 8) & 0xff) * intensity_int) >> 8;
                    pixel = (r << 24) | (g << 16) | (b << 8) | 0xff;
                }

                self.pixels[y * WIDTH + x] = pixel;
                tex_y_fp += step;
            }
        }
    }

    /// Renders sorted sprites with wall occlusion checking
    pub fn cast_sprites(&mut self, player_x: f32, player_y: f32, dir_x: f32, dir_y: f32, plane_x: f32, plane_y: f32, sprites: &[SpriteToRender], assets: &GameAssets) {
        let time = get_time();
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

            // Height/width scaling based on sprite type
            let pos_z = 0.4_f32; // Camera height
            let full_height = (HEIGHT as f32 / transform_y).abs() as i32;
            let scale = match sprite.texture_idx {
                7 => 0.15, // Blood sprinkle
                4 | 8 => 0.25, // Meat chunk
                9 => 0.22, // Guided missile glowing sphere
                10 | 11 | 12 => 0.35, // Smoke trail particles
                15 => 0.25, // Steam small
                16 => 0.45, // Steam medium
                17 => 0.65, // Steam large
                18 | 19 | 20 => 0.70, // Neon signs
                _ => 0.60,  // Citizens
            };

            let texture = &assets.sprites[sprite.texture_idx];
            let tex_w = texture.width;
            let tex_h = texture.height;

            let orig_sprite_height = (full_height as f32 * scale) as i32;
            let orig_sprite_width = (full_height as f32 * scale) as i32;

            // Scale bounding box on screen based on the cropped bounds relative to original 64x64
            let sprite_height = (orig_sprite_height as f32 * (tex_h as f32 / 64.0)) as i32;
            let sprite_width = (orig_sprite_width as f32 * (tex_w as f32 / 64.0)) as i32;

            if sprite_width <= 0 || sprite_height <= 0 {
                continue;
            }

            // Align cropped sprite with the original vertical center
            let orig_draw_end_y = (HEIGHT as i32 / 2).saturating_add(((pos_z - sprite.z) * full_height as f32) as i32);
            let orig_center_y = orig_draw_end_y - orig_sprite_height / 2;

            let fog = (1.0 - (transform_y / VISIBILITY_DIST)).clamp(0.0, 1.0);
            let fog_int = (fog * 256.0) as u32;

            if sprite.angle == 0.0 {
                // Compute screen bounding box based on tight crop offsets
                let draw_start_y_unclamped = (orig_center_y - orig_sprite_height / 2)
                    + (texture.offset_y as f32 * orig_sprite_height as f32 / 64.0) as i32;
                let draw_start_y = draw_start_y_unclamped.clamp(0, HEIGHT as i32 - 1);
                let draw_end_y = (draw_start_y_unclamped + sprite_height).clamp(0, HEIGHT as i32 - 1);

                let draw_start_x_unclamped = (sprite_screen_x - orig_sprite_width / 2)
                    + (texture.offset_x as f32 * orig_sprite_width as f32 / 64.0) as i32;
                let draw_start_x = draw_start_x_unclamped.clamp(0, WIDTH as i32 - 1);
                let draw_end_x = (draw_start_x_unclamped + sprite_width).clamp(0, WIDTH as i32 - 1);

                let step_y = (tex_h as f32) / (sprite_height as f32);

                // Draw the sprite column by column (optimized vertical-strip caster)
                for stripe in draw_start_x..draw_end_x {
                    // Check Z-Buffer: sprite is blocked by walls
                    if transform_y >= self.z_buffer[stripe as usize] {
                        continue;
                    }

                    let tex_x = (((stripe - draw_start_x_unclamped) * tex_w as i32 / sprite_width) as usize)
                        .min(tex_w - 1);

                    let mut tex_y_fp = (draw_start_y - draw_start_y_unclamped) as f32 * step_y;

                    for y in draw_start_y..draw_end_y {
                        let tex_y = (tex_y_fp as usize).min(tex_h - 1);
                        let mut pixel = texture.pixels[tex_y * tex_w + tex_x];
                        tex_y_fp += step_y;

                        // Transparent chroma-key (Black pixels 0x00000000)
                        if (pixel & 0xff) == 0 {
                            continue;
                        }

                        // Apply scanner glow/tint effect if targeted
                        if sprite.is_targeted {
                            let scan_pos = (((time * 3.0).sin() * 0.5 + 0.5) * sprite_height as f64) as i32;
                            let dy = y - draw_start_y_unclamped;
                            if (dy - scan_pos).abs() < 2 {
                                pixel = sprite.target_color;
                            } else {
                                let orig_r = (pixel >> 24) & 0xff;
                                let orig_g = (pixel >> 16) & 0xff;
                                let orig_b = (pixel >> 8) & 0xff;
                                let target_r = (sprite.target_color >> 24) & 0xff;
                                let target_g = (sprite.target_color >> 16) & 0xff;
                                let target_b = (sprite.target_color >> 8) & 0xff;

                                let r = (orig_r * 180 + target_r * 76) >> 8;
                                let g = (orig_g * 180 + target_g * 76) >> 8;
                                let b = (orig_b * 180 + target_b * 76) >> 8;
                                pixel = (r << 24) | (g << 16) | (b << 8) | 0xff;
                            }
                        }

                        // Apply distance fog to sprite pixel (using optimized integer math)
                        if fog_int < 256 {
                            let r = (((pixel >> 24) & 0xff) * fog_int) >> 8;
                            let g = (((pixel >> 16) & 0xff) * fog_int) >> 8;
                            let b = (((pixel >> 8) & 0xff) * fog_int) >> 8;
                            pixel = (r << 24) | (g << 16) | (b << 8) | (pixel & 0xff); // Keep alpha
                        }

                        // CPU-side alpha blending if pixel is translucent (alpha < 255)
                        let alpha = pixel & 0xff;
                        if alpha < 255 {
                            let dest_idx = (y as usize) * WIDTH + (stripe as usize);
                            let dest_pixel = self.pixels[dest_idx];
                            let dest_r = (dest_pixel >> 24) & 0xff;
                            let dest_g = (dest_pixel >> 16) & 0xff;
                            let dest_b = (dest_pixel >> 8) & 0xff;

                            let src_r = (pixel >> 24) & 0xff;
                            let src_g = (pixel >> 16) & 0xff;
                            let src_b = (pixel >> 8) & 0xff;

                            let r_val = src_r * alpha + dest_r * (255 - alpha);
                            let g_val = src_g * alpha + dest_g * (255 - alpha);
                            let b_val = src_b * alpha + dest_b * (255 - alpha);

                            let r = (r_val + 1 + (r_val >> 8)) >> 8;
                            let g = (g_val + 1 + (g_val >> 8)) >> 8;
                            let b = (b_val + 1 + (b_val >> 8)) >> 8;
                            pixel = (r << 24) | (g << 16) | (b << 8) | 0xff;
                        }

                        self.pixels[(y as usize) * WIDTH + (stripe as usize)] = pixel;
                    }
                }
            } else {
                // Rotated caster path
                let cos_t = sprite.angle.cos();
                let sin_t = sprite.angle.sin();

                // Center of the cropped pixels in 64x64 texture coordinates
                let pixel_cx = texture.offset_x as f32 + tex_w as f32 / 2.0;
                let pixel_cy = texture.offset_y as f32 + tex_h as f32 / 2.0;

                // Center of the cropped pixels on screen
                let screen_cx = sprite_screen_x - orig_sprite_width / 2 + (pixel_cx * orig_sprite_width as f32 / 64.0) as i32;
                let screen_cy = orig_center_y - orig_sprite_height / 2 + (pixel_cy * orig_sprite_height as f32 / 64.0) as i32;

                // Draw bounding box of the uncropped 64x64 space to cover the whole rotated range
                let draw_start_x = (sprite_screen_x - orig_sprite_width / 2).clamp(0, WIDTH as i32 - 1);
                let draw_end_x = (sprite_screen_x + orig_sprite_width / 2).clamp(0, WIDTH as i32 - 1);
                let draw_start_y = (orig_center_y - orig_sprite_height / 2).clamp(0, HEIGHT as i32 - 1);
                let draw_end_y = (orig_center_y + orig_sprite_height / 2).clamp(0, HEIGHT as i32 - 1);

                for stripe in draw_start_x..draw_end_x {
                    // Check Z-Buffer: sprite is blocked by walls
                    if transform_y >= self.z_buffer[stripe as usize] {
                        continue;
                    }

                    // Compute rotated distance from screen center (screen_cx, screen_cy) in scaled units
                    let u = (stripe - screen_cx) as f32 / orig_sprite_width as f32;

                    for y in draw_start_y..draw_end_y {
                        let v = (y - screen_cy) as f32 / orig_sprite_height as f32;

                        // Rotate coordinates around screen center (which maps to pixel_cx, pixel_cy)
                        let tx = u * cos_t - v * sin_t;
                        let ty = u * sin_t + v * cos_t;

                        // Convert rotated coordinate back to original 64x64 space relative to pixel_cx, pixel_cy
                        let orig_x = (tx * 64.0 + pixel_cx) as i32;
                        let orig_y = (ty * 64.0 + pixel_cy) as i32;

                        // Check if the rotated pixel lies inside the cropped bounds
                        let tex_x = orig_x - texture.offset_x;
                        let tex_y = orig_y - texture.offset_y;

                        if tex_x >= 0 && tex_x < tex_w as i32 && tex_y >= 0 && tex_y < tex_h as i32 {
                            let mut pixel = texture.pixels[tex_y as usize * tex_w + tex_x as usize];

                            // Transparent chroma-key (Black pixels 0x00000000)
                            if (pixel & 0xff) == 0 {
                                continue;
                            }

                            // Apply scanner glow/tint effect if targeted
                            if sprite.is_targeted {
                                let scan_pos = (((time * 3.0).sin() * 0.5 + 0.5) * sprite_height as f64) as i32;
                                let dy = y - (orig_center_y - sprite_height / 2);
                                if (dy - scan_pos).abs() < 2 {
                                    pixel = sprite.target_color;
                                } else {
                                    let orig_r = (pixel >> 24) & 0xff;
                                    let orig_g = (pixel >> 16) & 0xff;
                                    let orig_b = (pixel >> 8) & 0xff;
                                    let target_r = (sprite.target_color >> 24) & 0xff;
                                    let target_g = (sprite.target_color >> 16) & 0xff;
                                    let target_b = (sprite.target_color >> 8) & 0xff;

                                    let r = (orig_r * 180 + target_r * 76) >> 8;
                                    let g = (orig_g * 180 + target_g * 76) >> 8;
                                    let b = (orig_b * 180 + target_b * 76) >> 8;
                                    pixel = (r << 24) | (g << 16) | (b << 8) | 0xff;
                                }
                            }

                            // Apply distance fog to sprite pixel (using optimized integer math)
                            if fog_int < 256 {
                                let r = (((pixel >> 24) & 0xff) * fog_int) >> 8;
                                let g = (((pixel >> 16) & 0xff) * fog_int) >> 8;
                                let b = (((pixel >> 8) & 0xff) * fog_int) >> 8;
                                pixel = (r << 24) | (g << 16) | (b << 8) | (pixel & 0xff); // Keep alpha
                            }

                            // CPU-side alpha blending if pixel is translucent (alpha < 255)
                            let alpha = pixel & 0xff;
                            if alpha < 255 {
                                let dest_idx = (y as usize) * WIDTH + (stripe as usize);
                                let dest_pixel = self.pixels[dest_idx];
                                let dest_r = (dest_pixel >> 24) & 0xff;
                                let dest_g = (dest_pixel >> 16) & 0xff;
                                let dest_b = (dest_pixel >> 8) & 0xff;

                                let src_r = (pixel >> 24) & 0xff;
                                let src_g = (pixel >> 16) & 0xff;
                                let src_b = (pixel >> 8) & 0xff;

                                let r_val = src_r * alpha + dest_r * (255 - alpha);
                                let g_val = src_g * alpha + dest_g * (255 - alpha);
                                let b_val = src_b * alpha + dest_b * (255 - alpha);

                                let r = (r_val + 1 + (r_val >> 8)) >> 8;
                                let g = (g_val + 1 + (g_val >> 8)) >> 8;
                                let b = (b_val + 1 + (b_val >> 8)) >> 8;
                                pixel = (r << 24) | (g << 16) | (b << 8) | 0xff;
                            }

                            self.pixels[(y as usize) * WIDTH + (stripe as usize)] = pixel;
                        }
                    }
                }
            }
        }
    }
}
