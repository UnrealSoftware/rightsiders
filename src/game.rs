use crate::map::{CityMap, TileType, MAP_WIDTH, MAP_HEIGHT};

pub const MAX_CITIZENS: usize = 50;

#[cfg(target_arch = "wasm32")]
unsafe extern "C" {
    fn play_sfx(ptr: *const u8, len: usize);
}

pub fn play_sound(name: &str) {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        play_sfx(name.as_ptr(), name.len());
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        println!("[sound] {}", name);
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum CitizenState {
    Walking,
    Exploding(f32), // Timer
    Dead,
}

pub struct Citizen {
    pub x: f32,
    pub y: f32,
    pub base_x: f32,
    pub base_y: f32,
    pub tx: usize,
    pub ty: usize,
    pub prev_tx: usize,
    pub prev_ty: usize,
    pub next_tx: usize,
    pub next_ty: usize,
    pub progress: f32,
    pub speed: f32,
    pub is_leftsider: bool, // Offsets left (criminal) or right (compliant)
    pub is_rebel: bool,      // Armed rebel who attacks the player
    pub health: f32,
    pub state: CitizenState,
    pub shoot_cooldown: f32,
    pub name: String,
    pub id_num: String,
    pub walk_frame: usize, // Animation frame (0 or 1)
}

impl Citizen {
    pub fn align_position(&mut self) {
        // Linear path interpolation
        let sx = self.tx as f32 + 0.5;
        let sy = self.ty as f32 + 0.5;
        let ex = self.next_tx as f32 + 0.5;
        let ey = self.next_ty as f32 + 0.5;

        self.base_x = sx + (ex - sx) * self.progress;
        self.base_y = sy + (ey - sy) * self.progress;

        // Sidewalk Lane Offset
        let dx = ex - sx;
        let dy = ey - sy;
        let len = (dx*dx + dy*dy).sqrt();
        
        if len > 0.01 {
            let ndx = dx / len;
            let ndy = dy / len;
            let px = -ndy;
            let py = ndx;

            let offset_dist = 0.22;
            let mult = if self.is_leftsider { -offset_dist } else { offset_dist };

            self.x = self.base_x + px * mult;
            self.y = self.base_y + py * mult;
        } else {
            self.x = self.base_x;
            self.y = self.base_y;
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum WeaponState {
    Idle,
    Firing(f32), // Timer
}

// Standalone Helper functions for RNG to avoid borrow checker conflicts
fn next_rng(state: &mut u32) -> u32 {
    *state = state.wrapping_mul(1664525).wrapping_add(1013904223);
    *state
}

fn rng_float(state: &mut u32) -> f32 {
    (next_rng(state) as f32) / (u32::MAX as f32)
}

// Standalone Helper to pick a random adjacent walkable tile
fn pick_next_tile(map: &CityMap, rng_state: &mut u32, tx: usize, ty: usize, prev_tx: usize, prev_ty: usize) -> (usize, usize) {
    let mut candidates = Vec::new();
    let neighbors = [
        (tx as i32 + 1, ty as i32),
        (tx as i32 - 1, ty as i32),
        (tx as i32, ty as i32 + 1),
        (tx as i32, ty as i32 - 1),
    ];

    for &(nx, ny) in neighbors.iter() {
        if nx >= 0 && nx < MAP_WIDTH as i32 && ny >= 0 && ny < MAP_HEIGHT as i32 {
            let tile = map.grid[nx as usize][ny as usize];
            match tile {
                TileType::SidewalkVert | TileType::SidewalkHoriz | TileType::Intersection => {
                    candidates.push((nx as usize, ny as usize));
                }
                _ => {}
            }
        }
    }

    if candidates.is_empty() {
        return (tx, ty);
    }

    let filtered: Vec<&(usize, usize)> = candidates.iter()
        .filter(|&&(cx, cy)| !(cx == prev_tx && cy == prev_ty))
        .collect();

    if !filtered.is_empty() {
        let idx = (next_rng(rng_state) as usize) % filtered.len();
        *filtered[idx]
    } else {
        let idx = (next_rng(rng_state) as usize) % candidates.len();
        candidates[idx]
    }
}

pub struct LaserBeam {
    pub sx: f32,
    pub sy: f32,
    pub ex: f32,
    pub ey: f32,
    pub duration: f32,
    pub is_player: bool,
}

pub struct FloatingText {
    pub text: String,
    pub x: f32, // 3D world x
    pub y: f32, // 3D world y
    pub color: u32,
    pub duration: f32,
}

pub struct Player {
    pub x: f32,
    pub y: f32,
    pub dir_x: f32,
    pub dir_y: f32,
    pub plane_x: f32,
    pub plane_y: f32,
    
    pub health: f32,
    pub shield: f32,
    pub battery: f32, // Ammo charge (0.0 to 100.0)
    pub credits: i32,
    
    pub weapon_state: WeaponState,
    pub target_idx: Option<usize>,
    pub damage_flash: f32, // Visual flash timer when hit
}

pub struct GameState {
    pub player: Player,
    pub citizens: Vec<Citizen>,
    pub lasers: Vec<LaserBeam>,
    pub floating_texts: Vec<FloatingText>,
    pub credits_flash: Option<(String, u32, f32)>, // Text, Color, Duration
    pub screen_shake: f32,
    pub map: CityMap,
    // LCG Deterministic PRNG State
    rng_state: u32,
}

impl GameState {
    pub fn new() -> Self {
        let map = CityMap::new();
        
        // Spawn player on a walkable sidewalk tile at X=3.5, Y=2.5
        let player = Player {
            x: 3.5,
            y: 2.5,
            dir_x: 1.0,
            dir_y: 0.0,
            plane_x: 0.0,
            plane_y: 0.66, // standard 66deg FOV
            health: 100.0,
            shield: 100.0,
            battery: 100.0,
            credits: 1000,
            weapon_state: WeaponState::Idle,
            target_idx: None,
            damage_flash: 0.0,
        };

        let mut state = Self {
            player,
            citizens: Vec::new(),
            lasers: Vec::new(),
            floating_texts: Vec::new(),
            credits_flash: None,
            screen_shake: 0.0,
            map,
            rng_state: 123456789,
        };

        // Spawn initial citizens
        let waypoints = state.map.get_waypoints();
        for i in 0..MAX_CITIZENS {
            // Find a waypoint that is reasonably far from the player spawn
            let mut spawn_wp = waypoints[0];
            let wp_idx = (next_rng(&mut state.rng_state) as usize) % waypoints.len();
            let candidate = waypoints[wp_idx];
            let dx = candidate.0 - state.player.x;
            let dy = candidate.1 - state.player.y;
            if (dx*dx + dy*dy).sqrt() > 2.0 {
                spawn_wp = candidate;
            }
            state.spawn_citizen_at(spawn_wp.0 as usize, spawn_wp.1 as usize, i);
        }

        state
    }


    /// Spawn a citizen at a given tile
    pub fn spawn_citizen_at(&mut self, tx: usize, ty: usize, index: usize) {
        let val = next_rng(&mut self.rng_state);
        
        // Compliance profile:
        // - 40% Leftsider violators (is_rebel = false, is_leftsider = true)
        // - 60% Compliant Rightsiders (is_rebel = false, is_leftsider = false)
        let roll = val % 100;
        let is_rebel = false;
        let is_leftsider = roll < 40;

        // Generate names
        let name_prefix = if is_rebel { "REBEL-" } else { "CITIZEN-" };
        let name_char = ((val >> 8) % 26 + 65) as u8 as char;
        let name_num = (val >> 16) % 900 + 100;
        let name = format!("{}{}{}", name_prefix, name_char, name_num);
        let id_num = format!("ID:{:08x}", val);

        // Find walkable neighboring tile
        let (next_tx, next_ty) = pick_next_tile(&self.map, &mut self.rng_state, tx, ty, tx, ty);

        let speed = 0.6 + rng_float(&mut self.rng_state) * 0.6; // speed between 0.6 and 1.2

        let mut citizen = Citizen {
            x: tx as f32 + 0.5,
            y: ty as f32 + 0.5,
            base_x: tx as f32 + 0.5,
            base_y: ty as f32 + 0.5,
            tx,
            ty,
            prev_tx: tx,
            prev_ty: ty,
            next_tx,
            next_ty,
            progress: 0.0,
            speed,
            is_leftsider,
            is_rebel,
            health: 100.0,
            state: CitizenState::Walking,
            shoot_cooldown: 0.5 + rng_float(&mut self.rng_state) * 1.5,
            name,
            id_num,
            walk_frame: 0,
        };

        // Align position with offset
        citizen.align_position();

        if index < self.citizens.len() {
            self.citizens[index] = citizen;
        } else {
            self.citizens.push(citizen);
        }
    }

    /// Primary game state update loop
    pub fn update(&mut self, dt: f32) {
        // Timers update
        if self.player.damage_flash > 0.0 {
            self.player.damage_flash -= dt;
        }
        if self.screen_shake > 0.0 {
            self.screen_shake -= dt * 4.0;
        }

        // Weapon state update
        if let WeaponState::Firing(timer) = self.player.weapon_state {
            let next_timer = timer - dt;
            if next_timer <= 0.0 {
                self.player.weapon_state = WeaponState::Idle;
            } else {
                self.player.weapon_state = WeaponState::Firing(next_timer);
            }
        }

        // Battery cooling/replenishing
        if self.player.weapon_state == WeaponState::Idle {
            self.player.battery = (self.player.battery + dt * 45.0).min(100.0);
        }

        // Update lasers
        self.lasers.retain_mut(|laser| {
            laser.duration -= dt;
            laser.duration > 0.0
        });

        // Update floating text
        self.floating_texts.retain_mut(|txt| {
            txt.duration -= dt;
            txt.y -= dt * 0.3; // drift upward slightly in world coordinates
            txt.duration > 0.0
        });

        // Update credits flash banner
        if let Some((_, _, ref mut duration)) = self.credits_flash {
            *duration -= dt;
            if *duration <= 0.0 {
                self.credits_flash = None;
            }
        }

        // Update citizens
        for i in 0..self.citizens.len() {
            // To satisfy borrow checker, we copy needed state or avoid holding a mutable borrow
            // while calling other self methods. Let's do updates inside a block or use local variables.
            let (new_tx, new_ty, new_prev_tx, new_prev_ty, new_next_tx, new_next_ty, new_progress, switch_compliance) = {
                let citizen = &self.citizens[i];
                if citizen.state == CitizenState::Walking {
                    let progress = citizen.progress + citizen.speed * dt;
                    if progress >= 1.0 {
                        let old_prev_x = citizen.tx;
                        let old_prev_y = citizen.ty;
                        let tx = citizen.next_tx;
                        let ty = citizen.next_ty;
                        // Standalone helper
                        let mut temp_rng = self.rng_state;
                        let (nx, ny) = pick_next_tile(&self.map, &mut temp_rng, tx, ty, old_prev_x, old_prev_y);
                        
                        let switch_roll = next_rng(&mut temp_rng) % 100;
                        let switch = !citizen.is_rebel && (switch_roll < 10);
                        
                        (tx, ty, old_prev_x, old_prev_y, nx, ny, 0.0, Some((switch, temp_rng)))
                    } else {
                        (citizen.tx, citizen.ty, citizen.prev_tx, citizen.prev_ty, citizen.next_tx, citizen.next_ty, progress, None)
                    }
                } else {
                    (citizen.tx, citizen.ty, citizen.prev_tx, citizen.prev_ty, citizen.next_tx, citizen.next_ty, citizen.progress, None)
                }
            };

            // Apply updates to citizen fields
            {
                let citizen = &mut self.citizens[i];
                if citizen.state == CitizenState::Walking {
                    citizen.tx = new_tx;
                    citizen.ty = new_ty;
                    citizen.prev_tx = new_prev_tx;
                    citizen.prev_ty = new_prev_ty;
                    citizen.next_tx = new_next_tx;
                    citizen.next_ty = new_next_ty;
                    citizen.progress = new_progress;
                    if let Some((switch, new_rng_val)) = switch_compliance {
                        self.rng_state = new_rng_val;
                        if switch {
                            citizen.is_leftsider = !citizen.is_leftsider;
                        }
                    }
                    citizen.walk_frame = if (citizen.progress * 4.0) as i32 % 2 == 0 { 0 } else { 1 };
                    citizen.align_position();
                }
            }

            // Execute combat/timer behaviors
            let mut shoot_event = None;
            let mut respawn_event = false;
            let mut explode_done = false;

            {
                let player_x = self.player.x;
                let player_y = self.player.y;
                let citizen = &mut self.citizens[i];

                match citizen.state {
                    CitizenState::Walking => {
                        if citizen.is_rebel {
                            let dx = player_x - citizen.x;
                            let dy = player_y - citizen.y;
                            let dist = (dx*dx + dy*dy).sqrt();

                            if dist < 7.0 {
                                // Simple line of sight check
                                let steps = 15;
                                let mut has_los = true;
                                for step in 1..steps {
                                    let t = step as f32 / steps as f32;
                                    let check_x = citizen.x + dx * t;
                                    let check_y = citizen.y + dy * t;
                                    if self.map.is_solid(check_x, check_y) {
                                        has_los = false;
                                        break;
                                    }
                                }

                                if has_los {
                                    citizen.shoot_cooldown -= dt;
                                    if citizen.shoot_cooldown <= 0.0 {
                                        // Set cooldown using a local rng state update
                                        let mut temp_rng = self.rng_state;
                                        citizen.shoot_cooldown = 1.2 + rng_float(&mut temp_rng) * 1.0;
                                        self.rng_state = temp_rng;

                                        shoot_event = Some((citizen.x, citizen.y));
                                    }
                                }
                            }
                        }
                    }
                    CitizenState::Exploding(ref mut timer) => {
                        *timer += dt;
                        if *timer >= 0.4 {
                            explode_done = true;
                        }
                    }
                    CitizenState::Dead => {
                        citizen.shoot_cooldown -= dt;
                        if citizen.shoot_cooldown <= -8.0 {
                            respawn_event = true;
                        }
                    }
                }
            }

            // Apply any post-borrow effects
            if explode_done {
                self.citizens[i].state = CitizenState::Dead;
            }

            if let Some((sx, sy)) = shoot_event {
                self.lasers.push(LaserBeam {
                    sx,
                    sy,
                    ex: self.player.x,
                    ey: self.player.y,
                    duration: 0.12,
                    is_player: false,
                });

                // Deal damage to player
                let damage = 15.0;
                if self.player.shield > 0.0 {
                    self.player.shield = (self.player.shield - damage).max(0.0);
                } else {
                    self.player.health = (self.player.health - damage).max(0.0);
                }

                self.screen_shake = 0.25;
                self.player.damage_flash = 0.15;
                play_sound("hurt");
            }

            if respawn_event {
                // Respawn this citizen at a random waypoint
                let waypoints = self.map.get_waypoints();
                let mut temp_rng = self.rng_state;
                let wp = waypoints[(next_rng(&mut temp_rng) as usize) % waypoints.len()];
                self.rng_state = temp_rng;
                self.spawn_citizen_at(wp.0 as usize, wp.1 as usize, i);
            }
        }

        // Recalculate player scanner target
        self.update_scanner_target();
    }

    /// Finds the closest citizen directly under the player's crosshair
    fn update_scanner_target(&mut self) {
        let mut closest_idx = None;
        let mut min_dist = 8.0; // Max scanning distance

        for (idx, citizen) in self.citizens.iter().enumerate() {
            if citizen.state != CitizenState::Walking {
                continue;
            }

            let dx = citizen.x - self.player.x;
            let dy = citizen.y - self.player.y;
            let dist = (dx*dx + dy*dy).sqrt();

            if dist < min_dist {
                // Check if citizen is in front of the player
                let ndx = dx / dist;
                let ndy = dy / dist;

                // Dot product with player direction
                let dot = ndx * self.player.dir_x + ndy * self.player.dir_y;
                
                // Narrow scanner cone (roughly 12 degrees wide)
                if dot > 0.98 {
                    // Check line of sight (not blocked by walls)
                    let steps = (dist * 2.0) as i32;
                    let mut has_los = true;
                    for step in 1..steps {
                        let t = step as f32 / steps as f32;
                        let check_x = self.player.x + dx * t;
                        let check_y = self.player.y + dy * t;
                        if self.map.is_solid(check_x, check_y) {
                            has_los = false;
                            break;
                        }
                    }

                    if has_los {
                        min_dist = dist;
                        closest_idx = Some(idx);
                    }
                }
            }
        }

        self.player.target_idx = closest_idx;
    }

    /// Trigger player weapon fire
    pub fn trigger_fire(&mut self) {
        if self.player.weapon_state != WeaponState::Idle || self.player.health <= 0.0 {
            return;
        }

        // Trigger animation
        self.player.weapon_state = WeaponState::Firing(0.18);
        self.screen_shake = 0.12;
        play_sound("laser");

        // Laser beam start and end coordinates
        let beam_end_x: f32;
        let beam_end_y: f32;

        if let Some(target_idx) = self.player.target_idx {
            let target = &mut self.citizens[target_idx];
            beam_end_x = target.x;
            beam_end_y = target.y;

            // Damage target
            target.health -= 100.0;
            if target.health <= 0.0 {
                // Kill target!
                target.state = CitizenState::Exploding(0.0);
                target.shoot_cooldown = 0.0; // reset as respawn timer

                // Check compliance for reward/penalty
                let is_lefty = target.is_leftsider;
                let is_rebel = target.is_rebel;

                if is_lefty || is_rebel {
                    // Correct elimination of criminal
                    let reward = 100;
                    self.player.credits += reward;
                    play_sound("explosion");
                    
                    self.floating_texts.push(FloatingText {
                        text: format!("+{} CR", reward),
                        x: target.x,
                        y: target.y - 0.4,
                        color: 0x39ff14ff, // Neon Green
                        duration: 1.2,
                    });

                    self.credits_flash = Some((
                        format!("CRIMINAL ELIMINATED // +{} CR", reward),
                        0x39ff14ff,
                        1.5
                    ));
                } else {
                    // Collateral Damage! Shot a compliant Rightsider!
                    let penalty = 500;
                    self.player.credits -= penalty;
                    self.player.damage_flash = 0.2; // Red screenshake glow
                    play_sound("collateral");

                    self.floating_texts.push(FloatingText {
                        text: format!("-{} CR COLLATERAL", penalty),
                        x: target.x,
                        y: target.y - 0.4,
                        color: 0xff007fff, // Neon Pink/Red
                        duration: 1.5,
                    });

                    self.credits_flash = Some((
                        "COLLATERAL DAMAGE // PENALTY -500 CR".to_string(),
                        0xff007fff,
                        2.0
                    ));
                }
            }
        } else {
            // Missed shoot, shoot straight into wall in player direction
            // Raycast player direction to find wall hit
            let mut check_dist = 0.0;
            let step = 0.1;
            loop {
                check_dist += step;
                let cx = self.player.x + self.player.dir_x * check_dist;
                let cy = self.player.y + self.player.dir_y * check_dist;
                if self.map.is_solid(cx, cy) || check_dist > 15.0 {
                    beam_end_x = cx;
                    beam_end_y = cy;
                    break;
                }
            }
        }

        // Spawn player laser beam event
        self.lasers.push(LaserBeam {
            sx: self.player.x,
            sy: self.player.y + 0.1, // slightly lower/offset to look like gun barrel
            ex: beam_end_x,
            ey: beam_end_y,
            duration: 0.08,
            is_player: true,
        });
    }

    /// Process player movements and collision checking
    pub fn move_player(&mut self, move_forward: f32, move_strafe: f32, rotate: f32) {
        if self.player.health <= 0.0 {
            return;
        }

        let move_speed = 3.0; // units/sec
        let rot_speed = 2.0;  // rad/sec

        // 1. Rotation
        if rotate != 0.0 {
            let theta = rotate * rot_speed;
            
            let old_dir_x = self.player.dir_x;
            self.player.dir_x = self.player.dir_x * theta.cos() - self.player.dir_y * theta.sin();
            self.player.dir_y = old_dir_x * theta.sin() + self.player.dir_y * theta.cos();
            
            let old_plane_x = self.player.plane_x;
            self.player.plane_x = self.player.plane_x * theta.cos() - self.player.plane_y * theta.sin();
            self.player.plane_y = old_plane_x * theta.sin() + self.player.plane_y * theta.cos();
        }

        // 2. Forward / Backward Movement
        if move_forward != 0.0 {
            let dx = self.player.dir_x * move_forward * move_speed;
            let dy = self.player.dir_y * move_forward * move_speed;

            // Simple wall collision checking with a small buffer radius
            let buffer = 0.25;
            let target_x = self.player.x + dx;
            let target_y = self.player.y + dy;

            let check_x_pos = target_x + if dx > 0.0 { buffer } else { -buffer };
            if !self.map.is_solid(check_x_pos, self.player.y) {
                self.player.x = target_x;
            }
            let check_y_pos = target_y + if dy > 0.0 { buffer } else { -buffer };
            if !self.map.is_solid(self.player.x, check_y_pos) {
                self.player.y = target_y;
            }
        }

        // 3. Strafing Movement
        if move_strafe != 0.0 {
            // Strafe vector is perpendicular to player direction: (-dir_y, dir_x)
            let sx = -self.player.dir_y * move_strafe * move_speed;
            let sy = self.player.dir_x * move_strafe * move_speed;

            let buffer = 0.25;
            let target_x = self.player.x + sx;
            let target_y = self.player.y + sy;

            let check_x_pos = target_x + if sx > 0.0 { buffer } else { -buffer };
            if !self.map.is_solid(check_x_pos, self.player.y) {
                self.player.x = target_x;
            }
            let check_y_pos = target_y + if sy > 0.0 { buffer } else { -buffer };
            if !self.map.is_solid(self.player.x, check_y_pos) {
                self.player.y = target_y;
            }
        }
    }
}
