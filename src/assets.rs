// Procedural Asset Generator for Rightsiders FPS

pub const TEX_SIZE: usize = 64;

#[derive(Clone)]
pub struct SpriteTexture {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u32>, // RGBA format: 0xRRGGBBAA
}

impl SpriteTexture {
    pub fn new(width: usize, height: usize, default_color: u32) -> Self {
        Self {
            width,
            height,
            pixels: vec![default_color; width * height],
        }
    }

    pub fn set_pixel(&mut self, x: i32, y: i32, color: u32) {
        if x >= 0 && x < self.width as i32 && y >= 0 && y < self.height as i32 {
            self.pixels[(y as usize) * self.width + (x as usize)] = color;
        }
    }

    pub fn draw_rect(&mut self, rx: i32, ry: i32, rw: i32, rh: i32, color: u32) {
        for y in ry..(ry + rh) {
            for x in rx..(rx + rw) {
                self.set_pixel(x, y, color);
            }
        }
    }

    pub fn draw_line(&mut self, mut x1: i32, mut y1: i32, x2: i32, y2: i32, color: u32) {
        let dx = (x2 - x1).abs();
        let dy = (y2 - y1).abs();
        let sx = if x1 < x2 { 1 } else { -1 };
        let sy = if y1 < y2 { 1 } else { -1 };
        let mut err = dx - dy;

        loop {
            self.set_pixel(x1, y1, color);
            if x1 == x2 && y1 == y2 { break; }
            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x1 += sx;
            }
            if e2 < dx {
                err += dx;
                y1 += sy;
            }
        }
    }

    pub fn draw_circle(&mut self, cx: i32, cy: i32, radius: i32, color: u32) {
        let mut x = radius;
        let mut y = 0;
        let mut err = 0;

        while x >= y {
            self.draw_horizontal_line(cx - x, cx + x, cy + y, color);
            self.draw_horizontal_line(cx - x, cx + x, cy - y, color);
            self.draw_horizontal_line(cx - y, cx + y, cy + x, color);
            self.draw_horizontal_line(cx - y, cx + y, cy - x, color);

            y += 1;
            if err <= 0 {
                err += 2 * y + 1;
            } else {
                x -= 1;
                err += 2 * (y - x) + 1;
            }
        }
    }

    fn draw_horizontal_line(&mut self, x1: i32, x2: i32, y: i32, color: u32) {
        for x in x1..=x2 {
            self.set_pixel(x, y, color);
        }
    }
}

pub struct GameAssets {
    // Wall textures (0: Neon Grid, 1: Tech Panel, 2: Advertisement, 3: Police HQ)
    pub walls: Vec<SpriteTexture>,
    // Sprites (0: Citizen Walk A, 1: Citizen Walk B, 2: Rebel Walk A, 3: Rebel Walk B, 4: Explode A, 5: Explode B, 6: Dead)
    pub sprites: Vec<SpriteTexture>,
    // Blaster Weapon (0: Idle, 1: Firing)
    #[allow(dead_code)]
    pub weapon: Vec<SpriteTexture>,
}

