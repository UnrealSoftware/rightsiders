use crate::map::{CityMap, TileType, MAP_WIDTH, MAP_HEIGHT};
use crate::raycaster::{WIDTH, HEIGHT};

#[cfg(target_arch = "wasm32")]
unsafe extern "C" {
    fn play_sfx(ptr: *const u8, len: usize);
    fn set_menu_active(active: bool);
    fn load_scores_js(ptr: *mut u8, max_len: usize) -> usize;
    fn save_scores_js(ptr: *const u8, len: usize);
    fn is_game_started_js() -> bool;
    fn set_entering_highscore_js(entering: bool);
    fn get_mobile_highscore_name_js(ptr: *mut u8, max_len: usize) -> usize;
    fn is_mobile_highscore_submitted_js() -> bool;
    fn clear_mobile_highscore_submit_js();
    fn is_mobile_js() -> bool;
    fn js_get_switch_lane_left_js() -> bool;
    fn js_get_switch_lane_right_js() -> bool;
    fn js_get_trigger_fire_js() -> bool;
    fn js_get_trigger_missile_js() -> bool;
    fn open_privacy_modal_js();
    fn open_music_modal_js();
    fn toggle_help_js();
    fn hide_help_js();
    fn toggle_fullscreen_js();
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

pub fn is_game_started() -> bool {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        is_game_started_js()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        true
    }
}

pub fn set_entering_highscore(entering: bool) {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        set_entering_highscore_js(entering);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        println!("[entering_highscore] {}", entering);
    }
}

pub fn get_mobile_highscore_name(buf: &mut [u8]) -> usize {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        get_mobile_highscore_name_js(buf.as_mut_ptr(), buf.len())
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = buf;
        0
    }
}

pub fn is_mobile_highscore_submitted() -> bool {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        is_mobile_highscore_submitted_js()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        false
    }
}

pub fn clear_mobile_highscore_submit() {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        clear_mobile_highscore_submit_js();
    }
}

pub fn is_mobile() -> bool {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        is_mobile_js()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        false
    }
}

pub fn js_get_switch_lane_left() -> bool {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        js_get_switch_lane_left_js()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        false
    }
}

pub fn js_get_switch_lane_right() -> bool {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        js_get_switch_lane_right_js()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        false
    }
}

pub fn js_get_trigger_fire() -> bool {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        js_get_trigger_fire_js()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        false
    }
}

pub fn js_get_trigger_missile() -> bool {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        js_get_trigger_missile_js()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        false
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
    Smoke,
    Steam,
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
pub struct MenuShockwave {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    #[allow(dead_code)]
    pub max_radius: f32,
    pub speed: f32,
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub thickness: f32,
    pub color_r: u8,
    pub color_g: u8,
    pub color_b: u8,
}

#[derive(Clone)]
pub struct RainDrop {
    pub x: f32,
    pub y: f32,
    pub speed: f32,
    pub length: f32,
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
    pub tex_idx: usize,
    pub angle: f32,
    pub spin_speed: f32,
}

#[derive(Clone)]
pub struct ScreenBlood {
    pub x: f32,
    pub y: f32,
    pub start_y: f32,
    pub vy: f32,
    pub size: f32,
    pub lifetime: f32,
    pub max_lifetime: f32,
}

pub struct Vehicle {
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
    pub sprite_idx: usize,
    pub hover_offset: f32,
    pub hover_speed: f32,
    pub has_played_driveby: bool,
    pub age: f32,
}

impl Vehicle {
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

        self.base_x = sx + (ex - sx) * self.progress;
        self.base_y = sy + (ey - sy) * self.progress;

        // Right-lane offset (vehicles are always compliant rightsiders driving on the road)
        let len = (dx*dx + dy*dy).sqrt();
        if len > 0.01 {
            let ndx = dx / len;
            let ndy = dy / len;
            
            // Left-normal for lane offset
            let px = -ndy;
            let py = ndx;
            
            // Off-center driving offset for street lanes
            let offset_dist = 0.28;
            self.x = self.base_x + px * offset_dist;
            self.y = self.base_y + py * offset_dist;
        } else {
            self.x = self.base_x;
            self.y = self.base_y;
        }
    }
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
    pub lane_offset: f32,
    pub health: f32,
    pub state: CitizenState,
    pub shoot_cooldown: f32,
    pub name: String,
    pub id_num: String,
    pub walk_frame: usize, // Animation frame (0 or 1)
}

impl Citizen {
    pub fn is_visually_leftsider(&self) -> bool {
        self.lane_offset < 0.0
    }

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

            let mult = self.lane_offset;

            // Right-normal for passing offset
            let rx = -ndy;
            let ry = ndx;
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
    pub z: f32, // 3D world z (height)
    pub is_hud: bool, // Render relative to crosshair
    pub color: u32,
    pub duration: f32,
}

/// A guided missile in flight
pub struct GuidedMissile {
    pub x: f32,
    pub y: f32,
    pub z: f32,       // Height in world units
    pub vx: f32,
    pub vy: f32,
    pub vz: f32,
    pub target_id: String,
    pub target_x: f32,
    pub target_y: f32,
    pub flight_time: f32,    // Elapsed since launch
    pub steer_delay: f32,    // Time before homing kicks in
    pub total_flight: f32,   // Total planned flight duration
}

pub struct Player {
    pub x: f32,
    pub y: f32,
    pub dir_x: f32,
    pub dir_y: f32,
    pub plane_x: f32,
    pub plane_y: f32,
    
