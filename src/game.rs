use crate::map::{CityMap, TileType, MAP_WIDTH, MAP_HEIGHT};

#[cfg(target_arch = "wasm32")]
unsafe extern "C" {
    fn play_sfx(ptr: *const u8, len: usize);
    fn set_menu_active(active: bool);
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

pub fn update_menu_active_js(active: bool) {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        set_menu_active(active);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        println!("[menu_active] {}", active);
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum CitizenState {
    Walking,
    Exploding(f32), // Timer
    Dead,
}

#[derive(Clone, Copy)]
pub struct BloodDecal {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub lifetime: f32,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ParticleType {
    BloodSprinkle,
    GoreDebris,
}

#[derive(Clone)]
pub struct MenuParticle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub size: f32,
    pub color_r: u8,
    pub color_g: u8,
    pub color_b: u8,
}

#[derive(Clone)]
pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub vx: f32,
    pub vy: f32,
    pub vz: f32,
    pub p_type: ParticleType,
    pub bounces: u32,
    pub lifetime: f32,
    pub first_impact: bool,
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
        let mut ex = self.next_tx as f32 + 0.5;
        let mut ey = self.next_ty as f32 + 0.5;

        // Shortest path on torus wrapping
        let mut dx = ex - sx;
        if dx > MAP_WIDTH as f32 / 2.0 {
            ex -= MAP_WIDTH as f32;
            dx = ex - sx;
        } else if dx < -(MAP_WIDTH as f32 / 2.0) {
            ex += MAP_WIDTH as f32;
            dx = ex - sx;
        }

        let mut dy = ey - sy;
        if dy > MAP_HEIGHT as f32 / 2.0 {
            ey -= MAP_HEIGHT as f32;
            dy = ey - sy;
        } else if dy < -(MAP_HEIGHT as f32 / 2.0) {
            ey += MAP_HEIGHT as f32;
            dy = ey - sy;
        }

        self.base_x = sx + (ex - sx) * self.progress;
        self.base_y = sy + (ey - sy) * self.progress;

        // Sidewalk Lane Offset (with minor right offset relative to movement direction for passing)
        let len = (dx*dx + dy*dy).sqrt();
        
        if len > 0.01 {
            let ndx = dx / len;
            let ndy = dy / len;
            
            // Left-normal for lane offset
            let px = -ndy;
            let py = ndx;

            let offset_dist = 0.22;
            let mult = if self.is_leftsider { -offset_dist } else { offset_dist };

            // Right-normal for passing offset
            let rx = ndy;
            let ry = -ndx;
            let passing_offset = 0.07;

            self.x = self.base_x + px * mult + rx * passing_offset;
            self.y = self.base_y + py * mult + ry * passing_offset;
        } else {
            self.x = self.base_x;
            self.y = self.base_y;
        }
    }
}