pub fn generate_assets() -> GameAssets {
    let mut walls = Vec::new();
    let mut sprites = Vec::new();
    let mut weapon = Vec::new();

    // COLOR PALETTE (RGBA 0xRRGGBBAA)
    let c_black = 0x00000000; // Transparent for sprites, black for walls
    let c_dark_blue = 0x090b15ff;
    let c_neon_cyan = 0x00f0ffff;
    let c_neon_pink = 0xff007fff;
    let c_neon_green = 0x39ff14ff;
    let c_neon_yellow = 0xffd700ff;
    let c_gray = 0x555555ff;
    let c_dark_gray = 0x222222ff;
    let c_light_gray = 0xaaaaaaff;
    let c_white = 0xffffffff;
    let c_red = 0xff0000ff;

    // ==========================================
    // WALL 0: Neon Grid Wall (Building)
    // ==========================================
    let mut w0 = SpriteTexture::new(TEX_SIZE, TEX_SIZE, c_dark_blue);
    // Draw neon cyan grid border
    w0.draw_rect(0, 0, TEX_SIZE as i32, 2, c_neon_cyan);
    w0.draw_rect(0, (TEX_SIZE - 2) as i32, TEX_SIZE as i32, 2, c_neon_cyan);
    w0.draw_rect(0, 0, 2, TEX_SIZE as i32, c_neon_cyan);
    w0.draw_rect((TEX_SIZE - 2) as i32, 0, 2, TEX_SIZE as i32, c_neon_cyan);
    // Draw windows
    for row in 0..3 {
        for col in 0..3 {
            let wx = 8 + col * 18;
            let wy = 8 + row * 18;
            w0.draw_rect(wx, wy, 10, 10, 0x112244ff);
            w0.draw_rect(wx + 2, wy + 2, 6, 6, c_neon_pink);
        }
    }
    walls.push(w0);

    // ==========================================
    // WALL 1: Tech Panel Wall
    // ==========================================
    let mut w1 = SpriteTexture::new(TEX_SIZE, TEX_SIZE, c_dark_gray);
    // Draw panel rivets and borders
    w1.draw_rect(0, 0, TEX_SIZE as i32, 1, c_gray);
    w1.draw_rect(0, 0, 1, TEX_SIZE as i32, c_gray);
    w1.draw_rect(0, (TEX_SIZE - 1) as i32, TEX_SIZE as i32, 1, c_black);
    w1.draw_rect((TEX_SIZE - 1) as i32, 0, 1, TEX_SIZE as i32, c_black);
    // Draw circuit-like lines
    w1.draw_line(10, 10, 10, 54, c_neon_green);
    w1.draw_line(10, 32, 54, 32, c_neon_green);
    w1.draw_line(54, 10, 54, 54, c_neon_green);
    w1.draw_circle(32, 32, 4, c_neon_green);
    w1.draw_circle(32, 32, 2, c_white);
    walls.push(w1);

    // ==========================================
    // WALL 2: Cyber-Advertisement Wall ("KEEP RIGHT")
    // ==========================================
    let mut w2 = SpriteTexture::new(TEX_SIZE, TEX_SIZE, 0x110515ff);
    w2.draw_rect(0, 0, TEX_SIZE as i32, TEX_SIZE as i32, 0x220a30ff);
    // Neon pink warning sign
    w2.draw_rect(10, 10, 44, 44, c_black);
    // Border glow
    w2.draw_rect(10, 10, 44, 2, c_neon_pink);
    w2.draw_rect(10, 52, 44, 2, c_neon_pink);
    w2.draw_rect(10, 10, 2, 44, c_neon_pink);
    w2.draw_rect(52, 10, 2, 44, c_neon_pink);
    // Draw an arrow pointing RIGHT (symbolic of keeping right)
    // Shaft
    w2.draw_rect(18, 29, 20, 6, c_neon_cyan);
    // Tip
    w2.draw_line(38, 22, 48, 32, c_neon_cyan);
    w2.draw_line(38, 42, 48, 32, c_neon_cyan);
    w2.draw_line(38, 23, 48, 32, c_neon_cyan);
    w2.draw_line(38, 41, 48, 32, c_neon_cyan);
    // "RIGHT" neon sign
    // Let's write simple pixel words: "GO" and "->"
    // Letter R
    w2.draw_rect(18, 14, 2, 5, c_neon_green);
    w2.draw_rect(18, 14, 4, 1, c_neon_green);
    w2.draw_rect(21, 14, 1, 3, c_neon_green);
    w2.draw_rect(18, 16, 4, 1, c_neon_green);
    w2.draw_line(20, 17, 22, 19, c_neon_green);
    // Letter ->
    w2.draw_rect(28, 16, 8, 2, c_neon_green);
    w2.draw_line(33, 13, 36, 17, c_neon_green);
    w2.draw_line(33, 21, 36, 17, c_neon_green);
    walls.push(w2);

    // ==========================================
    // WALL 3: Police HQ Panel
    // ==========================================
    let mut w3 = SpriteTexture::new(TEX_SIZE, TEX_SIZE, 0x050c18ff);
    w3.draw_rect(0, 0, TEX_SIZE as i32, TEX_SIZE as i32, 0x08152cff);
    // Double grid lines
    w3.draw_rect(0, 0, TEX_SIZE as i32, 2, c_neon_cyan);
    w3.draw_rect(0, (TEX_SIZE - 2) as i32, TEX_SIZE as i32, 2, c_neon_cyan);
    // Police Shield shape in the center
    // Top bar
    w3.draw_rect(20, 16, 24, 4, c_neon_cyan);
    // Shield sides curving down
    w3.draw_line(20, 18, 20, 36, c_neon_cyan);
    w3.draw_line(43, 18, 43, 36, c_neon_cyan);
    w3.draw_line(20, 36, 32, 48, c_neon_cyan);
    w3.draw_line(43, 36, 32, 48, c_neon_cyan);
    // Police star inside
    w3.draw_circle(32, 30, 4, c_neon_pink);
    w3.draw_circle(32, 30, 1, c_white);
    walls.push(w3);

    // ==========================================
    // SPRITE 0: Compliant Citizen Walk A
    // ==========================================
    let mut s0 = SpriteTexture::new(TEX_SIZE, TEX_SIZE, c_black);
    // Robot head
    s0.draw_circle(32, 16, 6, c_gray);
    s0.draw_circle(32, 16, 5, c_light_gray);
    // Visor (Green for compliant)
    s0.draw_rect(28, 14, 8, 2, c_neon_green);
    // Neck
    s0.draw_rect(30, 22, 4, 3, c_gray);
    // Torso (futuristic panel jacket)
    s0.draw_rect(22, 25, 20, 18, c_dark_blue);
    s0.draw_rect(24, 27, 16, 14, 0x152545ff);
    // Neon compliant chest strips
    s0.draw_rect(25, 29, 4, 10, c_neon_green);
    s0.draw_rect(35, 29, 4, 10, c_neon_green);
    // Left arm (idle)
    s0.draw_rect(18, 26, 4, 12, c_gray);
    // Right arm (walking forward/swung)
    s0.draw_rect(42, 26, 4, 10, c_gray);
    s0.draw_rect(42, 36, 4, 4, c_light_gray);
    // Left leg (stepping forward)
    s0.draw_rect(24, 43, 5, 12, c_gray);
    s0.draw_rect(22, 55, 7, 4, c_light_gray);
    // Right leg (trailing)
    s0.draw_rect(33, 43, 5, 8, c_gray);
    s0.draw_rect(33, 51, 5, 6, c_dark_gray);
    sprites.push(s0);

    // ==========================================
    // SPRITE 1: Compliant Citizen Walk B
    // ==========================================
    let mut s1 = SpriteTexture::new(TEX_SIZE, TEX_SIZE, c_black);
    // Same citizen, inverted leg positions
    s1.draw_circle(32, 16, 6, c_gray);
    s1.draw_circle(32, 16, 5, c_light_gray);
    s1.draw_rect(28, 14, 8, 2, c_neon_green);
    s1.draw_rect(30, 22, 4, 3, c_gray);
    s1.draw_rect(22, 25, 20, 18, c_dark_blue);
    s1.draw_rect(24, 27, 16, 14, 0x152545ff);
    s1.draw_rect(25, 29, 4, 10, c_neon_green);
    s1.draw_rect(35, 29, 4, 10, c_neon_green);
    // Arms swapped
    s1.draw_rect(18, 26, 4, 10, c_gray);
    s1.draw_rect(18, 36, 4, 4, c_light_gray);
    s1.draw_rect(42, 26, 4, 12, c_gray);
    // Legs swapped
    s1.draw_rect(24, 43, 5, 8, c_gray);
    s1.draw_rect(24, 51, 5, 6, c_dark_gray);
    s1.draw_rect(33, 43, 5, 12, c_gray);
    s1.draw_rect(33, 55, 7, 4, c_light_gray);
    sprites.push(s1);

    // ==========================================
    // SPRITE 2: Violator Walk A (Red details)
    // ==========================================
    let mut s2 = SpriteTexture::new(TEX_SIZE, TEX_SIZE, c_black);
    // Head
    s2.draw_circle(32, 16, 6, c_gray);
    s2.draw_circle(32, 16, 5, c_light_gray);
    // Red Visor of violator
    s2.draw_rect(28, 14, 8, 2, c_red);
    // Neck
    s2.draw_rect(30, 22, 4, 3, c_gray);
    // Torso (red jacket)
    s2.draw_rect(22, 25, 20, 18, 0x330808ff);
    s2.draw_rect(24, 27, 16, 14, 0x881515ff);
    // Neon Red warning strips
    s2.draw_rect(25, 29, 4, 10, c_red);
    s2.draw_rect(35, 29, 4, 10, c_red);
    // Left arm (idle)
    s2.draw_rect(18, 26, 4, 12, c_gray);
    // Right arm (walking forward/swung)
    s2.draw_rect(42, 26, 4, 10, c_gray);
    s2.draw_rect(42, 36, 4, 4, c_light_gray);
    // Legs (Step A)
    s2.draw_rect(24, 43, 5, 12, c_gray);
    s2.draw_rect(22, 55, 7, 4, c_light_gray);
    s2.draw_rect(33, 43, 5, 8, c_gray);
    s2.draw_rect(33, 51, 5, 6, c_dark_gray);
    sprites.push(s2);

    // ==========================================
    // SPRITE 3: Violator Walk B
    // ==========================================
    let mut s3 = SpriteTexture::new(TEX_SIZE, TEX_SIZE, c_black);
    // Violator Walk Frame B
    s3.draw_circle(32, 16, 6, c_gray);
    s3.draw_circle(32, 16, 5, c_light_gray);
    s3.draw_rect(28, 14, 8, 2, c_red);
    s3.draw_rect(30, 22, 4, 3, c_gray);
    s3.draw_rect(22, 25, 20, 18, 0x330808ff);
    s3.draw_rect(24, 27, 16, 14, 0x881515ff);
    s3.draw_rect(25, 29, 4, 10, c_red);
    s3.draw_rect(35, 29, 4, 10, c_red);
    // Arms swapped
    s3.draw_rect(18, 26, 4, 10, c_gray);
    s3.draw_rect(18, 36, 4, 4, c_light_gray);
    s3.draw_rect(42, 26, 4, 12, c_gray);
    // Legs swapped
    s3.draw_rect(24, 43, 5, 8, c_gray);
    s3.draw_rect(24, 51, 5, 6, c_dark_gray);
    s3.draw_rect(33, 43, 5, 12, c_gray);
    s3.draw_rect(33, 55, 7, 4, c_light_gray);
    sprites.push(s3);

    // ==========================================
    // SPRITE 4: Explode A (Robot Sparks)
    // ==========================================
    let mut s4 = SpriteTexture::new(TEX_SIZE, TEX_SIZE, c_black);
    // Expanding blue and pink electric sparks
    s4.draw_circle(32, 32, 8, c_neon_cyan);
    s4.draw_circle(32, 32, 4, c_white);
    s4.draw_circle(24, 20, 2, c_neon_pink);
    s4.draw_circle(40, 24, 2, c_neon_pink);
    s4.draw_circle(20, 40, 2, c_neon_pink);
    s4.draw_circle(44, 44, 2, c_neon_pink);
    // Floating random pixels
    s4.set_pixel(15, 15, c_neon_cyan);
    s4.set_pixel(50, 15, c_neon_pink);
    s4.set_pixel(15, 50, c_neon_pink);
    s4.set_pixel(50, 50, c_neon_cyan);
    sprites.push(s4);

    // ==========================================
    // SPRITE 5: Explode B
    // ==========================================
    let mut s5 = SpriteTexture::new(TEX_SIZE, TEX_SIZE, c_black);
    // Larger, fading ring of sparks
    s5.draw_circle(32, 32, 16, c_neon_pink);
    s5.draw_circle(32, 32, 12, c_black);
    s5.draw_circle(32, 32, 6, c_neon_cyan);
    s5.draw_circle(32, 32, 2, c_white);
    s5.draw_circle(14, 14, 3, c_neon_cyan);
    s5.draw_circle(50, 14, 3, c_neon_cyan);
    s5.draw_circle(14, 50, 3, c_neon_cyan);
    s5.draw_circle(50, 50, 3, c_neon_cyan);
    sprites.push(s5);

    // ==========================================
    // SPRITE 6: Dead (Robot Scrap Pile)
    // ==========================================
    let mut s6 = SpriteTexture::new(TEX_SIZE, TEX_SIZE, c_black);
    // Scrap metal heap on floor
    s6.draw_circle(32, 54, 8, c_dark_gray);
    s6.draw_circle(26, 56, 5, c_gray);
    s6.draw_circle(38, 56, 5, c_gray);
    s6.draw_rect(24, 52, 16, 6, c_gray);
    // Broken head lying separate
    s6.draw_circle(18, 56, 4, c_light_gray);
    s6.set_pixel(17, 55, c_black); // Dead black visor pixel
    // Glowing neon red coolant puddle leaking out
    s6.draw_horizontal_line(15, 48, 60, c_red);
    s6.draw_horizontal_line(20, 42, 61, c_red);
    sprites.push(s6);

    // ==========================================
    // SPRITE 7: Compliant Citizen Walk A (Back View)
    // ==========================================
    let mut s7 = SpriteTexture::new(TEX_SIZE, TEX_SIZE, c_black);
    s7.draw_circle(32, 16, 6, c_gray);
    s7.draw_circle(32, 16, 5, c_light_gray);
    // (No visor)
    s7.draw_rect(30, 22, 4, 3, c_gray);
    s7.draw_rect(22, 25, 20, 18, c_dark_blue);
    s7.draw_rect(24, 27, 16, 14, 0x152545ff);
    // (No chest strips)
    s7.draw_rect(18, 26, 4, 12, c_gray);
    s7.draw_rect(42, 26, 4, 10, c_gray);
    s7.draw_rect(42, 36, 4, 4, c_light_gray);
    s7.draw_rect(24, 43, 5, 12, c_gray);
    s7.draw_rect(22, 55, 7, 4, c_light_gray);
    s7.draw_rect(33, 43, 5, 8, c_gray);
    s7.draw_rect(33, 51, 5, 6, c_dark_gray);
    sprites.push(s7);

    // ==========================================
    // SPRITE 8: Compliant Citizen Walk B (Back View)
    // ==========================================
    let mut s8 = SpriteTexture::new(TEX_SIZE, TEX_SIZE, c_black);
    s8.draw_circle(32, 16, 6, c_gray);
    s8.draw_circle(32, 16, 5, c_light_gray);
    s8.draw_rect(30, 22, 4, 3, c_gray);
    s8.draw_rect(22, 25, 20, 18, c_dark_blue);
    s8.draw_rect(24, 27, 16, 14, 0x152545ff);
    s8.draw_rect(18, 26, 4, 10, c_gray);
    s8.draw_rect(18, 36, 4, 4, c_light_gray);
    s8.draw_rect(42, 26, 4, 12, c_gray);
    s8.draw_rect(24, 43, 5, 8, c_gray);
    s8.draw_rect(24, 51, 5, 6, c_dark_gray);
    s8.draw_rect(33, 43, 5, 12, c_gray);
    s8.draw_rect(33, 55, 7, 4, c_light_gray);
    sprites.push(s8);

    // ==========================================
    // SPRITE 9: Violator Walk A (Back View)
    // ==========================================
    let mut s9 = SpriteTexture::new(TEX_SIZE, TEX_SIZE, c_black);
    s9.draw_circle(32, 16, 6, c_gray);
    s9.draw_circle(32, 16, 5, c_light_gray);
    s9.draw_rect(30, 22, 4, 3, c_gray);
    s9.draw_rect(22, 25, 20, 18, 0x330808ff);
    s9.draw_rect(24, 27, 16, 14, 0x881515ff);
    s9.draw_rect(18, 26, 4, 12, c_gray);
    s9.draw_rect(42, 26, 4, 10, c_gray);
    s9.draw_rect(42, 36, 4, 4, c_light_gray);
    s9.draw_rect(24, 43, 5, 12, c_gray);
    s9.draw_rect(22, 55, 7, 4, c_light_gray);
    s9.draw_rect(33, 43, 5, 8, c_gray);
    s9.draw_rect(33, 51, 5, 6, c_dark_gray);
    sprites.push(s9);

    // ==========================================
    // SPRITE 10: Violator Walk B (Back View)
    // ==========================================
    let mut s10 = SpriteTexture::new(TEX_SIZE, TEX_SIZE, c_black);
    s10.draw_circle(32, 16, 6, c_gray);
    s10.draw_circle(32, 16, 5, c_light_gray);
    s10.draw_rect(30, 22, 4, 3, c_gray);
    s10.draw_rect(22, 25, 20, 18, 0x330808ff);
    s10.draw_rect(24, 27, 16, 14, 0x881515ff);
    s10.draw_rect(18, 26, 4, 10, c_gray);
    s10.draw_rect(18, 36, 4, 4, c_light_gray);
    s10.draw_rect(42, 26, 4, 12, c_gray);
    s10.draw_rect(24, 43, 5, 8, c_gray);
    s10.draw_rect(24, 51, 5, 6, c_dark_gray);
    s10.draw_rect(33, 43, 5, 12, c_gray);
    s10.draw_rect(33, 55, 7, 4, c_light_gray);
    sprites.push(s10);

    // ==========================================
    // WEAPON 0: Robo-Blaster Idle
    // ==========================================
    let mut w_idle = SpriteTexture::new(TEX_SIZE * 2, TEX_SIZE * 2, c_black);
    let wx_offset = TEX_SIZE as i32; // Centering helper
    let wy_offset = TEX_SIZE as i32;
    // Draw player arms/glove (metallic navy-blue and black)
    w_idle.draw_rect(wx_offset - 20, wy_offset + 30, 24, 40, c_dark_gray);
    w_idle.draw_rect(wx_offset - 16, wy_offset + 34, 18, 36, c_gray);
    // Gun body (sleek chrome/black)
    w_idle.draw_rect(wx_offset - 10, wy_offset - 10, 20, 45, 0x151b26ff);
    w_idle.draw_rect(wx_offset - 8, wy_offset - 5, 16, 40, c_gray);
    // Cyber scope with a blue light
    w_idle.draw_rect(wx_offset - 12, wy_offset + 10, 4, 15, c_neon_cyan);
    w_idle.draw_circle(wx_offset - 10, wy_offset + 17, 2, c_white);
    // Cyber glowing vents (neon cyan)
    w_idle.draw_rect(wx_offset - 4, wy_offset + 5, 8, 3, c_neon_cyan);
    w_idle.draw_rect(wx_offset - 4, wy_offset + 15, 8, 3, c_neon_cyan);
    w_idle.draw_rect(wx_offset - 4, wy_offset + 25, 8, 3, c_neon_cyan);
    // Laser Barrel pointing up-front
    w_idle.draw_rect(wx_offset - 6, wy_offset - 25, 12, 15, c_gray);
    w_idle.draw_rect(wx_offset - 3, wy_offset - 35, 6, 12, c_light_gray);
    w_idle.draw_rect(wx_offset - 2, wy_offset - 36, 4, 2, c_neon_cyan); // Glow tip
    weapon.push(w_idle);

    // ==========================================
    // WEAPON 1: Robo-Blaster Firing
    // ==========================================
    let mut w_fire = SpriteTexture::new(TEX_SIZE * 2, TEX_SIZE * 2, c_black);
    // Gun body shifted slightly down (recoil) and muzzle flash drawn
    let recoil_y = 6;
    w_fire.draw_rect(wx_offset - 20, wy_offset + 30 + recoil_y, 24, 40, c_dark_gray);
    w_fire.draw_rect(wx_offset - 16, wy_offset + 34 + recoil_y, 18, 36, c_gray);
    w_fire.draw_rect(wx_offset - 10, wy_offset - 10 + recoil_y, 20, 45, 0x151b26ff);
    w_fire.draw_rect(wx_offset - 8, wy_offset - 5 + recoil_y, 16, 40, c_gray);
    w_fire.draw_rect(wx_offset - 12, wy_offset + 10 + recoil_y, 4, 15, c_neon_cyan);
    w_fire.draw_circle(wx_offset - 10, wy_offset + 17 + recoil_y, 2, c_white);
    // Cyan vents glow brighter (white center)
    w_fire.draw_rect(wx_offset - 4, wy_offset + 5 + recoil_y, 8, 3, c_white);
    w_fire.draw_rect(wx_offset - 4, wy_offset + 15 + recoil_y, 8, 3, c_white);
    w_fire.draw_rect(wx_offset - 4, wy_offset + 25 + recoil_y, 8, 3, c_white);
    w_fire.draw_rect(wx_offset - 6, wy_offset - 25 + recoil_y, 12, 15, c_gray);
    w_fire.draw_rect(wx_offset - 3, wy_offset - 35 + recoil_y, 6, 12, c_light_gray);
    // MUZZLE FLASH! Large yellow/cyan starburst
    let flash_cy = wy_offset - 35 + recoil_y;
    w_fire.draw_circle(wx_offset, flash_cy, 18, c_neon_yellow);
    w_fire.draw_circle(wx_offset, flash_cy, 10, c_white);
    w_fire.draw_circle(wx_offset, flash_cy, 4, c_neon_cyan);
    // Sparks shooting out
    w_fire.draw_line(wx_offset, flash_cy, wx_offset - 25, flash_cy - 20, c_neon_cyan);
    w_fire.draw_line(wx_offset, flash_cy, wx_offset + 25, flash_cy - 20, c_neon_cyan);
    w_fire.draw_line(wx_offset, flash_cy, wx_offset, flash_cy - 30, c_white);
    weapon.push(w_fire);

    GameAssets {
        walls,
        sprites,
        weapon,
    }
}
