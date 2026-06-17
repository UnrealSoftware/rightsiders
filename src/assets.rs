// Procedural Asset Generator for Rightsiders FPS

pub const TEX_SIZE: usize = 64;

#[derive(Clone)]
pub struct SpriteTexture {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u32>, // RGBA format: 0xRRGGBBAA
}

#[allow(dead_code)]
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
}


#[allow(dead_code)]
pub fn generate_assets() -> GameAssets {
    let mut walls = Vec::new();
    let mut sprites = Vec::new();

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
    // Draw windows (some dark, some bright glowing cyan/yellow glass)
    for row in 0..3 {
        for col in 0..3 {
            let wx = 8 + col * 18;
            let wy = 8 + row * 18;
            w0.draw_rect(wx, wy, 10, 10, 0x050c18ff); // Dark window frame
            
            // Stagger window illumination/types
            let win_type = (row * 3 + col) % 4;
            match win_type {
                0 => {
                    // Bright glowing yellow cyber glass window
                    w0.draw_rect(wx + 1, wy + 1, 8, 8, 0xffd700ff);
                    w0.draw_rect(wx + 3, wy + 3, 4, 4, c_white); // reflective glare
                }
                1 => {
                    // Bright glowing cyan cyber glass window
                    w0.draw_rect(wx + 1, wy + 1, 8, 8, 0x00f0ffff);
                    w0.draw_rect(wx + 3, wy + 3, 4, 4, c_white); // reflective glare
                }
                2 => {
                    // Neon pink panel
                    w0.draw_rect(wx + 2, wy + 2, 6, 6, c_neon_pink);
                }
                _ => {
                    // Unlit dark blue window
                    w0.draw_rect(wx + 2, wy + 2, 6, 6, 0x141b2bff);
                }
            }
        }
    }
    walls.push(w0);

    // ==========================================
    // WALL 1: Tech Panel Wall with Air Conditioning module
    // ==========================================
    let mut w1 = SpriteTexture::new(TEX_SIZE, TEX_SIZE, c_dark_gray);
    // Draw panel rivets and borders
    w1.draw_rect(0, 0, TEX_SIZE as i32, 1, c_gray);
    w1.draw_rect(0, 0, 1, TEX_SIZE as i32, c_gray);
    w1.draw_rect(0, (TEX_SIZE - 1) as i32, TEX_SIZE as i32, 1, c_black);
    w1.draw_rect((TEX_SIZE - 1) as i32, 0, 1, TEX_SIZE as i32, c_black);
    // Draw circuit-like lines
    w1.draw_line(10, 10, 10, 24, c_neon_green);
    w1.draw_line(10, 20, 54, 20, c_neon_green);
    w1.draw_line(54, 10, 54, 24, c_neon_green);
    w1.draw_circle(32, 20, 3, c_neon_green);
    w1.draw_circle(32, 20, 1, c_white);
    
    // Draw Air Conditioning (AC) module in the bottom half
    let ac_x = 12;
    let ac_y = 32;
    let ac_w = 40;
    let ac_h = 24;
    // Box background (metallic)
    w1.draw_rect(ac_x, ac_y, ac_w, ac_h, 0x3a3d45ff);
    w1.draw_rect(ac_x + 1, ac_y + 1, ac_w - 2, ac_h - 2, 0x6e7380ff);
    // Box shadow/borders
    w1.draw_rect(ac_x, ac_y + ac_h - 1, ac_w, 1, 0x1a1c20ff);
    w1.draw_rect(ac_x + ac_w - 1, ac_y, 1, ac_h, 0x1a1c20ff);
    // Circular fan grill on the left (x: 16..32, y: 36..52)
    w1.draw_circle(ac_x + 10, ac_y + 12, 8, 0x1a1c20ff);
    // Fan center
    w1.draw_circle(ac_x + 10, ac_y + 12, 2, 0x3a3d45ff);
    // Fan blades (lines)
    w1.draw_line(ac_x + 10, ac_y + 4, ac_x + 10, ac_y + 20, 0x4a4d55ff);
    w1.draw_line(ac_x + 4, ac_y + 12, ac_x + 16, ac_y + 12, 0x4a4d55ff);
    // Vents on the right (horizontal lines)
    w1.draw_rect(ac_x + 22, ac_y + 5, 12, 2, 0x1a1c20ff);
    w1.draw_rect(ac_x + 22, ac_y + 9, 12, 2, 0x1a1c20ff);
    w1.draw_rect(ac_x + 22, ac_y + 13, 12, 2, 0x1a1c20ff);
    w1.draw_rect(ac_x + 22, ac_y + 17, 12, 2, 0x1a1c20ff);
    // Condenser piping (copper pipes at top/right)
    w1.draw_line(ac_x + 36, ac_y + 6, ac_x + 36, ac_y + 18, 0xb87333ff); // Copper color
    w1.draw_line(ac_x + 36, ac_y + 18, ac_x + 39, ac_y + 18, 0xb87333ff);
    
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

    // ==========================================
    // SPRITE 4: Explode A (Blood Burst Start)
    // ==========================================
    let mut s4 = SpriteTexture::new(TEX_SIZE, TEX_SIZE, c_black);
    // Dark red base with crimson core and hot pink highlights
    let c_dark_red = 0x880000ff;
    s4.draw_circle(32, 32, 10, c_dark_red);
    s4.draw_circle(32, 32, 6, c_red);
    s4.draw_circle(32, 32, 2, c_neon_pink);
    // Blood splatters shooting out
    s4.draw_line(32, 32, 20, 20, c_red);
    s4.draw_line(32, 32, 44, 20, c_red);
    s4.draw_line(32, 32, 20, 44, c_red);
    s4.draw_line(32, 32, 44, 44, c_red);
    s4.draw_line(32, 32, 32, 12, c_dark_red);
    s4.draw_line(32, 32, 32, 52, c_dark_red);
    s4.draw_line(32, 32, 12, 32, c_dark_red);
    s4.draw_line(32, 32, 52, 32, c_dark_red);
    sprites.push(s4);

    // ==========================================
    // SPRITE 5: Explode B (Blood Dispersion)
    // ==========================================
    let mut s5 = SpriteTexture::new(TEX_SIZE, TEX_SIZE, c_black);
    // Expanding, dissipating cloud of blood mist and droplets
    let c_dark_red = 0x880000ff;
    s5.draw_circle(32, 32, 16, c_dark_red);
    s5.draw_circle(32, 32, 12, c_black); // hollow center for expansion
    s5.draw_circle(32, 32, 8, c_red);
    s5.draw_circle(32, 32, 6, c_black); // hollow center
    s5.draw_circle(32, 32, 3, c_neon_pink);
    // Droplets flying further out
    s5.draw_circle(14, 14, 2, c_red);
    s5.draw_circle(50, 14, 2, c_red);
    s5.draw_circle(14, 50, 2, c_red);
    s5.draw_circle(50, 50, 2, c_red);
    s5.draw_circle(32, 8, 2, c_dark_red);
    s5.draw_circle(32, 56, 2, c_dark_red);
    s5.draw_circle(8, 32, 2, c_dark_red);
    s5.draw_circle(56, 32, 2, c_dark_red);
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

    // ==========================================
    // SPRITE 11: Blood Sprinkle (Pixelated droplet)
    // ==========================================
    let mut s11 = SpriteTexture::new(TEX_SIZE, TEX_SIZE, c_black);
    // Draw a tiny 2x2 red pixel dot at the bottom center (aligned to ground)
    s11.draw_rect(31, 62, 2, 2, c_red);
    sprites.push(s11);

    // ==========================================
    // SPRITE 12: Meat Chunk (Pixelated organic chunk)
    // ==========================================
    let mut s12 = SpriteTexture::new(TEX_SIZE, TEX_SIZE, c_black);
    // Draw a small 4x4 red chunk with dark border at the bottom center (aligned to ground)
    s12.draw_rect(30, 59, 4, 5, 0x550000ff); // Border
    s12.draw_rect(31, 60, 2, 3, c_red);      // Crimson inner
    s12.set_pixel(31, 60, 0xff5555ff);       // Highlight flesh
    sprites.push(s12);

    // ==========================================
    // SPRITE 13: Guided Missile (Glowing Cyber Sphere)
    // ==========================================
    let mut s13 = SpriteTexture::new(TEX_SIZE, TEX_SIZE, c_black);
    // Draw glowing sphere
    s13.draw_circle(32, 32, 10, c_red);
    s13.draw_circle(32, 32, 7, c_neon_pink);
    s13.draw_circle(32, 32, 4, c_neon_yellow);
    s13.draw_circle(32, 32, 2, c_white);
    sprites.push(s13);

    // ==========================================
    // SPRITE 14: Smoke Trail Stage 1 (Hot Fire)
    // ==========================================
    let mut s14 = SpriteTexture::new(TEX_SIZE, TEX_SIZE, c_black);
    s14.draw_circle(32, 32, 8, c_neon_yellow);
    s14.draw_circle(32, 32, 4, c_white);
    sprites.push(s14);

    // ==========================================
    // SPRITE 15: Smoke Trail Stage 2 (Orange/Pink Spark)
    // ==========================================
    let mut s15 = SpriteTexture::new(TEX_SIZE, TEX_SIZE, c_black);
    s15.draw_circle(32, 32, 10, c_neon_pink);
    s15.draw_circle(32, 32, 6, c_neon_yellow);
    sprites.push(s15);

    // ==========================================
    // SPRITE 16: Smoke Trail Stage 3 (Dark Grey Smoke)
    // ==========================================
    let mut s16 = SpriteTexture::new(TEX_SIZE, TEX_SIZE, c_black);
    s16.draw_circle(32, 32, 12, 0x333333ff);
    s16.draw_circle(32, 32, 8, 0x222222ff);
    sprites.push(s16);

    // ==========================================
    // SPRITE 17: Cyber Hover-Cruiser A (Cyan / Neon Blue)
    // ==========================================
    let mut s17 = SpriteTexture::new(TEX_SIZE, TEX_SIZE, c_black);
    // Fuselage
    s17.draw_rect(12, 32, 40, 16, c_gray);
    s17.draw_rect(14, 30, 36, 18, c_dark_blue);
    // Thrusters (Cyan)
    s17.draw_rect(8, 36, 6, 10, c_neon_cyan);
    s17.draw_rect(50, 36, 6, 10, c_neon_cyan);
    // Canopy
    s17.draw_rect(24, 24, 16, 8, c_black);
    s17.draw_rect(26, 26, 12, 6, c_neon_cyan);
    // Hover pads underglow
    s17.draw_rect(16, 48, 8, 3, c_neon_cyan);
    s17.draw_rect(40, 48, 8, 3, c_neon_cyan);
    s17.draw_circle(20, 50, 4, c_neon_cyan);
    s17.draw_circle(44, 50, 4, c_neon_cyan);
    sprites.push(s17);

    // ==========================================
    // SPRITE 18: Cyber Hover-Cruiser B (Neon Pink)
    // ==========================================
    let mut s18 = SpriteTexture::new(TEX_SIZE, TEX_SIZE, c_black);
    // Fuselage
    s18.draw_rect(12, 32, 40, 16, c_dark_gray);
    s18.draw_rect(14, 30, 36, 18, 0x330808ff);
    // Thrusters (Pink)
    s18.draw_rect(8, 36, 6, 10, c_neon_pink);
    s18.draw_rect(50, 36, 6, 10, c_neon_pink);
    // Canopy
    s18.draw_rect(24, 24, 16, 8, c_black);
    s18.draw_rect(26, 26, 12, 6, c_neon_pink);
    // Hover pads underglow
    s18.draw_rect(16, 48, 8, 3, c_neon_pink);
    s18.draw_rect(40, 48, 8, 3, c_neon_pink);
    s18.draw_circle(20, 50, 4, c_neon_pink);
    s18.draw_circle(44, 50, 4, c_neon_pink);
    sprites.push(s18);

    // ==========================================
    // SPRITE 19: Steam Cloud Small
    // ==========================================
    let mut s19 = SpriteTexture::new(TEX_SIZE, TEX_SIZE, c_black);
    s19.draw_circle(32, 32, 6, 0xcccccc20); // translucent grey
    s19.draw_circle(32, 32, 3, 0xffffff40); // brighter core
    sprites.push(s19);

    // ==========================================
    // SPRITE 20: Steam Cloud Medium
    // ==========================================
    let mut s20 = SpriteTexture::new(TEX_SIZE, TEX_SIZE, c_black);
    s20.draw_circle(32, 32, 10, 0xcccccc15);
    s20.draw_circle(32, 32, 6, 0xffffff30);
    sprites.push(s20);

    // ==========================================
    // SPRITE 21: Steam Cloud Large
    // ==========================================
    let mut s21 = SpriteTexture::new(TEX_SIZE, TEX_SIZE, c_black);
    s21.draw_circle(32, 32, 14, 0xcccccc10);
    s21.draw_circle(32, 32, 9, 0xffffff20);
    sprites.push(s21);

    // ==========================================
    // SPRITE 22: Neon Sign - Vertical Cyan "CYBER"
    // ==========================================
    let mut s22 = SpriteTexture::new(TEX_SIZE, TEX_SIZE, c_black);
    // Support bracket
    s22.draw_rect(0, 4, 32, 2, c_dark_gray);
    s22.draw_line(0, 4, 16, 16, c_gray);
    // Glowing border (Cyan)
    s22.draw_rect(14, 10, 12, 48, c_neon_cyan);
    s22.draw_rect(15, 11, 10, 46, c_black);
    // Letter C
    s22.draw_rect(18, 14, 4, 2, c_neon_pink);
    s22.draw_rect(18, 16, 2, 4, c_neon_pink);
    s22.draw_rect(18, 20, 4, 2, c_neon_pink);
    // Letter Y
    s22.draw_line(18, 24, 20, 26, c_neon_pink);
    s22.draw_line(22, 24, 20, 26, c_neon_pink);
    s22.draw_rect(20, 26, 2, 4, c_neon_pink);
    // Letter B
    s22.draw_rect(18, 32, 4, 6, c_neon_pink);
    s22.draw_rect(20, 33, 2, 1, c_black);
    s22.draw_rect(20, 36, 2, 1, c_black);
    // Letter E
    s22.draw_rect(18, 40, 4, 6, c_neon_pink);
    s22.draw_rect(20, 41, 2, 1, c_black);
    s22.draw_rect(20, 43, 2, 1, c_black);
    // Letter R
    s22.draw_rect(18, 48, 4, 6, c_neon_pink);
    s22.draw_rect(20, 49, 2, 1, c_black);
    s22.draw_line(20, 51, 22, 53, c_neon_pink);
    sprites.push(s22);

    // ==========================================
    // SPRITE 23: Neon Sign - Vertical Pink "BAR"
    // ==========================================
    let mut s23 = SpriteTexture::new(TEX_SIZE, TEX_SIZE, c_black);
    // Support bracket
    s23.draw_rect(0, 4, 32, 2, c_dark_gray);
    s23.draw_line(0, 4, 16, 16, c_gray);
    // Glowing border (Pink)
    s23.draw_rect(14, 10, 12, 48, c_neon_pink);
    s23.draw_rect(15, 11, 10, 46, c_black);
    // Letter B
    s23.draw_rect(18, 16, 4, 6, c_neon_cyan);
    s23.draw_rect(20, 17, 2, 1, c_black);
    s23.draw_rect(20, 20, 2, 1, c_black);
    // Letter A
    s23.draw_rect(18, 26, 4, 6, c_neon_cyan);
    s23.draw_rect(20, 27, 2, 1, c_black);
    s23.draw_rect(20, 30, 2, 2, c_black);
    // Letter R
    s23.draw_rect(18, 36, 4, 6, c_neon_cyan);
    s23.draw_rect(20, 37, 2, 1, c_black);
    s23.draw_line(20, 39, 22, 41, c_neon_cyan);
    sprites.push(s23);

    // ==========================================
    // SPRITE 24: Neon Sign - Green Cyber-Glyphs
    // ==========================================
    let mut s24 = SpriteTexture::new(TEX_SIZE, TEX_SIZE, c_black);
    // Support bracket
    s24.draw_rect(0, 4, 32, 2, c_dark_gray);
    s24.draw_line(0, 4, 16, 16, c_gray);
    // Glowing border (Green)
    s24.draw_rect(14, 10, 12, 48, c_neon_green);
    s24.draw_rect(15, 11, 10, 46, c_black);
    // Glyphs (Yellow)
    s24.draw_circle(20, 18, 2, c_neon_yellow);
    s24.draw_rect(18, 26, 4, 2, c_neon_yellow);
    s24.draw_line(18, 32, 22, 36, c_neon_yellow);
    s24.draw_line(22, 32, 18, 36, c_neon_yellow);
    s24.draw_rect(20, 42, 2, 6, c_neon_yellow);
    sprites.push(s24);

    GameAssets {
        walls,
        sprites,
    }
}

