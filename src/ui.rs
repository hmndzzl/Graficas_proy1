use crate::framebuffer::Framebuffer;

const FONT_DATA: &[(char, &[u8])] = &[
    ('A', &[0b010, 0b101, 0b111, 0b101, 0b101]),
    ('B', &[0b110, 0b101, 0b110, 0b101, 0b110]),
    ('C', &[0b011, 0b100, 0b100, 0b100, 0b011]),
    ('D', &[0b110, 0b101, 0b101, 0b101, 0b110]),
    ('E', &[0b111, 0b100, 0b111, 0b100, 0b111]),
    ('F', &[0b111, 0b100, 0b110, 0b100, 0b100]),
    ('G', &[0b011, 0b100, 0b101, 0b101, 0b011]),
    ('H', &[0b101, 0b101, 0b111, 0b101, 0b101]),
    ('I', &[0b111, 0b010, 0b010, 0b010, 0b111]),
    ('J', &[0b001, 0b001, 0b001, 0b101, 0b011]),
    ('K', &[0b101, 0b110, 0b100, 0b110, 0b101]),
    ('L', &[0b100, 0b100, 0b100, 0b100, 0b111]),
    ('M', &[0b101, 0b111, 0b101, 0b101, 0b101]),
    ('N', &[0b110, 0b101, 0b101, 0b101, 0b101]),
    ('O', &[0b010, 0b101, 0b101, 0b101, 0b010]),
    ('P', &[0b110, 0b101, 0b110, 0b100, 0b100]),
    ('Q', &[0b010, 0b101, 0b101, 0b011, 0b001]),
    ('R', &[0b110, 0b101, 0b110, 0b101, 0b101]),
    ('S', &[0b011, 0b100, 0b010, 0b001, 0b110]),
    ('T', &[0b111, 0b010, 0b010, 0b010, 0b010]),
    ('U', &[0b101, 0b101, 0b101, 0b101, 0b011]),
    ('V', &[0b101, 0b101, 0b101, 0b101, 0b010]),
    ('W', &[0b101, 0b101, 0b101, 0b111, 0b101]),
    ('X', &[0b101, 0b101, 0b010, 0b101, 0b101]),
    ('Y', &[0b101, 0b101, 0b010, 0b010, 0b010]),
    ('Z', &[0b111, 0b001, 0b010, 0b100, 0b111]),
    ('-', &[0b000, 0b000, 0b111, 0b000, 0b000]),
    ('=', &[0b000, 0b111, 0b000, 0b111, 0b000]),
    ('(', &[0b010, 0b100, 0b100, 0b100, 0b010]),
    (')', &[0b010, 0b001, 0b001, 0b001, 0b010]),
    (' ', &[0b000, 0b000, 0b000, 0b000, 0b000]),
];

const NUMBER_FONT: [[u8; 15]; 10] = [
    [1, 1, 1, 1, 0, 1, 1, 0, 1, 1, 0, 1, 1, 1, 1], // 0
    [0, 1, 0, 1, 1, 0, 0, 1, 0, 0, 1, 0, 1, 1, 1], // 1
    [1, 1, 1, 0, 0, 1, 1, 1, 1, 1, 0, 0, 1, 1, 1], // 2
    [1, 1, 1, 0, 0, 1, 1, 1, 1, 0, 0, 1, 1, 1, 1], // 3
    [1, 0, 1, 1, 0, 1, 1, 1, 1, 0, 0, 1, 0, 0, 1], // 4
    [1, 1, 1, 1, 0, 0, 1, 1, 1, 0, 0, 1, 1, 1, 1], // 5
    [1, 1, 1, 1, 0, 0, 1, 1, 1, 1, 0, 1, 1, 1, 1], // 6
    [1, 1, 1, 0, 0, 1, 0, 1, 0, 0, 1, 0, 0, 1, 0], // 7
    [1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1], // 8
    [1, 1, 1, 1, 0, 1, 1, 1, 1, 0, 0, 1, 1, 1, 1], // 9
];
const NUMBER_SLASH: [u8; 15] = [0, 0, 1, 0, 0, 1, 0, 1, 0, 1, 0, 0, 1, 0, 0];

