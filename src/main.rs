mod caster;
mod enemy;
mod framebuffer;
mod maze;
mod player;
mod texture;

use minifb::{Key, Window, WindowOptions};
use std::f32::consts::PI;
use std::time::{Duration, Instant};

use crate::caster::{cast_ray, cast_ray_3d};
use crate::framebuffer::Framebuffer;
use crate::maze::{Maze, load_maze};
use crate::player::{Player, process_events};
use crate::texture::Texture;

const BLOCK_SIZE: usize = 100;

/// Cantidad de rayos que se lanzan en abanico para formar el campo de visión.
const NUM_RAYS: usize = 5;

/// Amplitud del campo de visión (field of view), en radianes.
const FOV: f32 = PI / 3.0;

fn cell_color(cell: char) -> u32 {
    match cell {
        '0' => 0x222222,       // columnas
        '1' => 0x222222,       // paredes 1
        '2' => 0x222222,       // paredes 2
        '3' => 0x222222,       // paredes 3
        '4' => 0xFFFFFF,       // luz
        'g' | 'G' => 0x00FF00, // meta
        _ => 0xFFDDDD,         // cualquier otra cosa
    }
}

fn draw_cell(framebuffer: &mut Framebuffer, xo: usize, yo: usize, block_size: usize, cell: char) {
    if cell == ' ' {
        return;
    }

    framebuffer.set_current_color(cell_color(cell));

    for x in xo..xo + block_size {
        for y in yo..yo + block_size {
            framebuffer.point(x, y);
        }
    }
}

fn render_map(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    player: &Player,
    block_size: usize,
    offset_x: usize,
    offset_y: usize,
) {
    let scale = block_size as f32 / BLOCK_SIZE as f32;

    for (row, line) in maze.iter().enumerate() {
        for (col, &cell) in line.iter().enumerate() {
            draw_cell(
                framebuffer,
                offset_x + col * block_size,
                offset_y + row * block_size,
                block_size,
                cell,
            );
        }
    }

    framebuffer.set_current_color(0xFFFF00);

    let px = offset_x + (player.pos.x * scale) as usize;
    let py = offset_y + (player.pos.y * scale) as usize;
    let p_radius = (3.0 * scale).max(1.0) as usize;

    for x in px.saturating_sub(p_radius)..=px + p_radius {
        for y in py.saturating_sub(p_radius)..=py + p_radius {
            framebuffer.point(x, y);
        }
    }

    for i in 0..NUM_RAYS {
        let ray_fraction = i as f32 / (NUM_RAYS - 1).max(1) as f32;
        let angle = player.a - FOV / 2.0 + FOV * ray_fraction;
        cast_ray(
            framebuffer,
            maze,
            player,
            angle,
            BLOCK_SIZE,
            offset_x,
            offset_y,
            scale,
        );
    }
}

fn render2d(framebuffer: &mut Framebuffer, maze: &Maze, player: &Player) {
    render_map(framebuffer, maze, player, BLOCK_SIZE, 0, 0);
}

fn get_texture_bounds(cell: char) -> (u32, u32, u32, u32) {
    // Retorna (start_x, start_y, width, height) en el sprite sheet
    // Ajusta estos valores a las coordenadas reales de tu imagen
    match cell {
        '0' => (328, 152, 24, 64),
        '1' => (360, 152, 24, 64),
        '2' => (392, 152, 24, 64),
        '3' => (424, 152, 24, 64),
        '4' => (8, 224, 72, 64),
        'g' => (464, 224, 46, 64),
        _ => (0, 0, 24, 64),
    }
}