impl Player {
    pub fn align_position(&mut self) {
        let sx = self.tx as f32 + 0.5;
        let sy = self.ty as f32 + 0.5;
        let mut ex = self.next_tx as f32 + 0.5;
        let mut ey = self.next_ty as f32 + 0.5;

        // Shortest path on torus wrapping
        let mut dx = ex - sx;
        if dx > MAP_WIDTH as f32 / 2.0 {
            ex -= MAP_WIDTH as f32;
            dx = ex - sx;
        } else if dx < -(MAP_WIDTH as f32 / 2.0) {
            ex += MAP_WIDTH as f32;
            dx = ex - sx;
        }

        let mut dy = ey - sy;
        if dy > MAP_HEIGHT as f32 / 2.0 {
            ey -= MAP_HEIGHT as f32;
            dy = ey - sy;
        } else if dy < -(MAP_HEIGHT as f32 / 2.0) {
            ey += MAP_HEIGHT as f32;
            dy = ey - sy;
        }

        let base_x = sx + (ex - sx) * self.progress;
        let base_y = sy + (ey - sy) * self.progress;

        // Sidewalk Lane Offset (based on current lane_offset)
        let len = (dx*dx + dy*dy).sqrt();

        if len > 0.01 {
            let ndx = dx / len;
            let ndy = dy / len;
            let px = -ndy;
            let py = ndx;

            self.x = base_x + px * self.lane_offset;
            self.y = base_y + py * self.lane_offset;
        } else {
            self.x = base_x;
            self.y = base_y;
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

// Standalone Helper to pick the next tile, continuing straight and wrapping on the torus
fn pick_next_tile(map: &CityMap, rng_state: &mut u32, tx: usize, ty: usize, prev_tx: usize, prev_ty: usize) -> (usize, usize) {
    let mut dx = tx as i32 - prev_tx as i32;
    if dx > MAP_WIDTH as i32 / 2 {
        dx -= MAP_WIDTH as i32;
    } else if dx < -(MAP_WIDTH as i32 / 2) {
        dx += MAP_WIDTH as i32;
    }

    let mut dy = ty as i32 - prev_ty as i32;
    if dy > MAP_HEIGHT as i32 / 2 {
        dy -= MAP_HEIGHT as i32;
    } else if dy < -(MAP_HEIGHT as i32 / 2) {
        dy += MAP_HEIGHT as i32;
    }

    if dx == 0 && dy == 0 {
        // Spawn/start: pick a random direction that matches the sidewalk direction
        let neighbors = [
            (1, 0),
            (-1, 0),
            (0, 1),
            (0, -1),
        ];
        let mut candidates = Vec::new();
        for &(nx_offset, ny_offset) in neighbors.iter() {
            let nx = (tx as i32 + nx_offset).rem_euclid(MAP_WIDTH as i32) as usize;
            let ny = (ty as i32 + ny_offset).rem_euclid(MAP_HEIGHT as i32) as usize;
            let tile = map.grid[nx][ny];
            let is_valid = match tile {
                TileType::SidewalkVert => ny_offset != 0,
                TileType::SidewalkHoriz => nx_offset != 0,
                TileType::Intersection => true,
                _ => false,
            };
            if is_valid {
                candidates.push((nx, ny));
            }
        }
        if !candidates.is_empty() {
            let idx = (next_rng(rng_state) as usize) % candidates.len();
            return candidates[idx];
        }
        return (tx, ty);
    }

    // Walk straight all the time wrapping on the torus
    let next_tx = (tx as i32 + dx).rem_euclid(MAP_WIDTH as i32) as usize;
    let next_ty = (ty as i32 + dy).rem_euclid(MAP_HEIGHT as i32) as usize;
    (next_tx, next_ty)
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

    pub tx: usize,
    pub ty: usize,
    pub prev_tx: usize,
    pub prev_ty: usize,
    pub next_tx: usize,
    pub next_ty: usize,
    pub progress: f32,
    pub speed: f32,
    pub is_leftsider: bool,
    pub lane_offset: f32,
    pub view_angle: f32,
}

pub struct GameState {
    pub player: Player,
    pub citizens: Vec<Citizen>,
    pub lasers: Vec<LaserBeam>,
    pub floating_texts: Vec<FloatingText>,
    pub credits_flash: Option<(String, u32, f32)>, // Text, Color, Duration
    pub screen_shake: f32,
    pub map: CityMap,
    pub decals: Vec<BloodDecal>,
    pub particles: Vec<Particle>,
    pub focus_target_idx: Option<usize>,
    pub focus_text_timer: f32,
    pub is_in_menu: bool,
    pub menu_timer: f32,
    pub menu_selected_idx: usize,
    pub menu_particles: Vec<MenuParticle>,
    pub menu_title_landed: bool,
    // LCG Deterministic PRNG State
    rng_state: u32,
}

impl GameState {
    pub fn new() -> Self {
        let map = CityMap::new();
        
        // Spawn player on a walkable sidewalk tile at X=3.5, Y=2.5
        let tx = 3;
        let ty = 2;
        let next_tx = 3;
        let next_ty = 3;
        let player = Player {
            x: tx as f32 + 0.5,
            y: ty as f32 + 0.5,
            dir_x: 0.0,
            dir_y: 1.0,
            plane_x: -0.66,
            plane_y: 0.0,
            health: 100.0,
            shield: 100.0,
            battery: 100.0,
            credits: 1000,
            weapon_state: WeaponState::Idle,
            target_idx: None,
            damage_flash: 0.0,
            
            tx,
            ty,
            prev_tx: tx,
            prev_ty: ty,
            next_tx,
            next_ty,
            progress: 0.0,
            speed: 1.5,
            is_leftsider: false,
            lane_offset: 0.22,
            view_angle: std::f32::consts::FRAC_PI_2,
        };

        let state = Self {
            player,
            citizens: Vec::new(),
            lasers: Vec::new(),
            floating_texts: Vec::new(),
            credits_flash: None,
            screen_shake: 0.0,
            map,
            decals: Vec::new(),
            particles: Vec::new(),
            focus_target_idx: None,
            focus_text_timer: 0.0,
            is_in_menu: true,
            menu_timer: -3.5, // negative to offset for HTML preloader (~3.2s)
            menu_selected_idx: 0,
            menu_particles: Vec::new(),
            menu_title_landed: false,
            rng_state: 123456789,
        };

        // Notify JS menu is active
        update_menu_active_js(true);

        // Initial citizens will spawn dynamically in the update loop based on player visibility

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

    /// Spawn 3D blood droplets and meat debris
    pub fn spawn_blood_explosion(&mut self, x: f32, y: f32) {
        // Spawn blood sprinkles (droplets)
        let num_sprinkles = 15;
        for _ in 0..num_sprinkles {
            let theta = rng_float(&mut self.rng_state) * 2.0 * std::f32::consts::PI;
            let speed_h = 0.8 + rng_float(&mut self.rng_state) * 1.5;
            let vx = theta.cos() * speed_h;
            let vy = theta.sin() * speed_h;
            let vz = 1.0 + rng_float(&mut self.rng_state) * 2.0;
            let z = 0.1 + rng_float(&mut self.rng_state) * 0.4;
            
            self.particles.push(Particle {
                x,
                y,
                z,
                vx,
                vy,
                vz,
                p_type: ParticleType::BloodSprinkle,
                bounces: 1,
                lifetime: 0.6 + rng_float(&mut self.rng_state) * 0.6,
                first_impact: true,
            });
        }

        // Spawn gore chunks (red meaty chunks)
        let num_chunks = 6;
        for _ in 0..num_chunks {
            let theta = rng_float(&mut self.rng_state) * 2.0 * std::f32::consts::PI;
            let speed_h = 0.4 + rng_float(&mut self.rng_state) * 1.0;
            let vx = theta.cos() * speed_h;
            let vy = theta.sin() * speed_h;
            let vz = 1.5 + rng_float(&mut self.rng_state) * 2.5;
            let z = 0.1 + rng_float(&mut self.rng_state) * 0.4;
            
            self.particles.push(Particle {
                x,
                y,
                z,
                vx,
                vy,
                vz,
                p_type: ParticleType::GoreDebris,
                bounces: 3 + (next_rng(&mut self.rng_state) % 3), // Bounces 3-5 times
                lifetime: 1.5 + rng_float(&mut self.rng_state) * 1.5,
                first_impact: true,
            });
        }
    }

    /// Primary game state update loop
    pub fn update(&mut self, dt: f32) {
        if self.is_in_menu {
            self.menu_timer += dt;
        }

        // Focus scan window typing animation update
        if self.player.target_idx != self.focus_target_idx {
            self.focus_target_idx = self.player.target_idx;
            self.focus_text_timer = 0.0;
        } else {
            self.focus_text_timer += dt;
        }

        // Update player auto-movement and camera orientation
        if !self.is_in_menu && self.player.health > 0.0 {
            // Smoothly interpolate lane_offset
            let target_offset = if self.player.is_leftsider { -0.22 } else { 0.22 };
            self.player.lane_offset += (target_offset - self.player.lane_offset) * 8.0 * dt;

            // Move player along their path
            self.player.progress += self.player.speed * dt;
            if self.player.progress >= 1.0 {
                let old_prev_x = self.player.tx;
                let old_prev_y = self.player.ty;
                self.player.tx = self.player.next_tx;
                self.player.ty = self.player.next_ty;
                self.player.prev_tx = old_prev_x;
                self.player.prev_ty = old_prev_y;

                // Pick next tile
                let mut temp_rng = self.rng_state;
                let (nx, ny) = pick_next_tile(&self.map, &mut temp_rng, self.player.tx, self.player.ty, self.player.prev_tx, self.player.prev_ty);
                self.rng_state = temp_rng;

                self.player.next_tx = nx;
                self.player.next_ty = ny;
                self.player.progress = 0.0;
            }

            // Align visual coordinates
            self.player.align_position();

            // Smooth camera rotation around corners (with torus wrap-around handling)
            let mut dx = self.player.next_tx as f32 - self.player.tx as f32;
            if dx > MAP_WIDTH as f32 / 2.0 { dx -= MAP_WIDTH as f32; }
            else if dx < -(MAP_WIDTH as f32 / 2.0) { dx += MAP_WIDTH as f32; }

            let mut dy = self.player.next_ty as f32 - self.player.ty as f32;
            if dy > MAP_HEIGHT as f32 / 2.0 { dy -= MAP_HEIGHT as f32; }
            else if dy < -(MAP_HEIGHT as f32 / 2.0) { dy += MAP_HEIGHT as f32; }

            let len = (dx*dx + dy*dy).sqrt();
            if len > 0.01 {
                let target_angle = dy.atan2(dx);
                let diff = (target_angle - self.player.view_angle + std::f32::consts::PI).rem_euclid(2.0 * std::f32::consts::PI) - std::f32::consts::PI;
                self.player.view_angle += diff * 5.0 * dt;
            }

            // Update direction and plane
            self.player.dir_x = self.player.view_angle.cos();
            self.player.dir_y = self.player.view_angle.sin();
            self.player.plane_x = -self.player.dir_y * 0.66;
            self.player.plane_y = self.player.dir_x * 0.66;
        }

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

        // Update particles
        let gravity = 8.5;
        let map_w = MAP_WIDTH as f32;
        let map_h = MAP_HEIGHT as f32;
        
        let mut new_decals = Vec::new();

        self.particles.retain_mut(|p| {
            p.lifetime -= dt;
            if p.lifetime <= 0.0 {
                return false;
            }

            p.vz -= gravity * dt;

            // Physics step
            let prev_x = p.x;
            let prev_y = p.y;
            let mut next_x = (p.x + p.vx * dt).rem_euclid(map_w);
            let mut next_y = (p.y + p.vy * dt).rem_euclid(map_h);

            // Wall collision check
            if self.map.is_solid(next_x, next_y) {
                // If moving diagonally, try sliding on X or Y
                let slide_x = (p.x + p.vx * dt).rem_euclid(map_w);
                let slide_y = (p.y + p.vy * dt).rem_euclid(map_h);
                if !self.map.is_solid(slide_x, prev_y) {
                    next_x = slide_x;
                    next_y = prev_y;
                    p.vy = -p.vy * 0.4;
                } else if !self.map.is_solid(prev_x, slide_y) {
                    next_x = prev_x;
                    next_y = slide_y;
                    p.vx = -p.vx * 0.4;
                } else {
                    // Total bounce back
                    p.vx = -p.vx * 0.4;
                    p.vy = -p.vy * 0.4;
                    next_x = prev_x;
                    next_y = prev_y;
                }
            }

            p.x = next_x;
            p.y = next_y;
            p.z += p.vz * dt;

            // Floor collision
            if p.z <= 0.0 {
                p.z = 0.0;
                
                // Spawn a blood decal on floor contact with reduced probability
                let is_sprinkle = p.p_type == ParticleType::BloodSprinkle;
                let should_spawn = if is_sprinkle {
                    // 30% chance for sprinkles
                    (next_rng(&mut self.rng_state) % 10) < 3
                } else {
                    // 50% chance only on the first impact for chunks
                    let spawn = p.first_impact && (next_rng(&mut self.rng_state) % 2 == 0);
                    p.first_impact = false;
                    spawn
                };

                if should_spawn {
                    let decal_radius = if is_sprinkle {
                        0.05 + rng_float(&mut self.rng_state) * 0.12
                    } else {
                        0.12 + rng_float(&mut self.rng_state) * 0.18
                    };
                    
                    new_decals.push(BloodDecal {
                        x: p.x,
                        y: p.y,
                        radius: decal_radius,
                        lifetime: 10.0 + rng_float(&mut self.rng_state) * 10.0,
                    });
                } else if !is_sprinkle {
                    p.first_impact = false; // ensure it's marked as false even if not spawned
                }

                if p.bounces > 0 {
                    p.bounces -= 1;
                    p.vz = -p.vz * 0.45; // Restitution
                    p.vx *= 0.5; // Friction
                    p.vy *= 0.5;
                } else {
                    p.vz = 0.0;
                    p.vx = 0.0;
                    p.vy = 0.0;
                }
            }

            true
        });

        // Add new decals and enforce limit of 512 decals to prevent slowdowns
        for decal in new_decals {
            self.decals.push(decal);
        }
        if self.decals.len() > 512 {
            let excess = self.decals.len() - 512;
            self.decals.drain(0..excess);
        }

        // Update decals lifetime
        self.decals.retain_mut(|decal| {
            decal.lifetime -= dt;
            decal.lifetime > 0.0
        });

        // Update credits flash banner
        if let Some((_, _, ref mut duration)) = self.credits_flash {
            *duration -= dt;
            if *duration <= 0.0 {
                self.credits_flash = None;
            }
        }

        // ------------------------------------------
        // DYNAMIC SPARK & DESPAWN CONE FOR CITIZENS
        // ------------------------------------------
        let px = self.player.x;
        let py = self.player.y;
        let pdx = self.player.dir_x;
        let pdy = self.player.dir_y;

        // Despawn citizens that are too far away or behind the player, or dead too long
        self.citizens.retain(|citizen| {
            if citizen.state == CitizenState::Dead && citizen.shoot_cooldown <= -8.0 {
                return false;
            }

            // Keep exploding citizens so they finish their animations
            if citizen.state != CitizenState::Walking && citizen.state != CitizenState::Dead {
                return true;
            }

            let mut dx = citizen.x - px;
            if dx > MAP_WIDTH as f32 / 2.0 { dx -= MAP_WIDTH as f32; }
            else if dx < -(MAP_WIDTH as f32 / 2.0) { dx += MAP_WIDTH as f32; }

            let mut dy = citizen.y - py;
            if dy > MAP_HEIGHT as f32 / 2.0 { dy -= MAP_HEIGHT as f32; }
            else if dy < -(MAP_HEIGHT as f32 / 2.0) { dy += MAP_HEIGHT as f32; }

            let dist = (dx*dx + dy*dy).sqrt();

            if dist > 18.0 {
                return false;
            }

            let dot = dx * pdx + dy * pdy;
            if dot < -2.0 && dist > 3.0 {
                return false;
            }

            true
        });

        // Count visible citizens
        let mut visible_count = 0;
        for citizen in &self.citizens {
            if citizen.state == CitizenState::Walking {
                let mut dx = citizen.x - px;
                if dx > MAP_WIDTH as f32 / 2.0 { dx -= MAP_WIDTH as f32; }
                else if dx < -(MAP_WIDTH as f32 / 2.0) { dx += MAP_WIDTH as f32; }

                let mut dy = citizen.y - py;
                if dy > MAP_HEIGHT as f32 / 2.0 { dy -= MAP_HEIGHT as f32; }
                else if dy < -(MAP_HEIGHT as f32 / 2.0) { dy += MAP_HEIGHT as f32; }

                let dist = (dx*dx + dy*dy).sqrt();
                if dist < 16.0 {
                    let dot = dx * pdx + dy * pdy;
                    if dot > 0.0 {
                        visible_count += 1;
                    }
                }
            }
        }

        // Spawn new citizens to keep the visibility field populated
        let target_visible = 28;
        let mut spawn_attempts = 0;
        let player_is_vert = pdy.abs() > pdx.abs();

        while visible_count < target_visible && spawn_attempts < 15 {
            spawn_attempts += 1;
            
            let p_tile_x = self.player.tx as i32;
            let p_tile_y = self.player.ty as i32;
            let center_x = p_tile_x + (pdx * 10.0) as i32;
            let center_y = p_tile_y + (pdy * 10.0) as i32;

            let mut same_side_candidates = Vec::new();
            let mut other_candidates = Vec::new();

            for gx_raw in (center_x - 12)..=(center_x + 12) {
                for gy_raw in (center_y - 12)..=(center_y + 12) {
                    let gx = gx_raw.rem_euclid(MAP_WIDTH as i32) as usize;
                    let gy = gy_raw.rem_euclid(MAP_HEIGHT as i32) as usize;

                    let tile = self.map.grid[gx][gy];
                    match tile {
                        TileType::SidewalkVert | TileType::SidewalkHoriz | TileType::Intersection => {
                            let mut tdx = gx as f32 + 0.5 - px;
                            if tdx > MAP_WIDTH as f32 / 2.0 { tdx -= MAP_WIDTH as f32; }
                            else if tdx < -(MAP_WIDTH as f32 / 2.0) { tdx += MAP_WIDTH as f32; }

                            let mut tdy = gy as f32 + 0.5 - py;
                            if tdy > MAP_HEIGHT as f32 / 2.0 { tdy -= MAP_HEIGHT as f32; }
                            else if tdy < -(MAP_HEIGHT as f32 / 2.0) { tdy += MAP_HEIGHT as f32; }

                            let dist = (tdx*tdx + tdy*tdy).sqrt();
                            if dist >= 7.0 && dist <= 16.0 {
                                let dot = tdx * pdx + tdy * pdy;
                                if dot > 0.5 { // ~120 degree cone in front of player
                                    let mut occupied = false;
                                    for c in &self.citizens {
                                        if (c.tx % MAP_WIDTH == gx && c.ty % MAP_HEIGHT == gy) || 
                                           (c.next_tx % MAP_WIDTH == gx && c.next_ty % MAP_HEIGHT == gy) {
                                            occupied = true;
                                            break;
                                        }
                                    }
                                    if !occupied {
                                        let is_same = if player_is_vert {
                                            gx == self.player.tx % MAP_WIDTH
                                        } else {
                                            gy == self.player.ty % MAP_HEIGHT
                                        };
                                        if is_same {
                                            same_side_candidates.push((gx_raw, gy_raw));
                                        } else {
                                            other_candidates.push((gx_raw, gy_raw));
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }

            let rolled_same = (next_rng(&mut self.rng_state) % 100) < 70;
            let chosen_candidates = if rolled_same && !same_side_candidates.is_empty() {
                &same_side_candidates
            } else if !other_candidates.is_empty() {
                &other_candidates
            } else {
                &same_side_candidates
            };

            if !chosen_candidates.is_empty() {
                let mut temp_rng = self.rng_state;
                let idx = (next_rng(&mut temp_rng) as usize) % chosen_candidates.len();
                let (sx_raw, sy_raw) = chosen_candidates[idx];
                self.rng_state = temp_rng;

                self.spawn_citizen_at(sx_raw as usize, sy_raw as usize, self.citizens.len());
                visible_count += 1;
            } else {
                break;
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
            let mut explode_done = false;

            {
                let player_x = self.player.x;
                let player_y = self.player.y;
                let citizen = &mut self.citizens[i];

                match citizen.state {
                    CitizenState::Walking => {
                        if citizen.is_rebel && !self.is_in_menu {
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

            // (Respawn managed dynamically by visibility spawner/despawner)
        }

        // Recalculate player scanner target
        if self.is_in_menu {
            self.player.target_idx = None;
        } else {
            self.update_scanner_target();
        }
    }

    /// Finds the closest citizen directly under the player's crosshair (only on player's own side and lane)
    fn update_scanner_target(&mut self) {
        let mut closest_idx = None;
        let mut min_dist = 15.0; // Max scanning distance (aligned with fog visibility)

        let player_is_vert = self.player.dir_y.abs() > self.player.dir_x.abs();

        for (idx, citizen) in self.citizens.iter().enumerate() {
            if citizen.state != CitizenState::Walking {
                continue;
            }

            // Must walk in the same orientation (vertical or horizontal)
            let citizen_is_vert = (citizen.next_tx as i32 - citizen.tx as i32).abs() < (citizen.next_ty as i32 - citizen.ty as i32).abs();
            if player_is_vert != citizen_is_vert {
                continue;
            }

            // Must be on the same sidewalk corridor
            let same_sidewalk = if player_is_vert {
                (citizen.tx % MAP_WIDTH) == (self.player.tx % MAP_WIDTH)
            } else {
                (citizen.ty % MAP_HEIGHT) == (self.player.ty % MAP_HEIGHT)
            };
            if !same_sidewalk {
                continue;
            }

            // Must be on the same spatial lane (on screen)
            let same_lane = if player_is_vert {
                let player_is_left_spatial = self.player.x < (self.player.tx as f32 + 0.5);
                let citizen_is_left_spatial = citizen.x < (citizen.tx as f32 + 0.5);
                player_is_left_spatial == citizen_is_left_spatial
            } else {
                let player_is_top_spatial = self.player.y < (self.player.ty as f32 + 0.5);
                let citizen_is_top_spatial = citizen.y < (citizen.ty as f32 + 0.5);
                player_is_top_spatial == citizen_is_top_spatial
            };
            if !same_lane {
                continue;
            }

            let mut dx = citizen.x - self.player.x;
            if dx > MAP_WIDTH as f32 / 2.0 { dx -= MAP_WIDTH as f32; }
            else if dx < -(MAP_WIDTH as f32 / 2.0) { dx += MAP_WIDTH as f32; }

            let mut dy = citizen.y - self.player.y;
            if dy > MAP_HEIGHT as f32 / 2.0 { dy -= MAP_HEIGHT as f32; }
            else if dy < -(MAP_HEIGHT as f32 / 2.0) { dy += MAP_HEIGHT as f32; }

            // Check if citizen is in front of the player along our walking direction
            let is_in_front = if player_is_vert {
                dy * self.player.dir_y > 0.0
            } else {
                dx * self.player.dir_x > 0.0
            };
            if !is_in_front {
                continue;
            }

            let dist = (dx*dx + dy*dy).sqrt();
            if dist < min_dist {
                min_dist = dist;
                closest_idx = Some(idx);
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
            if target_idx < self.citizens.len() {
                let mut target_died = false;
                let mut tx = 0.0;
                let mut ty = 0.0;
                
                {
                    let target = &mut self.citizens[target_idx];
                    beam_end_x = target.x;
                    beam_end_y = target.y;

                    // Damage target
                    target.health -= 100.0;
                    if target.health <= 0.0 {
                        // Kill target!
                        target.state = CitizenState::Exploding(0.0);
                        target.shoot_cooldown = 0.0; // reset as respawn timer
                        target_died = true;
                        tx = target.x;
                        ty = target.y;

                        // Check compliance for reward/penalty
                        let is_lefty = target.is_leftsider;
                        let is_rebel = target.is_rebel;

                        if is_lefty || is_rebel {
                            // Correct elimination of criminal
                            let reward = 1000;
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
                                format!("COLLATERAL DAMAGE // -{} CR", penalty),
                                0xff007fff,
                                1.5
                            ));
                        }
                    }
                }

                if target_died {
                    self.spawn_blood_explosion(tx, ty);
                }
            } else {
                beam_end_x = self.player.x + self.player.dir_x * 8.0;
                beam_end_y = self.player.y + self.player.dir_y * 8.0;
            }
        } else {
            // Missed shoot, shoot straight into wall in player direction
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

    /// Process player lane switching inputs
    pub fn move_player(&mut self, switch_left: bool, switch_right: bool) {
        if self.player.health <= 0.0 {
            return;
        }
        if switch_left {
            self.player.is_leftsider = true;
        }
        if switch_right {
            self.player.is_leftsider = false;
        }
    }
}