pub fn draw_text_simple(
    framebuffer: &mut Framebuffer,
    text: &str,
    x: usize,
    y: usize,
    size: usize,
    color: u32,
) {
    framebuffer.set_current_color(color);
    let mut cursor_x = x;
    for ch in text.chars() {
        let ch_upper = ch.to_ascii_uppercase();
        if let Some((_, glyph)) = FONT_DATA.iter().find(|(c, _)| *c == ch_upper) {
            for (row, &bits) in glyph.iter().enumerate() {
                for col in 0..3 {
                    if (bits >> (2 - col)) & 1 == 1 {
                        let px = cursor_x + col * size;
                        let py = y + row * size;
                        for dy in 0..size {
                            for dx in 0..size {
                                framebuffer.point(px + dx, py + dy);
                            }
                        }
                    }
                }
            }
        }
        cursor_x += 4 * size;
    }
}

pub fn draw_text_pixel_art(
    framebuffer: &mut Framebuffer,
    text: &[&str],
    size: usize,
    y: usize,
    color: u32,
) {
    let x = framebuffer.width / 2 - (text[0].len() * size) / 2;
    framebuffer.set_current_color(color);
    for (row, line) in text.iter().enumerate() {
        for (col, ch) in line.chars().enumerate() {
            if ch != ' ' {
                let px = x + col * size;
                let py = y + row * size;
                for dy in 0..size {
                    for dx in 0..size {
                        framebuffer.point(px + dx, py + dy);
                    }
                }
            }
        }
    }
}

pub fn draw_numbers(
    framebuffer: &mut Framebuffer,
    x: usize,
    y: usize,
    text: &str,
    size: usize,
    color: u32,
) {
    framebuffer.set_current_color(color);

    let mut cursor_x = x;
    for ch in text.chars() {
        let bitmap = match ch {
            '0'..='9' => &NUMBER_FONT[(ch as usize) - ('0' as usize)],
            '/' => &NUMBER_SLASH,
            _ => continue,
        };

        for (i, &pixel) in bitmap.iter().enumerate() {
            if pixel == 1 {
                let px = cursor_x + (i % 3) * size;
                let py = y + (i / 3) * size;
                for dx in 0..size {
                    for dy in 0..size {
                        framebuffer.point(px + dx, py + dy);
                    }
                }
            }
        }
        cursor_x += 4 * size;
    }
}

pub fn draw_health_bar(framebuffer: &mut Framebuffer, hp: f32) {
    let bar_width = 300;
    let bar_height = 20;
    let x = (framebuffer.width - bar_width) / 2;
    let y = framebuffer.height - 40;

    let clamped_hp = hp.clamp(0.0, 100.0);
    let ratio = clamped_hp / 100.0;
    let current_width = (bar_width as f32 * ratio) as usize;

    let r = ((1.0 - ratio) * 255.0) as u32;
    let g = (ratio * 255.0) as u32;
    let b = 0;
    let color = (r << 16) | (g << 8) | b;

    // Draw background (empty bar)
    framebuffer.set_current_color(0x333333);
    for dx in 0..bar_width {
        for dy in 0..bar_height {
            framebuffer.point(x + dx, y + dy);
        }
    }

    // Draw health
    framebuffer.set_current_color(color);
    for dx in 0..current_width {
        for dy in 0..bar_height {
            framebuffer.point(x + dx, y + dy);
        }
    }

    // Draw text
    let text = format!("{:.0}/100", clamped_hp);
    let text_len = text.len();
    let text_width = text_len * 4 * 2; // size=2
    draw_numbers(
        framebuffer,
        x + bar_width / 2 - text_width / 2,
        y + 4,
        &text,
        2,
        0xFFFFFF,
    );
}

pub fn draw_main_menu(framebuffer: &mut Framebuffer) {
    framebuffer.set_background_color(0x000000);
    framebuffer.clear();

    let title = [
        " SSSS   AAA  TTTTT    RRRR  U   U N   N N   N EEEE RRRR  ",
        "S      A   A   T      R   R U   U NN  N NN  N E    R   R ",
        " SSS   AAAAA   T      RRRR  U   U N N N N N N EEEE RRRR  ",
        "    S  A   A   T      R  R  U   U N  NN N  NN E    R  R  ",
        "SSSS   A   A   T      R   R  UUU  N   N N   N EEEE R   R ",
    ];

    let fb_height = framebuffer.height;
    let fb_width = framebuffer.width;

    draw_text_pixel_art(framebuffer, &title, 17, fb_height / 3, 0x00FFFF);

    let items = ["START ENTER", "CONTROLS C", "EXIT ESC"];

    let text_size = 6;
    for (i, item) in items.iter().enumerate() {
        let text_width = item.len() * 4 * text_size;
        let x = fb_width / 2 - text_width / 2;
        let y = fb_height / 2 + i * (10 * text_size) + 40;
        let color = 0xFFFFFF;
        draw_text_simple(framebuffer, item, x, y, text_size, color);
    }
}