fn render3d(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    player: &Player,
    texture: &Texture,
    enemies: &[crate::enemy::Enemy],
    enemy_texture: &Texture,
) {
    let num_rays = framebuffer.width;
    let hw = framebuffer.width as f32 / 2.0;
    let hh = framebuffer.height as f32 / 2.0;
    let d_to_plane = hw / (FOV / 2.0).tan();

    let mut z_buffer = vec![0.0; framebuffer.width];

    for i in 0..num_rays {
        let ray_fraction = i as f32 / (num_rays - 1).max(1) as f32; // de 0.0 a 1.0
        let angle = player.a - (FOV / 2.0) + FOV * ray_fraction;

        let (mut d, cell, hit_x, hit_y) = cast_ray_3d(maze, player, angle, BLOCK_SIZE);

        // Corrección del ojo de pez (fisheye)
        d *= (angle - player.a).cos();

        // Prevenir división por cero
        if d < 1.0 {
            d = 1.0;
        }

        z_buffer[i] = d;

        // Cálculo de altura de la estaca (proyección)
        let wall_height = (BLOCK_SIZE as f32 / d) * d_to_plane;

        // Trimming (recorte) para no dibujar fuera de los límites de la pantalla
        let wall_top = (hh - wall_height / 2.0) as isize;
        let wall_bottom = (hh + wall_height / 2.0) as isize;

        let wall_top_usize = wall_top.max(0) as usize;
        let wall_bottom_usize = wall_bottom.min(framebuffer.height as isize) as usize;

        // Dibujar cielo
        framebuffer.set_current_color(0x333355);
        for y in 0..wall_top_usize {
            framebuffer.point(i, y);
        }

        // Dibujar pared (estaca) con texturas
        if cell != ' ' {
            let (tex_x, tex_y, tex_w, tex_h) = get_texture_bounds(cell);

            // Determinar UV horizontal
            let hit_x_block = hit_x % BLOCK_SIZE as f32;
            let hit_y_block = hit_y % BLOCK_SIZE as f32;

            let dist_x = hit_x_block.min(BLOCK_SIZE as f32 - hit_x_block);
            let dist_y = hit_y_block.min(BLOCK_SIZE as f32 - hit_y_block);

            let tx_ratio = if dist_x < dist_y {
                hit_y_block / BLOCK_SIZE as f32
            } else {
                hit_x_block / BLOCK_SIZE as f32
            };

            let tx = (tx_ratio * tex_w as f32) as u32;

            let wall_real_height = wall_bottom - wall_top;

            for y in wall_top_usize..wall_bottom_usize {
                let ty_ratio = (y as isize - wall_top) as f32 / wall_real_height as f32;
                let ty = (ty_ratio * tex_h as f32) as u32;

                let color = texture.get_pixel_color(tex_x + tx, tex_y + ty);
                framebuffer.set_current_color(color);
                framebuffer.point(i, y);
            }
        }

        // Dibujar suelo
        framebuffer.set_current_color(0x222222);
        for y in wall_bottom_usize..framebuffer.height {
            framebuffer.point(i, y);
        }
    }

    // Renderizar enemigos (Billboarding)
    let mut sorted_enemies: Vec<&crate::enemy::Enemy> = enemies.iter().collect();
    // Ordenar por distancia, el más lejano primero
    sorted_enemies.sort_by(|a, b| {
        let dist_a = (a.pos.x - player.pos.x).powi(2) + (a.pos.y - player.pos.y).powi(2);
        let dist_b = (b.pos.x - player.pos.x).powi(2) + (b.pos.y - player.pos.y).powi(2);
        dist_b
            .partial_cmp(&dist_a)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    for enemy in sorted_enemies {
        let dx = enemy.pos.x - player.pos.x;
        let dy = enemy.pos.y - player.pos.y;
        let distance = (dx * dx + dy * dy).sqrt();

        // Ángulo del enemigo con respecto al jugador
        let enemy_angle = dy.atan2(dx);

        // Diferencia de ángulo entre el jugador y el enemigo
        let mut angle_diff = enemy_angle - player.a;

        // Normalizar el ángulo de diferencia entre -PI y PI
        while angle_diff < -PI {
            angle_diff += 2.0 * PI;
        }
        while angle_diff > PI {
            angle_diff -= 2.0 * PI;
        }

        // Si el enemigo está detrás de la cámara, ignorarlo
        if angle_diff.abs() > FOV {
            continue;
        }

        let corrected_dist = distance * angle_diff.cos();
        if corrected_dist < 1.0 {
            continue;
        } // Evitar divisiones por cero o muy pequeñas

        // Tamaño del sprite en pantalla
        let sprite_height = (BLOCK_SIZE as f32 / corrected_dist) * d_to_plane;
        let sprite_width = sprite_height; // Asumimos un sprite cuadrado

        let screen_x = hw + (angle_diff / (FOV / 2.0)) * hw;

        let sprite_top = (hh - sprite_height / 2.0) as isize;
        let sprite_bottom = (hh + sprite_height / 2.0) as isize;

        let sprite_left = (screen_x - sprite_width / 2.0) as isize;
        let sprite_right = (screen_x + sprite_width / 2.0) as isize;

        let tex_w = enemy_texture.width as f32;
        let tex_h = enemy_texture.height as f32;

        for x in sprite_left..sprite_right {
            if x >= 0 && x < framebuffer.width as isize {
                let ux = x as usize;

                // Z-buffer check
                if corrected_dist < z_buffer[ux] {
                    let tx = ((x - sprite_left) as f32 / sprite_width * tex_w) as u32;

                    let y_top = sprite_top.max(0) as usize;
                    let y_bottom = sprite_bottom.min(framebuffer.height as isize) as usize;

                    for y in y_top..y_bottom {
                        let ty = ((y as isize - sprite_top) as f32 / sprite_height * tex_h) as u32;
                        let color = enemy_texture.get_pixel_color(tx, ty);

                        // Ignorar píxeles transparentes (asumiendo que alpha > 0 es opaco)
                        let alpha = (color >> 24) & 0xFF;
                        if alpha > 0 {
                            framebuffer.set_current_color(color);
                            framebuffer.point(ux, y);
                        }
                    }
                }
            }
        }
    }

    // Dibujar minimapa
    let minimap_block_size = BLOCK_SIZE / 5;
    let minimap_width = maze.first().map_or(0, |row| row.len()) * minimap_block_size;
    let offset_x = framebuffer.width.saturating_sub(minimap_width);
    render_map(framebuffer, maze, player, minimap_block_size, offset_x, 0);
}

fn draw_success_screen(framebuffer: &mut Framebuffer) {
    framebuffer.set_background_color(0x000000); // Negro
    framebuffer.clear();

    let win_text = [
        " !  N   N III V   V EEEE L      CCC  OOO  M   M PPPP  L    EEEE TTTTT  AAA  DDDD   OOO   ! ",
        "    NN  N  I  V   V E    L     C    O   O MM MM P   P L    E      T   A   A D   D O   O  ! ",
        " !  N N N  I  V   V EEEE L     C    O   O M M M PPPP  L    EEEE   T   AAAAA D   D O   O  ! ",
        " !  N  NN  I   V V  E    L     C    O   O M   M P     L    E      T   A   A D   D O   O    ",
        " !  N   N III   V   EEEE LLLL   CCC  OOO  M   M P     LLLL EEEE   T   A   A DDDD   OOO   ! ",
    ];

    let pixel_size = 12;
    let start_x = framebuffer.width / 2 - (win_text[0].len() * pixel_size) / 2;
    let start_y = framebuffer.height / 2 - (win_text.len() * pixel_size) / 2;

    framebuffer.set_current_color(0xFFD700); // Dorado

    for (row, line) in win_text.iter().enumerate() {
        for (col, ch) in line.chars().enumerate() {
            if ch != ' ' {
                let x = start_x + col * pixel_size;
                let y = start_y + row * pixel_size;
                for dy in 0..pixel_size {
                    for dx in 0..pixel_size {
                        framebuffer.point(x + dx, y + dy);
                    }
                }
            }
        }
    }
}

fn draw_text(
    framebuffer: &mut Framebuffer,
    x: usize,
    y: usize,
    text: &str,
    size: usize,
    color: u32,
) {
    let font = [
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
    let slash = [0, 0, 1, 0, 0, 1, 0, 1, 0, 1, 0, 0, 1, 0, 0];

    framebuffer.set_current_color(color);

    let mut cursor_x = x;
    for ch in text.chars() {
        let bitmap = match ch {
            '0'..='9' => &font[(ch as usize) - ('0' as usize)],
            '/' => &slash,
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

fn draw_health_bar(framebuffer: &mut Framebuffer, hp: f32) {
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
    draw_text(
        framebuffer,
        x + bar_width / 2 - text_width / 2,
        y + 4,
        &text,
        2,
        0xFFFFFF,
    );
}

fn draw_game_over_screen(framebuffer: &mut Framebuffer) {
    framebuffer.set_background_color(0x000000); // Negro
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

fn draw_text_pixel_art(
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

fn draw_main_menu(framebuffer: &mut Framebuffer) {
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

fn draw_text_simple(
    framebuffer: &mut Framebuffer,
    text: &str,
    x: usize,
    y: usize,
    size: usize,
    color: u32,
) {
    let font_data: &[(char, &[u8])] = &[
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
        ('(', &[0b010, 0b100, 0b100, 0b100, 0b010]),
        (')', &[0b010, 0b001, 0b001, 0b001, 0b010]),
        (' ', &[0b000, 0b000, 0b000, 0b000, 0b000]),
    ];

    framebuffer.set_current_color(color);
    let mut cursor_x = x;
    for ch in text.chars() {
        let ch_upper = ch.to_ascii_uppercase();
        if let Some((_, glyph)) = font_data.iter().find(|(c, _)| *c == ch_upper) {
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

fn draw_controls_screen(framebuffer: &mut Framebuffer) {
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

fn load_level(
    level: usize,
    stream_handle: Option<&rodio::OutputStreamHandle>,
    audio_data: Option<std::sync::Arc<Vec<u8>>>,
) -> (
    crate::maze::Maze,
    crate::player::Player,
    Vec<crate::enemy::Enemy>,
) {
    let filename = if level == 1 {
        "./maze.txt"
    } else if level == 2 {
        "./maze2.txt"
    } else {
        "./maze3.txt"
    };
    let (maze, player) = load_maze(filename, BLOCK_SIZE);

    let mut empty_spaces = Vec::new();
    for (j, row) in maze.iter().enumerate() {
        for (i, &cell) in row.iter().enumerate() {
            // Evitar spawns muy cerca del jugador o en la meta
            if cell == ' ' && (i > 2 || j > 2) {
                empty_spaces.push((i, j));
            }
        }
    }

    let mut enemies = Vec::new();
    let max_enemies = if level == 1 {
        3
    } else if level == 2 {
        8
    } else {
        12
    };
    let num_enemies = empty_spaces.len().min(max_enemies);
    for idx in 0..num_enemies {
        // Simple dispersión
        let step = (empty_spaces.len() / num_enemies).max(1);
        let (i, j) = empty_spaces[(idx * step) % empty_spaces.len()];
        let ex = (i * BLOCK_SIZE + BLOCK_SIZE / 2) as f32;
        let ey = (j * BLOCK_SIZE + BLOCK_SIZE / 2) as f32;
        enemies.push(crate::enemy::Enemy::new(
            ex,
            ey,
            stream_handle,
            audio_data.clone(),
        ));
    }

    (maze, player, enemies)
}

fn draw_paused_screen(framebuffer: &mut Framebuffer) {
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
    let pixel_size = 12;
    let text_x = framebuffer.width / 2 - (text[0].len() * pixel_size) / 2;
    let text_y = framebuffer.height / 2 - (text.len() * pixel_size) / 2;

    framebuffer.set_current_color(0xFFFFFF); // Blanco
    for (row, line) in text.iter().enumerate() {
        for (col, ch) in line.chars().enumerate() {
            if ch != ' ' {
                let x = text_x + col * pixel_size;
                let y = text_y + row * pixel_size;
                for dy in 0..pixel_size {
                    for dx in 0..pixel_size {
                        framebuffer.point(x + dx, y + dy);
                    }
                }
            }
        }
    }
}

fn main() {
    let window_width = 1300;
    let window_height = 900;
    let framebuffer_width = 1300;
    let framebuffer_height = 900;

    let (_stream, stream_handle) = match rodio::OutputStream::try_default() {
        Ok((s, h)) => (Some(s), Some(h)),
        Err(e) => {
            println!("No se pudo inicializar el audio: {:?}", e);
            (None, None)
        }
    };

    let audio_data = std::fs::read("./assets/SAT.mp3")
        .ok()
        .map(std::sync::Arc::new);

    let mut current_level = 1;
    let (mut maze, mut player, mut enemies) =
        load_level(current_level, stream_handle.as_ref(), audio_data.clone());

    // Cargar texturas
    let texture =
        Texture::new("./assets/textures.png").expect("No se pudo cargar ./assets/textures.png");
    let enemy_texture =
        Texture::new("./assets/sat.png").expect("No se pudo cargar ./assets/sat.png");

    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);
    framebuffer.set_background_color(0x333355);

    let mut window = Window::new(
        "SAT Runner",
        window_width,
        window_height,
        WindowOptions::default(),
    )
    .unwrap();

    let mut is_3d_mode = true;
    let mut last_m_pressed = false;
    let mut win_state = false;
    let mut game_over_state = false;
    let mut menu_state = true;
    let mut controls_state = false;
    let mut is_paused = false;
    let mut last_p_pressed = false;
    let mut last_c_pressed = false;
    let mut last_mouse_x: Option<f32> = None;

    let mut last_time = Instant::now();
    let target_frame_time = Duration::from_millis(1000 / 60); // ~16.6 ms per frame for 60 FPS

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let frame_start = Instant::now();
        let dt = frame_start.duration_since(last_time).as_secs_f32();
        last_time = frame_start;

        if menu_state {
            let c_pressed = window.is_key_down(Key::C);
            if c_pressed && !last_c_pressed {
                controls_state = !controls_state;
            }
            last_c_pressed = c_pressed;

            if !controls_state && window.is_key_down(Key::Enter) {
                menu_state = false;
                last_time = Instant::now(); // Reset dt so enemies don't jump
                last_mouse_x = None;
            }
        } else if !win_state && !game_over_state {
            let p_pressed = window.is_key_down(Key::P);
            if p_pressed && !last_p_pressed {
                is_paused = !is_paused;
                if !is_paused {
                    last_time = Instant::now();
                    last_mouse_x = None;
                }
            }
            last_p_pressed = p_pressed;

            if !is_paused {
                let m_pressed = window.is_key_down(Key::M);
                if m_pressed && !last_m_pressed {
                    is_3d_mode = !is_3d_mode;
                }
                last_m_pressed = m_pressed;
                process_events(
                    &window,
                    &mut player,
                    &maze,
                    &enemies,
                    BLOCK_SIZE,
                    &mut last_mouse_x,
                );

                // Actualizar enemigos
                for enemy in enemies.iter_mut() {
                    enemy.update(&mut player, dt, &maze, BLOCK_SIZE);
                }

                if player.hp <= 0.0 {
                    println!("¡Has muerto! Game Over.");
                    game_over_state = true;
                }

                // ¿el jugador llegó a la meta? Se traduce su posición en píxeles a la
                // celda que ocupa y se revisa si esa celda es la marca `g`.
                let i = player.pos.x as usize / BLOCK_SIZE;
                let j = player.pos.y as usize / BLOCK_SIZE;
                if maze.get(j).and_then(|row| row.get(i)) == Some(&'g') {
                    if current_level < 3 {
                        println!(
                            "¡Nivel {} completado! Cargando nivel {}...",
                            current_level,
                            current_level + 1
                        );
                        current_level += 1;
                        let current_hp = player.hp;
                        let (new_maze, mut new_player, new_enemies) =
                            load_level(current_level, stream_handle.as_ref(), audio_data.clone());
                        maze = new_maze;
                        new_player.hp = current_hp;
                        player = new_player;
                        enemies = new_enemies;

                        last_time = Instant::now();
                    } else {
                        println!("¡Meta alcanzada! Has ganado.");
                        win_state = true;
                    }
                }
            }
        } else if game_over_state {
            if window.is_key_down(Key::R) {
                current_level = 1;
                let (new_maze, new_player, new_enemies) =
                    load_level(current_level, stream_handle.as_ref(), audio_data.clone());
                maze = new_maze;
                player = new_player;
                enemies = new_enemies;
                game_over_state = false;
                last_time = Instant::now();
                last_mouse_x = None;
            } else if window.is_key_down(Key::M) {
                game_over_state = false;
                menu_state = true;
                last_time = Instant::now();
                last_mouse_x = None;
            }
        }

        framebuffer.clear();

        if menu_state {
            if controls_state {
                draw_controls_screen(&mut framebuffer);
            } else {
                draw_main_menu(&mut framebuffer);
            }
        } else if win_state {
            draw_success_screen(&mut framebuffer);
        } else if game_over_state {
            draw_game_over_screen(&mut framebuffer);
        } else {
            if is_3d_mode {
                render3d(
                    &mut framebuffer,
                    &maze,
                    &player,
                    &texture,
                    &enemies,
                    &enemy_texture,
                );
            } else {
                render2d(&mut framebuffer, &maze, &player);
            }
            draw_health_bar(&mut framebuffer, player.hp);

            if is_paused {
                draw_paused_screen(&mut framebuffer);
            }
        }

        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();

        // Controlar FPS a 60
        let elapsed = frame_start.elapsed();
        if elapsed < target_frame_time {
            std::thread::sleep(target_frame_time - elapsed);
        }

        // Mostrar FPS y HP en el título
        let final_elapsed = frame_start.elapsed();
        let fps = 1.0 / final_elapsed.as_secs_f32();
        window.set_title(&format!("SAT Runner - {:.0} FPS", fps,));
    }
}