#[allow(dead_code)]
pub async fn load_game_assets() -> GameAssets {
    use macroquad::texture::load_image;

    // Load sprite sheets from disk/server
    let walls_img = load_image("src/assets/walls.png").await.expect("Failed to load walls.png");
    let sprites_img = load_image("src/assets/sprites.png").await.expect("Failed to load sprites.png");

    let mut walls = Vec::new();
    let mut sprites = Vec::new();

    let wall_size = 64;
    let sprite_size = 64;

    // Extract walls (4 walls, arranged in a 2x2 grid)
    let walls_cols = 2;
    for i in 0..4 {
        let grid_col = i % walls_cols;
        let grid_row = i / walls_cols;
        walls.push(extract_sprite(&walls_img, grid_col * wall_size, grid_row * wall_size, wall_size, wall_size));
    }

    // Extract sprites (21 sprites, arranged in a 5x5 grid)
    let sprites_cols = 5;
    for i in 0..21 {
        let grid_col = i % sprites_cols;
        let grid_row = i / sprites_cols;
        sprites.push(extract_sprite(&sprites_img, grid_col * sprite_size, grid_row * sprite_size, sprite_size, sprite_size));
    }

    GameAssets {
        walls,
        sprites,
    }
}

#[allow(dead_code)]
fn extract_sprite(src: &macroquad::texture::Image, sx: usize, sy: usize, sw: usize, sh: usize) -> SpriteTexture {
    let mut pixels = Vec::with_capacity(sw * sh);
    let src_w = src.width as usize;
    for y in sy..(sy + sh) {
        for x in sx..(sx + sw) {
            let idx = (y * src_w + x) * 4;
            let r = src.bytes[idx] as u32;
            let g = src.bytes[idx + 1] as u32;
            let b = src.bytes[idx + 2] as u32;
            let a = src.bytes[idx + 3] as u32;
            let color = (r << 24) | (g << 16) | (b << 8) | a;
            pixels.push(color);
        }
    }
    SpriteTexture {
        width: sw,
        height: sh,
        pixels,
    }
}