pub fn draw_controls_screen(framebuffer: &mut Framebuffer) {
    framebuffer.set_background_color(0x000000);
    framebuffer.clear();

    let title = [
        "CCCC OOOO N   N TTTTT RRRR  OOOO L    SSSS",
        "C    O  O NN  N   T   R   R O  O L    S   ",
        "C    O  O N N N   T   RRRR  O  O L     SSS",
        "C    O  O N  NN   T   R  R  O  O L        S",
        "CCCC OOOO N   N   T   R   R OOOO LLLL SSSS",
    ];

    let fb_height = framebuffer.height;
    draw_text_pixel_art(framebuffer, &title, 12, fb_height / 6, 0x00FFFF);

    let items = [
        "W A S D - MOVERSE",
        "MOUSE - CAMARA",
        "P - PAUSAR",
        "M - MINIMAPA",
        "C - VOLVER",
    ];

    let text_size = 6;
    let fb_width = framebuffer.width;
    for (i, item) in items.iter().enumerate() {
        let text_width = item.len() * 4 * text_size;
        let x = fb_width / 2 - text_width / 2;
        let y = fb_height / 2 + i * (10 * text_size) - 50;
        draw_text_simple(framebuffer, item, x, y, text_size, 0xFFFFFF);
    }
}

pub fn draw_game_over_screen(framebuffer: &mut Framebuffer) {
    framebuffer.set_background_color(0x000000);
    framebuffer.clear();

    let text = [
        " GGG   AAA  M   M EEEE    OOO  V   V EEEE RRRR  ",
        "G     A   A MM MM E      O   O V   V E    R   R ",
        "G  GG AAAAA M M M EEEE   O   O V   V EEEE RRRR  ",
        "G   G A   A M   M E      O   O  V V  E    R  R  ",
        " GGG  A   A M   M EEEE    OOO    V   EEEE R   R ",
    ];

    let fb_height = framebuffer.height;
    let fb_width = framebuffer.width;

    draw_text_pixel_art(framebuffer, &text, 12, fb_height / 4, 0xFF0000);

    let items = ["RESTART = R", "MENU = M"];

    let text_size = 6;
    for (i, item) in items.iter().enumerate() {
        let text_width = item.len() * 4 * text_size;
        let x = fb_width / 2 - text_width / 2;
        let y = fb_height / 2 + i * (10 * text_size) + 40;
        draw_text_simple(framebuffer, item, x, y, text_size, 0xFFFFFF);
    }
}

pub fn draw_success_screen(framebuffer: &mut Framebuffer) {
    framebuffer.set_background_color(0x000000);
    framebuffer.clear();

    let win_text = [
        " !  N   N III V   V EEEE L      CCC  OOO  M   M PPPP  L    EEEE TTTTT  AAA  DDDD   OOO   ! ",
        "    NN  N  I  V   V E    L     C    O   O MM MM P   P L    E      T   A   A D   D O   O  ! ",
        " !  N N N  I  V   V EEEE L     C    O   O M M M PPPP  L    EEEE   T   AAAAA D   D O   O  ! ",
        " !  N  NN  I   V V  E    L     C    O   O M   M P     L    E      T   A   A D   D O   O    ",
        " !  N   N III   V   EEEE LLLL   CCC  OOO  M   M P     LLLL EEEE   T   A   A DDDD   OOO   ! ",
    ];

    let fb_height = framebuffer.height;
    draw_text_pixel_art(framebuffer, &win_text, 12, fb_height / 2 - 30, 0xFFD700);
}

pub fn draw_paused_screen(framebuffer: &mut Framebuffer) {
    // Dim the screen
    for pixel in framebuffer.buffer.iter_mut() {
        let r = ((*pixel >> 16) & 0xFF) / 2;
        let g = ((*pixel >> 8) & 0xFF) / 2;
        let b = (*pixel & 0xFF) / 2;
        *pixel = (r << 16) | (g << 8) | b;
    }

    let text = [
        "PPPP   AAA  U   U SSSS EEEE DDD ",
        "P   P A   A U   U S    E    D  D",
        "PPPP  AAAAA U   U SSSS EEEE D  D",
        "P     A   A U   U    S E    D  D",
        "P     A   A  UUU  SSSS EEEE DDD ",
    ];
    let fb_height = framebuffer.height;
    draw_text_pixel_art(framebuffer, &text, 12, fb_height / 2 - 30, 0xFFFFFF);
}