    pub health: f32,
    #[allow(dead_code)]
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
    pub menu_shockwaves: Vec<MenuShockwave>,
    pub menu_title_landed: bool,
    pub menu_star_played: bool,
    pub slogan_chars_played: usize,
    pub time_left: f32,
    pub is_entering_highscore: bool,
    pub highscore_name: String,
    pub highscore_input_delay: f32,
    pub last_beep_second: i32,
    pub show_leaderboard: bool,
    pub show_directives: bool,
    pub directives_stage: usize,
    pub directives_timer: f32,
    pub leaderboard_data: Vec<(String, i32)>,
    pub leaderboard_open_time: f64,
    pub new_rank: Option<usize>,
    pub offenders_killed_laser: u32,
    pub offenders_killed_rocket: u32,
    pub collateral_damage_kills: u32,
    pub is_showing_summary: bool,
    pub summary_timer: f32,
    pub summary_stage: usize,
    pub summary_count_anim: f32,
    pub summary_skip_buildup: bool,
    // Guided missile special attack
    pub missiles: Vec<GuidedMissile>,
    pub missile_used: bool,
    pub vehicles: Vec<Vehicle>,
    pub rain_drops: Vec<RainDrop>,
    pub screen_blood: Vec<ScreenBlood>,
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
            speed: 3.0,
            is_leftsider: false,
            lane_offset: 0.22,
            view_angle: std::f32::consts::FRAC_PI_2,
        };

        let mut state = Self {
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
            menu_timer: 0.0,
            menu_selected_idx: 0,
            menu_particles: Vec::new(),
            menu_shockwaves: Vec::new(),
            menu_title_landed: false,
            menu_star_played: false,
            slogan_chars_played: 0,
            time_left: 30.0,
            is_entering_highscore: false,
            highscore_name: String::new(),
            highscore_input_delay: 0.0,
            offenders_killed_laser: 0,
            offenders_killed_rocket: 0,
            collateral_damage_kills: 0,
            is_showing_summary: false,
            summary_timer: 0.0,
            summary_stage: 0,
            summary_count_anim: 0.0,
            summary_skip_buildup: false,
            last_beep_second: 6,
            show_leaderboard: false,
            show_directives: false,
            directives_stage: 0,
            directives_timer: 0.0,
            leaderboard_data: Vec::new(),
            leaderboard_open_time: 0.0,
            new_rank: None,
            missiles: Vec::new(),
            missile_used: false,
            vehicles: Vec::new(),
            rain_drops: Vec::new(),
            screen_blood: Vec::new(),
            rng_state: 123456789,
        };

        // Initialize rain drops using deterministic RNG
        for _ in 0..120 {
            let rx = rng_float(&mut state.rng_state) * WIDTH as f32;
            let ry = rng_float(&mut state.rng_state) * HEIGHT as f32;
            let speed = 400.0 + rng_float(&mut state.rng_state) * 200.0;
            let length = 10.0 + rng_float(&mut state.rng_state) * 8.0;
            state.rain_drops.push(RainDrop {
                x: rx,
                y: ry,
                speed,
                length,
            });
        }

        // Notify JS menu is active
        update_menu_active_js(true);
        set_entering_highscore(false);

        // Initial citizens will spawn dynamically in the update loop based on player visibility

        state
    }


    pub fn next_random(&mut self) -> u32 {
        self.rng_state = self.rng_state.wrapping_mul(1103515245).wrapping_add(12345);
        self.rng_state
    }

    pub fn random_float(&mut self) -> f32 {
        (self.next_random() as f32) / (u32::MAX as f32)
    }

    /// Spawn a citizen at a given tile
    pub fn spawn_citizen_at(&mut self, tx: usize, ty: usize, index: usize) {
        let val = next_rng(&mut self.rng_state);
        
        // Compliance profile:
        // - 40% Leftsider violators (is_rebel = false, is_leftsider = true)
        // - 60% Compliant Rightsiders (is_rebel = false, is_leftsider = false)
        let roll = val % 100;
        let is_leftsider = roll < 40;

        // Generate names
        let name_prefix = "CITIZEN-";
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
            lane_offset: if is_leftsider { -0.22 } else { 0.22 },
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

    pub fn spawn_vehicle_at(&mut self, tx: usize, ty: usize) {
        let val = next_rng(&mut self.rng_state);
        let mut dx_to_player = self.player.x - (tx as f32 + 0.5);
        if dx_to_player > MAP_WIDTH as f32 / 2.0 { dx_to_player -= MAP_WIDTH as f32; }
        else if dx_to_player < -(MAP_WIDTH as f32 / 2.0) { dx_to_player += MAP_WIDTH as f32; }

        let mut dy_to_player = self.player.y - (ty as f32 + 0.5);
        if dy_to_player > MAP_HEIGHT as f32 / 2.0 { dy_to_player -= MAP_HEIGHT as f32; }
        else if dy_to_player < -(MAP_HEIGHT as f32 / 2.0) { dy_to_player += MAP_HEIGHT as f32; }

        let dot = (tx as f32 + 0.5 - self.player.x) * self.player.dir_x + (ty as f32 + 0.5 - self.player.y) * self.player.dir_y;
        let is_behind = dot < 0.0;

        // Determine direction based on road type
        let (dx, dy) = if tx % 7 == 4 && ty % 7 == 4 {
            // Intersection
            if is_behind {
                if dx_to_player.abs() > dy_to_player.abs() {
                    (if dx_to_player > 0.0 { 1 } else { -1 }, 0)
                } else {
                    (0, if dy_to_player > 0.0 { 1 } else { -1 })
                }
            } else {
                match val % 4 {
                    0 => (0, 1),
                    1 => (0, -1),
                    2 => (1, 0),
                    _ => (-1, 0),
                }
            }
        } else if tx % 7 == 4 {
            // Vertical road
            if is_behind {
                (0, if dy_to_player > 0.0 { 1 } else { -1 })
            } else {
                if val % 2 == 0 { (0, 1) } else { (0, -1) }
            }
        } else {
            // Horizontal road
            if is_behind {
                (if dx_to_player > 0.0 { 1 } else { -1 }, 0)
            } else {
                if val % 2 == 0 { (1, 0) } else { (-1, 0) }
            }
        };

        let next_tx = (tx as i32 + dx).rem_euclid(MAP_WIDTH as i32) as usize;
        let next_ty = (ty as i32 + dy).rem_euclid(MAP_HEIGHT as i32) as usize;

        // Speed: hover vehicles should travel faster than walking citizens (0.6 - 1.2)
        let speed = 1.8 + rng_float(&mut self.rng_state) * 1.4;

        let sprite_idx = if val % 2 == 0 { 13 } else { 14 };
        let hover_speed = 3.0 + rng_float(&mut self.rng_state) * 2.0;

        let mut vehicle = Vehicle {
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
            sprite_idx,
            hover_offset: rng_float(&mut self.rng_state) * 2.0 * std::f32::consts::PI,
            hover_speed,
            has_played_driveby: false,
            age: 0.0,
        };
        vehicle.align_position();
        self.vehicles.push(vehicle);
    }

    /// Spawn 3D blood droplets and meat debris
    pub fn spawn_blood_explosion(&mut self, x: f32, y: f32) {
        play_sound("blood_explosion");
        // Project explosion to screen if close in front of player
        let mut dx = x - self.player.x;
        if dx > MAP_WIDTH as f32 / 2.0 { dx -= MAP_WIDTH as f32; }
        else if dx < -(MAP_WIDTH as f32 / 2.0) { dx += MAP_WIDTH as f32; }

        let mut dy = y - self.player.y;
        if dy > MAP_HEIGHT as f32 / 2.0 { dy -= MAP_HEIGHT as f32; }
        else if dy < -(MAP_HEIGHT as f32 / 2.0) { dy += MAP_HEIGHT as f32; }

        let dist_sq = dx * dx + dy * dy;
        let dot = dx * self.player.dir_x + dy * self.player.dir_y;
        let dist = dist_sq.sqrt();
        if dot > 0.0 && dist < 3.0 {
            let inv_det = 1.0 / (self.player.plane_x * self.player.dir_y - self.player.dir_x * self.player.plane_y);
            let transform_x = inv_det * (self.player.dir_y * dx - self.player.dir_x * dy);
            let transform_y = inv_det * (-self.player.plane_y * dx + self.player.plane_x * dy);
            if transform_y > 0.01 {
                // Subtle fast screen shake and wet splat sound when blood hits player visor
                self.screen_shake = (self.screen_shake + 0.15).min(0.22);
                play_sound("visor_splat");

                let screen_x = (240.0 * (1.0 + transform_x / transform_y)) as f32;
                
                let num_splatters = if dist < 1.0 {
                    8
                } else if dist < 2.0 {
                    5
                } else {
                    3
                };

                for _ in 0..num_splatters {
                    let x_offset = (rng_float(&mut self.rng_state) - 0.5) * 200.0;
                    let sx = (screen_x + x_offset).clamp(20.0, 460.0);
                    let sy = 30.0 + rng_float(&mut self.rng_state) * 150.0;
                    
                    let vy = 15.0 + rng_float(&mut self.rng_state) * 35.0;
                    let size = 4.0 + rng_float(&mut self.rng_state) * 12.0;
                    let max_lifetime = 1.5 + rng_float(&mut self.rng_state) * 2.0;

                    self.screen_blood.push(ScreenBlood {
                        x: sx,
                        y: sy,
                        start_y: sy,
                        vy,
                        size,
                        lifetime: max_lifetime,
                        max_lifetime,
                    });
                }
            }
        }

        // Spawn blood sprinkles (droplets)
        let num_sprinkles = 45;
        for i in 0..num_sprinkles {
            let (vx, vy, vz) = if i % 3 == 0 {
                // 1 in 3 fall straight down and hit the ground almost instantly
                (0.0, 0.0, -3.0 - rng_float(&mut self.rng_state) * 4.0)
            } else {
                let theta = rng_float(&mut self.rng_state) * 2.0 * std::f32::consts::PI;
                let speed_h = 2.0 + rng_float(&mut self.rng_state) * 4.5; // High speed scatter (faster)
                let vx = theta.cos() * speed_h;
                let vy = theta.sin() * speed_h;
                let vz = 2.2 + rng_float(&mut self.rng_state) * 4.3;     // Higher vertical splash (faster)
                (vx, vy, vz)
            };
            let z = 0.25 + rng_float(&mut self.rng_state) * 0.25;    // Chest level spawn
            
            self.particles.push(Particle {
                x,
                y,
                z,
                vx,
                vy,
                vz,
                p_type: ParticleType::BloodSprinkle,
                bounces: 1 + (next_rng(&mut self.rng_state) % 3), // Bounces 1-3 times
                lifetime: 0.6 + rng_float(&mut self.rng_state) * 1.0, // Shorter lifetime for faster dissipation
                first_impact: true,
                tex_idx: 7, // frame 8 (index 7)
                angle: 0.0,
                spin_speed: 0.0,
            });
        }

        // Spawn gore chunks (red meaty chunks)
        let num_chunks = 20;
        for i in 0..num_chunks {
            let (vx, vy, vz) = if i % 4 == 0 {
                // 1 in 4 fall straight down
                (0.0, 0.0, -4.0 - rng_float(&mut self.rng_state) * 5.0)
            } else {
                let theta = rng_float(&mut self.rng_state) * 2.0 * std::f32::consts::PI;
                let speed_h = 1.5 + rng_float(&mut self.rng_state) * 3.5; // Far flung chunks (faster)
                let vx = theta.cos() * speed_h;
                let vy = theta.sin() * speed_h;
                let vz = 3.5 + rng_float(&mut self.rng_state) * 5.5;     // Volcanic ejection upward (faster)
                (vx, vy, vz)
            };
            let z = 0.25 + rng_float(&mut self.rng_state) * 0.25;    // Chest level spawn
            
            let tex_idx = if next_rng(&mut self.rng_state) % 100 < 20 {
                4 // 20% chance of frame 5 (index 4)
            } else {
                8 // 80% chance of frame 9 (index 8)
            };

            let (angle, spin_speed) = if tex_idx == 4 {
                let start_angle = rng_float(&mut self.rng_state) * 2.0 * std::f32::consts::PI;
                let speed = 6.0 + rng_float(&mut self.rng_state) * 12.0;
                let dir = if next_rng(&mut self.rng_state) % 2 == 0 { 1.0 } else { -1.0 };
                (start_angle, speed * dir)
            } else {
                (0.0, 0.0)
            };

            self.particles.push(Particle {
                x,
                y,
                z,
                vx,
                vy,
                vz,
                p_type: ParticleType::GoreDebris,
                bounces: 4 + (next_rng(&mut self.rng_state) % 4), // Bounces 4-7 times
                lifetime: 1.2 + rng_float(&mut self.rng_state) * 1.8, // Shorter lifetime for faster dissipation
                first_impact: true,
                tex_idx,
                angle,
                spin_speed,
            });
        }
    }


    /// Launch guided missiles at all visible leftsiders
    pub fn trigger_missile_salvo(&mut self) {
        if self.missile_used || self.player.health <= 0.0 {
            return;
        }

        let px = self.player.x;
        let py = self.player.y;
        let pdx = self.player.dir_x;
        let pdy = self.player.dir_y;

        // Collect visible leftsider targets
        let mut targets: Vec<(usize, f32, f32)> = Vec::new();
        for (idx, citizen) in self.citizens.iter().enumerate() {
            if citizen.state != CitizenState::Walking || !citizen.is_visually_leftsider() {
                continue;
            }
            let mut dx = citizen.x - px;
            if dx > MAP_WIDTH as f32 / 2.0 { dx -= MAP_WIDTH as f32; }
            else if dx < -(MAP_WIDTH as f32 / 2.0) { dx += MAP_WIDTH as f32; }
            let mut dy = citizen.y - py;
            if dy > MAP_HEIGHT as f32 / 2.0 { dy -= MAP_HEIGHT as f32; }
            else if dy < -(MAP_HEIGHT as f32 / 2.0) { dy += MAP_HEIGHT as f32; }
            let dist = (dx * dx + dy * dy).sqrt();
            let dot = dx * pdx + dy * pdy;
            if dist < 16.0 && dot > 0.0 {
                targets.push((idx, citizen.x, citizen.y));
            }
        }

        if targets.is_empty() {
            return; // Nothing to shoot at – don't consume cooldown
        }

        // Launch one missile per target
        for (idx, tx, ty) in &targets {
            // Randomize flight duration: 0.4 to 1.0 s
            let rand_a = rng_float(&mut self.rng_state);
            let rand_b = rng_float(&mut self.rng_state);
            let rand_c = rng_float(&mut self.rng_state);
            let total_flight = 0.4 + rand_a * 0.6;
            // Steer kicks in after 35% of flight time to allow arcing up into the sky first
            let steer_delay = total_flight * 0.35;

            // Calculate distance to target to set appropriate base speed
            let mut dx = *tx - px;
            if dx > MAP_WIDTH as f32 / 2.0 { dx -= MAP_WIDTH as f32; }
            else if dx < -(MAP_WIDTH as f32 / 2.0) { dx += MAP_WIDTH as f32; }
            let mut dy = *ty - py;
            if dy > MAP_HEIGHT as f32 / 2.0 { dy -= MAP_HEIGHT as f32; }
            else if dy < -(MAP_HEIGHT as f32 / 2.0) { dy += MAP_HEIGHT as f32; }
            let dist = (dx * dx + dy * dy).sqrt().max(1.0);

            // Initial velocity: forward along player view direction with tiny random offset (±0.1 rad / ±5 deg)
            let angle_offset = (rand_c - 0.5) * 0.2; 
            let cos_a = angle_offset.cos();
            let sin_a = angle_offset.sin();
            let base_speed = dist / total_flight; // Cover the distance in target duration
            let init_vx = (pdx * cos_a - pdy * sin_a) * base_speed;
            let init_vy = (pdx * sin_a + pdy * cos_a) * base_speed;
            let init_vz = 3.5 + rand_b * 1.5; // Strong initial upward arc into the sky

            self.missiles.push(GuidedMissile {
                x: px,
                y: py,
                z: 0.4,
                vx: init_vx,
                vy: init_vy,
                vz: init_vz,
                target_id: self.citizens[*idx].id_num.clone(),
                target_x: *tx,
                target_y: *ty,
                flight_time: 0.0,
                steer_delay,
                total_flight,
            });
        }

        self.missile_used = true;
        play_sound("missile_launch");
    }

    /// Primary game state update loop
    pub fn update(&mut self, dt: f32) {
        // Update screen blood splatters
        self.screen_blood.retain_mut(|b| {
            b.lifetime -= dt;
            if b.lifetime <= 0.0 {
                return false;
            }
            // Move down (run down)
            b.y += b.vy * dt;
            // Slowly decelerate drip
            b.vy *= 0.96;
            true
        });

        if self.is_in_menu {
            if crate::game::is_game_started() {
                self.menu_timer += dt;
            }
            // Update menu shockwaves
            self.menu_shockwaves.retain_mut(|sw| {
                sw.lifetime += dt;
                sw.radius += sw.speed * dt;
                sw.lifetime < sw.max_lifetime
            });
        }

        // Focus scan window typing animation update
        let mut play_tick = false;
        if self.player.target_idx != self.focus_target_idx {
            self.focus_target_idx = self.player.target_idx;
            self.focus_text_timer = 0.0;
            if self.player.target_idx.is_some() {
                play_tick = true;
            }
        } else if self.player.target_idx.is_some() {
            let speed = 450.0;
            let prev_chars = (self.focus_text_timer * speed) as usize;
            self.focus_text_timer += dt;
            let cur_chars = (self.focus_text_timer * speed) as usize;

            // Total length of target info text is roughly 90-100 characters.
            // Play a tick every 4 characters during typing to create a high-speed chirping zipper effect.
            if cur_chars > prev_chars && cur_chars <= 100 {
                if cur_chars % 4 == 0 {
                    play_tick = true;
                }
            }
        } else {
            self.focus_text_timer += dt;
        }

        if play_tick {
            play_sound("scan_tick");
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
            txt.z += dt * 0.4; // drift upward vertically in 3D world space
            txt.duration > 0.0
        });

        // Spawn steam particles procedurally at sidewalk/road drains
        let p_tx = self.player.x.floor() as i32;
        let p_ty = self.player.y.floor() as i32;
        let scan_radius = 6;
        for dx in -scan_radius..=scan_radius {
            for dy in -scan_radius..=scan_radius {
                let gx = (p_tx + dx).rem_euclid(MAP_WIDTH as i32) as usize;
                let gy = (p_ty + dy).rem_euclid(MAP_HEIGHT as i32) as usize;
                
                // A tile is a drain if it's not a wall, and passes the coordinate hash check
                let is_wall = match self.map.grid[gx][gy] {
                    crate::map::TileType::Wall(_) => true,
                    _ => false,
                };

                // Check if any tile in a 1-tile radius is an AC wall (style 1)
                let mut near_ac_wall = false;
                if !is_wall {
                    for w_dx in -1..=1 {
                        for w_dy in -1..=1 {
                            let nx = (gx as i32 + w_dx).rem_euclid(MAP_WIDTH as i32) as usize;
                            let ny = (gy as i32 + w_dy).rem_euclid(MAP_HEIGHT as i32) as usize;
                            if let crate::map::TileType::Wall(1) = self.map.grid[nx][ny] {
                                near_ac_wall = true;
                                break;
                            }
                        }
                        if near_ac_wall {
                            break;
                        }
                    }
                }

                if !is_wall && !near_ac_wall && (gx * 23 + gy * 37) % 11 == 0 {
                    // Check if it's on the player's sidewalk
                    let player_tile = self.map.grid[self.player.tx][self.player.ty];
                    let is_player_sidewalk = match player_tile {
                        crate::map::TileType::SidewalkVert => gx == self.player.tx,
                        crate::map::TileType::SidewalkHoriz => gy == self.player.ty,
                        _ => false,
                    };

                    // Check if it's closely in front of the player (within 4.0 tiles distance)
                    let map_w = MAP_WIDTH as f32;
                    let map_h = MAP_HEIGHT as f32;
                    let mut rel_dx = (gx as f32 + 0.5) - self.player.x;
                    if rel_dx > map_w / 2.0 { rel_dx -= map_w; }
                    else if rel_dx < -map_w / 2.0 { rel_dx += map_w; }

                    let mut rel_dy = (gy as f32 + 0.5) - self.player.y;
                    if rel_dy > map_h / 2.0 { rel_dy -= map_h; }
                    else if rel_dy < -map_h / 2.0 { rel_dy += map_h; }

                    let dot = rel_dx * self.player.dir_x + rel_dy * self.player.dir_y;
                    let dist_sq = rel_dx * rel_dx + rel_dy * rel_dy;
                    let is_close_in_front = dot > 0.0 && dist_sq <= 16.0;

                    let mut spawn_chance = 25; // 2.5% base chance per frame
                    if is_player_sidewalk && is_close_in_front {
                        spawn_chance = 180; // Boosted to 18% chance per frame
                    }

                    if (next_rng(&mut self.rng_state) % 1000) < spawn_chance {
                        let offset_x = (rng_float(&mut self.rng_state) - 0.5) * 0.15;
                        let offset_y = (rng_float(&mut self.rng_state) - 0.5) * 0.15;
                        let sx = gx as f32 + 0.5 + offset_x;
                        let sy = gy as f32 + 0.5 + offset_y;
                        
                        let vx = (rng_float(&mut self.rng_state) - 0.5) * 0.06;
                        let vy = (rng_float(&mut self.rng_state) - 0.5) * 0.06;
                        let vz = 0.05 + rng_float(&mut self.rng_state) * 0.05;
                        
                        self.particles.push(Particle {
                            x: sx,
                            y: sy,
                            z: 0.0,
                            vx,
                            vy,
                            vz,
                            p_type: ParticleType::Steam,
                            bounces: 0,
                            lifetime: 1.5 + rng_float(&mut self.rng_state) * 1.0,
                            first_impact: false,
                            tex_idx: 0,
                            angle: 0.0,
                            spin_speed: 0.0,
                        });
                    }
                }
            }
        }

        // Update particles
        let gravity = 14.0; // Higher gravity for snappier/faster particle descent
        let map_w = MAP_WIDTH as f32;
        let map_h = MAP_HEIGHT as f32;
        
        let mut new_decals = Vec::new();

        self.particles.retain_mut(|p| {
            if p.tex_idx != 4 {
                p.lifetime -= dt;
                if p.lifetime <= 0.0 {
                    return false;
                }
            }

            // Immediately destroy particles behind the player to optimize performance
            let mut dx = p.x - self.player.x;
            if dx > map_w / 2.0 { dx -= map_w; }
            else if dx < -map_w / 2.0 { dx += map_w; }

            let mut dy = p.y - self.player.y;
            if dy > map_h / 2.0 { dy -= map_h; }
            else if dy < -map_h / 2.0 { dy += map_h; }

            let dot = dx * self.player.dir_x + dy * self.player.dir_y;
            if dot < 0.0 {
                return false;
            }

            if p.p_type == ParticleType::Smoke || p.p_type == ParticleType::Steam {
                p.x = (p.x + p.vx * dt).rem_euclid(map_w);
                p.y = (p.y + p.vy * dt).rem_euclid(map_h);
                p.z += p.vz * dt;
                if p.p_type == ParticleType::Steam {
                    p.vx *= 0.98;
                    p.vy *= 0.98;
                    p.vz = (p.vz + 0.15 * dt).min(0.25); // Float upward slowly
                } else {
                    p.vx *= 0.95;
                    p.vy *= 0.95;
                    p.vz *= 0.95;
                }
                return true;
            }

            p.vz -= gravity * dt;

            // Apply horizontal air resistance to blood/meat particles
            p.vx *= (1.0 - 5.0 * dt).max(0.0);
            p.vy *= (1.0 - 5.0 * dt).max(0.0);

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

            // Apply spin while flying/moving
            if p.tex_idx == 4 {
                p.angle += p.spin_speed * dt;
            }

            // Floor collision
            let coll_z = if p.tex_idx == 4 { 0.01953125 } else { 0.0 };
            if p.z <= coll_z {
                p.z = coll_z;
                
                // Spawn a blood decal on floor contact with reduced probability
                let is_sprinkle = p.p_type == ParticleType::BloodSprinkle;

                // Play meat landing impact sound less frequently:
                // Only on GoreDebris, falling fast (vz < -0.5), only on first impact, and with 33% chance.
                if p.p_type == ParticleType::GoreDebris && p.vz < -0.5 && p.first_impact && (next_rng(&mut self.rng_state) % 3 == 0) {
                    play_sound("impact");
                }
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

                    if p.tex_idx == 4 {
                        p.spin_speed = 0.0;
                        let norm_angle = p.angle.rem_euclid(2.0 * std::f32::consts::PI);
                        if norm_angle >= std::f32::consts::PI * 0.5 && norm_angle < std::f32::consts::PI * 1.5 {
                            p.angle = std::f32::consts::PI;
                        } else {
                            p.angle = 0.0;
                        }
                    }
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

        // Update decals lifetime & filter out distant decals so they are gone once scrolled out
        let px = self.player.x;
        let py = self.player.y;
        self.decals.retain_mut(|decal| {
            decal.lifetime -= dt;
            if decal.lifetime <= 0.0 {
                return false;
            }

            // Calculate distance to player (wrapping on torus)
            let mut dx = decal.x - px;
            if dx > MAP_WIDTH as f32 / 2.0 { dx -= map_w; }
            else if dx < -(MAP_WIDTH as f32 / 2.0) { dx += map_w; }
            let mut dy = decal.y - py;
            if dy > MAP_HEIGHT as f32 / 2.0 { dy -= map_h; }
            else if dy < -(MAP_HEIGHT as f32 / 2.0) { dy += map_h; }

            dx * dx + dy * dy < 15.0 * 15.0
        });

        // Update credits flash banner
        if let Some((_, _, ref mut duration)) = self.credits_flash {
            *duration -= dt;
            if *duration <= 0.0 {
                self.credits_flash = None;
            }
        }

        // Update guided missiles
        {
            let _px = self.player.x;
            let _py = self.player.y;
            let map_w = MAP_WIDTH as f32;
            let map_h = MAP_HEIGHT as f32;

            // Collect indices to remove and kill events
            let mut to_remove: Vec<usize> = Vec::new();
            let mut kills: Vec<(usize, f32, f32)> = Vec::new(); // (citizen_idx, x, y)

            for (mi, missile) in self.missiles.iter_mut().enumerate() {
                // Live target homing: update target coordinates in case the citizen moved
                if let Some(c) = self.citizens.iter().find(|c| c.id_num == missile.target_id) {
                    missile.target_x = c.x;
                    missile.target_y = c.y;
                }

                missile.flight_time += dt;

                // Phase 2: apply homing steering after steer_delay
                if missile.flight_time >= missile.steer_delay {
                    let mut tdx = missile.target_x - missile.x;
                    if tdx > map_w / 2.0 { tdx -= map_w; }
                    else if tdx < -(map_w / 2.0) { tdx += map_w; }
                    let mut tdy = missile.target_y - missile.y;
                    if tdy > map_h / 2.0 { tdy -= map_h; }
                    else if tdy < -(map_h / 2.0) { tdy += map_h; }

                    // Remaining time to target
                    let time_left = (missile.total_flight - missile.flight_time).max(0.01);

                    // Desired velocity to reach target in remaining time
                    let desired_vx = tdx / time_left;
                    let desired_vy = tdy / time_left;

                    // Descend to z=0.4 (head height) for terminal phase
                    let desired_vz = (0.4 - missile.z) / time_left;

                    // Exponential steering: blend toward desired velocity
                    let steer_rate = 9.0; // aggressiveness of course correction
                    missile.vx += (desired_vx - missile.vx) * steer_rate * dt;
                    missile.vy += (desired_vy - missile.vy) * steer_rate * dt;
                    missile.vz += (desired_vz - missile.vz) * steer_rate * dt;
                } else {
                    // Phase 1: just gravity drag on Z so it arcs naturally
                    missile.vz -= 1.5 * dt;
                }

                // Move missile
                missile.x = (missile.x + missile.vx * dt).rem_euclid(map_w);
                missile.y = (missile.y + missile.vy * dt).rem_euclid(map_h);
                missile.z += missile.vz * dt;
                if missile.z < 0.1 { missile.z = 0.1; } // don't go underground

                // Emit 3D smoke particle in the scene
                let rand_vx = (rng_float(&mut self.rng_state) - 0.5) * 0.4;
                let rand_vy = (rng_float(&mut self.rng_state) - 0.5) * 0.4;
                let rand_vz = (rng_float(&mut self.rng_state) - 0.2) * 0.4;
                self.particles.push(Particle {
                    x: missile.x,
                    y: missile.y,
                    z: missile.z,
                    vx: -missile.vx * 0.12 + rand_vx,
                    vy: -missile.vy * 0.12 + rand_vy,
                    vz: rand_vz,
                    p_type: ParticleType::Smoke,
                    bounces: 0,
                    lifetime: 0.8,
                    first_impact: false,
                    tex_idx: 0,
                    angle: 0.0,
                    spin_speed: 0.0,
                });

                // Check arrival (either time-based or close proximity to live target)
                let mut arrived = missile.flight_time >= missile.total_flight;
                if !arrived {
                    if let Some(c) = self.citizens.iter().find(|c| c.id_num == missile.target_id) {
                        let mut dx = c.x - missile.x;
                        if dx > map_w / 2.0 { dx -= map_w; }
                        else if dx < -(map_w / 2.0) { dx += map_w; }
                        let mut dy = c.y - missile.y;
                        if dy > map_h / 2.0 { dy -= map_h; }
                        else if dy < -(map_h / 2.0) { dy += map_h; }
                        
                        let dist_sq = dx * dx + dy * dy + (missile.z - 0.4) * (missile.z - 0.4);
                        if dist_sq < 0.25 { // Proximity threshold: 0.5 units in 3D space
                            arrived = true;
                        }
                    }
                }

                if arrived {
                    to_remove.push(mi);
                    // Schedule kill
                    if let Some(cidx) = self.citizens.iter().position(|c| c.id_num == missile.target_id) {
                        let c = &self.citizens[cidx];
                        if c.state == CitizenState::Walking && c.is_visually_leftsider() {
                            kills.push((cidx, c.x, c.y));
                        }
                    }
                }
            }

            // Remove finished missiles (in reverse order to preserve indices)
            to_remove.sort_unstable();
            to_remove.dedup();
            for mi in to_remove.iter().rev() {
                self.missiles.swap_remove(*mi);
            }

            // Apply kills
            for (cidx, kx, ky) in kills {
                if cidx >= self.citizens.len() { continue; }
                if self.citizens[cidx].state != CitizenState::Walking { continue; }
                if !self.citizens[cidx].is_visually_leftsider() { continue; }

                self.citizens[cidx].state = CitizenState::Exploding(0.0);
                self.citizens[cidx].shoot_cooldown = 0.0;

                play_sound("explosion");
                play_sound("cash_earn");

                let reward = 750;
                self.player.credits += reward;
                self.offenders_killed_rocket += 1;
                self.screen_shake = (self.screen_shake + 0.1).min(0.3);

                let mut dx = kx - self.player.x;
                if dx > MAP_WIDTH as f32 / 2.0 { dx -= MAP_WIDTH as f32; }
                else if dx < -(MAP_WIDTH as f32 / 2.0) { dx += MAP_WIDTH as f32; }

                let mut dy = ky - self.player.y;
                if dy > MAP_HEIGHT as f32 / 2.0 { dy -= MAP_HEIGHT as f32; }
                else if dy < -(MAP_HEIGHT as f32 / 2.0) { dy += MAP_HEIGHT as f32; }

                let dist = (dx * dx + dy * dy).sqrt();
                let is_hud = dist < 2.2;

                self.floating_texts.push(FloatingText {
                    text: format!("+{} CR", reward),
                    x: kx,
                    y: ky,
                    z: 0.55,
                    is_hud,
                    color: 0x39ff14ff,
                    duration: 1.2,
                });

                self.credits_flash = Some((
                    format!("CRIMINAL EDUCATED // +{} CR", reward),
                    0x39ff14ff,
                    1.2,
                ));

                self.spawn_blood_explosion(kx, ky);
            }

            // Drop missiles targeting despawned/dead citizens (safe borrow split)
            {
                let keep: Vec<bool> = self.missiles.iter().map(|m| {
                    if let Some(c) = self.citizens.iter().find(|c| c.id_num == m.target_id) {
                        c.state == CitizenState::Walking
                    } else {
                        false
                    }
                }).collect();
                let mut i = 0;
                self.missiles.retain(|_| { let k = keep[i]; i += 1; k });
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

        // Despawn vehicles that are too far away or behind the player, or moving perpendicular (sideways) to player direction
        self.vehicles.retain(|v| {
            let mut dx = v.x - px;
            if dx > MAP_WIDTH as f32 / 2.0 { dx -= MAP_WIDTH as f32; }
            else if dx < -(MAP_WIDTH as f32 / 2.0) { dx += MAP_WIDTH as f32; }

            let mut dy = v.y - py;
            if dy > MAP_HEIGHT as f32 / 2.0 { dy -= MAP_HEIGHT as f32; }
            else if dy < -(MAP_HEIGHT as f32 / 2.0) { dy += MAP_HEIGHT as f32; }

            let dist = (dx*dx + dy*dy).sqrt();
            if dist > 20.0 {
                return false;
            }

            let mut vdx = v.next_tx as f32 - v.tx as f32;
            if vdx > MAP_WIDTH as f32 / 2.0 { vdx -= MAP_WIDTH as f32; }
            else if vdx < -(MAP_WIDTH as f32 / 2.0) { vdx += MAP_WIDTH as f32; }

            let mut vdy = v.next_ty as f32 - v.ty as f32;
            if vdy > MAP_HEIGHT as f32 / 2.0 { vdy -= MAP_HEIGHT as f32; }
            else if vdy < -(MAP_HEIGHT as f32 / 2.0) { vdy += MAP_HEIGHT as f32; }

            let v_len = (vdx*vdx + vdy*vdy).sqrt();
            let is_moving_away = if v_len > 0.01 {
                let v_dir_x = vdx / v_len;
                let v_dir_y = vdy / v_len;
                v_dir_x * dx + v_dir_y * dy > 0.0
            } else {
                true
            };

            let dot = dx * pdx + dy * pdy;
            if dot < -4.0 && dist > 3.0 && is_moving_away {
                return false;
            }

            // Despawn vehicles moving perpendicular to player's view direction
            let p_len = (pdx*pdx + pdy*pdy).sqrt();
            if v_len > 0.01 && p_len > 0.01 {
                let v_dir_x = vdx / v_len;
                let v_dir_y = vdy / v_len;
                let p_dir_x = pdx / p_len;
                let p_dir_y = pdy / p_len;

                let dir_dot = v_dir_x * p_dir_x + v_dir_y * p_dir_y;
                if dir_dot.abs() < 0.7 {
                    return false; // Perpendicular! Despawn.
                }
            }

            true
        });

        // Count visible vehicles in front of the player
        let mut visible_v_count = 0;
        for v in &self.vehicles {
            let mut dx = v.x - px;
            if dx > MAP_WIDTH as f32 / 2.0 { dx -= MAP_WIDTH as f32; }
            else if dx < -(MAP_WIDTH as f32 / 2.0) { dx += MAP_WIDTH as f32; }

            let mut dy = v.y - py;
            if dy > MAP_HEIGHT as f32 / 2.0 { dy -= MAP_HEIGHT as f32; }
            else if dy < -(MAP_HEIGHT as f32 / 2.0) { dy += MAP_HEIGHT as f32; }

            let dist = (dx*dx + dy*dy).sqrt();
            if dist < 16.0 {
                let dot = dx * pdx + dy * pdy;
                if dot > 0.0 {
                    visible_v_count += 1;
                }
            }
        }

        // Spawn vehicles on roads in front of the player
        let target_visible_v = 10;
        let mut v_spawn_attempts = 0;
        while visible_v_count < target_visible_v && v_spawn_attempts < 15 {
            v_spawn_attempts += 1;
            let p_tile_x = self.player.tx as i32;
            let p_tile_y = self.player.ty as i32;
            let center_x = p_tile_x;
            let center_y = p_tile_y;

            let player_is_vert = pdy.abs() > pdx.abs();
            let mut candidates = Vec::new();
            for gx_raw in (center_x - 10)..=(center_x + 10) {
                for gy_raw in (center_y - 10)..=(center_y + 10) {
                    let gx = gx_raw.rem_euclid(MAP_WIDTH as i32) as usize;
                    let gy = gy_raw.rem_euclid(MAP_HEIGHT as i32) as usize;
                    
                    if self.map.grid[gx][gy] == TileType::Road {
                        // Only spawn on roads parallel to player's current movement axis
                        if player_is_vert && (gx % 7 != 4) {
                            continue;
                        }
                        if !player_is_vert && (gy % 7 != 4) {
                            continue;
                        }

                        let mut tdx = gx as f32 + 0.5 - px;
                        if tdx > MAP_WIDTH as f32 / 2.0 { tdx -= MAP_WIDTH as f32; }
                        else if tdx < -(MAP_WIDTH as f32 / 2.0) { tdx += MAP_WIDTH as f32; }

                        let mut tdy = gy as f32 + 0.5 - py;
                        if tdy > MAP_HEIGHT as f32 / 2.0 { tdy -= MAP_HEIGHT as f32; }
                        else if tdy < -(MAP_HEIGHT as f32 / 2.0) { tdy += MAP_HEIGHT as f32; }

                        let dist = (tdx*tdx + tdy*tdy).sqrt();
                        if dist >= 5.0 && dist <= 16.0 {
                            let dot = tdx * pdx + tdy * pdy;
                            if dot > 0.4 || dot < -0.4 {
                                let mut occupied = false;
                                for v in &self.vehicles {
                                    if (v.tx % MAP_WIDTH == gx && v.ty % MAP_HEIGHT == gy) ||
                                       (v.next_tx % MAP_WIDTH == gx && v.next_ty % MAP_HEIGHT == gy) {
                                        occupied = true;
                                        break;
                                    }
                                }
                                if !occupied {
                                    candidates.push((gx_raw, gy_raw));
                                }
                            }
                        }
                    }
                }
            }

            if !candidates.is_empty() {
                let idx = (next_rng(&mut self.rng_state) as usize) % candidates.len();
                let (sx, sy) = candidates[idx];
                self.spawn_vehicle_at(sx as usize, sy as usize);
                visible_v_count += 1;
            } else {
                break;
            }
        }

        // Update vehicles
        for i in 0..self.vehicles.len() {
            let (new_tx, new_ty, new_prev_tx, new_prev_ty, new_next_tx, new_next_ty, new_progress) = {
                let vehicle = &self.vehicles[i];
                
                // Determine if vehicle is moving in the same direction as the player
                let is_same_direction = {
                    let mut pdx = self.player.next_tx as f32 - self.player.tx as f32;
                    if pdx > MAP_WIDTH as f32 / 2.0 { pdx -= MAP_WIDTH as f32; }
                    else if pdx < -(MAP_WIDTH as f32 / 2.0) { pdx += MAP_WIDTH as f32; }

                    let mut pdy = self.player.next_ty as f32 - self.player.ty as f32;
                    if pdy > MAP_HEIGHT as f32 / 2.0 { pdy -= MAP_HEIGHT as f32; }
                    else if pdy < -(MAP_HEIGHT as f32 / 2.0) { pdy += MAP_HEIGHT as f32; }

                    let mut vdx = vehicle.next_tx as f32 - vehicle.tx as f32;
                    if vdx > MAP_WIDTH as f32 / 2.0 { vdx -= MAP_WIDTH as f32; }
                    else if vdx < -(MAP_WIDTH as f32 / 2.0) { vdx += MAP_WIDTH as f32; }

                    let mut vdy = vehicle.next_ty as f32 - vehicle.ty as f32;
                    if vdy > MAP_HEIGHT as f32 / 2.0 { vdy -= MAP_HEIGHT as f32; }
                    else if vdy < -(MAP_HEIGHT as f32 / 2.0) { vdy += MAP_HEIGHT as f32; }

                    let p_len = (pdx*pdx + pdy*pdy).sqrt();
                    let v_len = (vdx*vdx + vdy*vdy).sqrt();
                    if p_len > 0.01 && v_len > 0.01 {
                        let dot = (pdx / p_len) * (vdx / v_len) + (pdy / p_len) * (vdy / v_len);
                        dot > 0.9
                    } else {
                        false
                    }
                };

                let speed_mult = if is_same_direction { 2.5 } else { 1.0 };
                let mut current_speed = vehicle.speed * speed_mult;
                if is_same_direction && current_speed < self.player.speed * 2.0 {
                    current_speed = self.player.speed * 2.0;
                }
                let progress = vehicle.progress + current_speed * dt;
                
                if progress >= 1.0 {
                    let old_prev_x = vehicle.tx;
                    let old_prev_y = vehicle.ty;
                    let tx = vehicle.next_tx;
                    let ty = vehicle.next_ty;

                    // Continue straight in current direction
                    let mut dx = tx as i32 - old_prev_x as i32;
                    if dx > MAP_WIDTH as i32 / 2 { dx -= MAP_WIDTH as i32; }
                    else if dx < -(MAP_WIDTH as i32 / 2) { dx += MAP_WIDTH as i32; }

                    let mut dy = ty as i32 - old_prev_y as i32;
                    if dy > MAP_HEIGHT as i32 / 2 { dy -= MAP_HEIGHT as i32; }
                    else if dy < -(MAP_HEIGHT as i32 / 2) { dy += MAP_HEIGHT as i32; }

                    let next_tx = (tx as i32 + dx).rem_euclid(MAP_WIDTH as i32) as usize;
                    let next_ty = (ty as i32 + dy).rem_euclid(MAP_HEIGHT as i32) as usize;

                    (tx, ty, old_prev_x, old_prev_y, next_tx, next_ty, 0.0)
                } else {
                    (vehicle.tx, vehicle.ty, vehicle.prev_tx, vehicle.prev_ty, vehicle.next_tx, vehicle.next_ty, progress)
                }
            };

            let vehicle = &mut self.vehicles[i];
            vehicle.tx = new_tx;
            vehicle.ty = new_ty;
            vehicle.prev_tx = new_prev_tx;
            vehicle.prev_ty = new_prev_ty;
            vehicle.next_tx = new_next_tx;
            vehicle.next_ty = new_next_ty;
            vehicle.progress = new_progress;
            vehicle.align_position();
            vehicle.age += dt;

            // Drive-by sound trigger logic
            let mut dx = vehicle.x - self.player.x;
            if dx > MAP_WIDTH as f32 / 2.0 { dx -= MAP_WIDTH as f32; }
            else if dx < -(MAP_WIDTH as f32 / 2.0) { dx += MAP_WIDTH as f32; }

            let mut dy = vehicle.y - self.player.y;
            if dy > MAP_HEIGHT as f32 / 2.0 { dy -= MAP_HEIGHT as f32; }
            else if dy < -(MAP_HEIGHT as f32 / 2.0) { dy += MAP_HEIGHT as f32; }

            let dist = (dx * dx + dy * dy).sqrt();
            if dist < 2.0 {
                if !vehicle.has_played_driveby {
                    vehicle.has_played_driveby = true;
                    play_sound("driveby");
                }
            } else if dist > 4.5 {
                vehicle.has_played_driveby = false;
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
                        let switch = switch_roll < 10;
                        
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
                    
                    // Smoothly interpolate lane_offset towards target_offset
                    let target_offset = if citizen.is_leftsider { -0.22 } else { 0.22 };
                    citizen.lane_offset += (target_offset - citizen.lane_offset) * 8.0 * dt;

                    citizen.walk_frame = if (citizen.progress * 4.0) as i32 % 2 == 0 { 0 } else { 1 };
                    citizen.align_position();
                }
            }

            // Execute combat/timer behaviors
            let shoot_event: Option<(f32, f32)> = None;
            let mut explode_done = false;

            {
                let citizen = &mut self.citizens[i];

                match citizen.state {
                    CitizenState::Walking => { }
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

                // Deal damage to player (sensory shake/sound, no actual health/shield damage)

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

        // Update raindrops
        let width_f = WIDTH as f32;
        let height_f = HEIGHT as f32;
        for drop in &mut self.rain_drops {
            drop.y += drop.speed * dt;
            drop.x -= 20.0 * dt; // slight wind angle to the left
            if drop.y > height_f {
                drop.y = -drop.length; // start just above screen
                drop.x = rng_float(&mut self.rng_state) * width_f;
                drop.speed = 400.0 + rng_float(&mut self.rng_state) * 200.0;
                drop.length = 10.0 + rng_float(&mut self.rng_state) * 8.0;
            }
            if drop.x < 0.0 {
                drop.x += width_f;
            }
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
                        let is_lefty = target.is_visually_leftsider();

                        let mut dx = target.x - self.player.x;
                        if dx > MAP_WIDTH as f32 / 2.0 { dx -= MAP_WIDTH as f32; }
                        else if dx < -(MAP_WIDTH as f32 / 2.0) { dx += MAP_WIDTH as f32; }

                        let mut dy = target.y - self.player.y;
                        if dy > MAP_HEIGHT as f32 / 2.0 { dy -= MAP_HEIGHT as f32; }
                        else if dy < -(MAP_HEIGHT as f32 / 2.0) { dy += MAP_HEIGHT as f32; }

                        let dist = (dx * dx + dy * dy).sqrt();
                        let is_hud = dist < 2.2;

                        if is_lefty {
                            // Correct elimination of criminal
                            let reward = 1000;
                            self.player.credits += reward;
                            self.offenders_killed_laser += 1;
                            play_sound("explosion");
                            play_sound("cash_earn");
                            
                            self.floating_texts.push(FloatingText {
                                text: format!("+{} CR", reward),
                                x: target.x,
                                y: target.y,
                                z: 0.55,
                                is_hud,
                                color: 0x39ff14ff, // Neon Green
                                duration: 1.2,
                            });

                            self.credits_flash = Some((
                                format!("CRIMINAL EDUCATED // +{} CR", reward),
                                0x39ff14ff,
                                1.5
                            ));
                        } else {
                            // Collateral Damage! Shot a compliant Rightsider!
                            let penalty = 1250;
                            self.player.credits -= penalty;
                            self.collateral_damage_kills += 1;
                            self.player.damage_flash = 0.2; // Red screenshake glow
                            play_sound("collateral");

                            self.floating_texts.push(FloatingText {
                                text: format!("-{} CR COLLATERAL", penalty),
                                x: target.x,
                                y: target.y,
                                z: 0.55,
                                is_hud,
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
        if switch_left && !self.player.is_leftsider {
            self.player.is_leftsider = true;
            play_sound("lane_swoosh");
        }
        if switch_right && self.player.is_leftsider {
            self.player.is_leftsider = false;
            play_sound("lane_swoosh");
        }
    }

    pub fn save_highscore_rust(&self, name: &str, score: i32) {
        let mut scores = self.load_leaderboard_rust();
        scores.push((name.to_string(), score));
        scores.sort_by(|a, b| b.1.cmp(&a.1));
        scores.truncate(10);
        
        let mut serialized = String::new();
        for (i, (n, s)) in scores.iter().enumerate() {
            if i > 0 {
                serialized.push(',');
            }
            serialized.push_str(&format!("{}:{}", n, s));
        }
        
        #[cfg(target_arch = "wasm32")]
        unsafe {
            save_scores_js(serialized.as_ptr(), serialized.len());
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            println!("[save_highscore] {}", serialized);
        }
    }

    pub fn load_leaderboard_rust(&self) -> Vec<(String, i32)> {
        let mut buffer = vec![0u8; 1024];
        #[cfg(target_arch = "wasm32")]
        let len = unsafe {
            load_scores_js(buffer.as_mut_ptr(), buffer.len())
        };
        #[cfg(not(target_arch = "wasm32"))]
        let len = {
            let mock = "APX:2500,LAW:2000,KXX:1500,PRM:1000,SEC:800,COP:600,DED:500,BAD:400,OUT:300,AAA:100";
            buffer[..mock.len()].copy_from_slice(mock.as_bytes());
            mock.len()
        };
        
        let serialized = String::from_utf8_lossy(&buffer[..len]);
        let mut scores = Vec::new();
        if !serialized.is_empty() {
            for entry in serialized.split(',') {
                let parts: Vec<&str> = entry.split(':').collect();
                if parts.len() == 2 {
                    if let Ok(score) = parts[1].parse::<i32>() {
                        scores.push((parts[0].to_string(), score));
                    }
                }
            }
        }
        scores
    }
}

pub fn open_privacy_modal() {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        open_privacy_modal_js();
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        println!("[open_privacy_modal]");
    }
}

pub fn open_music_modal() {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        open_music_modal_js();
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        println!("[open_music_modal]");
    }
}

pub fn toggle_help() {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        toggle_help_js();
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        println!("[toggle_help]");
    }
}

pub fn hide_help() {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        hide_help_js();
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        println!("[hide_help]");
    }
}

pub fn toggle_fullscreen() {
    #[cfg(target_arch = "wasm32")]
    unsafe {
        toggle_fullscreen_js();
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        println!("[toggle_fullscreen]");
    }
}
