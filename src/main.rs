mod caster;
mod framebuffer;
mod maze;
mod player;

use minifb::{Key, Window, WindowOptions};
use std::f32::consts::PI;
use std::time::{Duration, Instant};

use crate::caster::{cast_ray, cast_ray_3d};
use crate::framebuffer::Framebuffer;
use crate::maze::{Maze, load_maze};
use crate::player::{Player, process_events};

const BLOCK_SIZE: usize = 100;

/// Cantidad de rayos que se lanzan en abanico para formar el campo de visión.
const NUM_RAYS: usize = 5;

/// Amplitud del campo de visión (field of view), en radianes.
const FOV: f32 = PI / 3.0;

fn cell_color(cell: char) -> u32 {
    match cell {
        '+' => 0xFFFFFF,       // columnas
        '-' => 0x00AAFF,       // paredes horizontales
        '|' => 0xFFAAAA,       // paredes verticales
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
            draw_cell(framebuffer, offset_x + col * block_size, offset_y + row * block_size, block_size, cell);
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
        cast_ray(framebuffer, maze, player, angle, BLOCK_SIZE, offset_x, offset_y, scale);
    }
}

fn render2d(framebuffer: &mut Framebuffer, maze: &Maze, player: &Player) {
    render_map(framebuffer, maze, player, BLOCK_SIZE, 0, 0);
}

fn render3d(framebuffer: &mut Framebuffer, maze: &Maze, player: &Player) {
    let num_rays = framebuffer.width;
    let hw = framebuffer.width as f32 / 2.0;
    let hh = framebuffer.height as f32 / 2.0;
    let d_to_plane = hw / (FOV / 2.0).tan();

    for i in 0..num_rays {
        let ray_fraction = i as f32 / (num_rays - 1).max(1) as f32; // de 0.0 a 1.0
        let angle = player.a - (FOV / 2.0) + FOV * ray_fraction;

        let (mut d, cell) = cast_ray_3d(maze, player, angle, BLOCK_SIZE);

        // Corrección del ojo de pez (fisheye)
        d *= (angle - player.a).cos();

        // Prevenir división por cero
        if d < 1.0 {
            d = 1.0;
        }

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

        // Dibujar pared (estaca)
        if cell != ' ' {
            framebuffer.set_current_color(cell_color(cell));
            for y in wall_top_usize..wall_bottom_usize {
                framebuffer.point(i, y);
            }
        }

        // Dibujar suelo
        framebuffer.set_current_color(0x222222);
        for y in wall_bottom_usize..framebuffer.height {
            framebuffer.point(i, y);
        }
    }

    // Dibujar minimapa
    let minimap_block_size = BLOCK_SIZE / 5;
    let minimap_width = maze.first().map_or(0, |row| row.len()) * minimap_block_size;
    let offset_x = framebuffer.width.saturating_sub(minimap_width);
    render_map(framebuffer, maze, player, minimap_block_size, offset_x, 0);
}

fn main() {
    let window_width = 1300;
    let window_height = 900;
    let framebuffer_width = 1300;
    let framebuffer_height = 900;

    let (maze, mut player) = load_maze("./maze.txt", BLOCK_SIZE);

    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);
    framebuffer.set_background_color(0x333355);

    let mut window = Window::new(
        "Maze Runner",
        window_width,
        window_height,
        WindowOptions::default(),
    )
    .unwrap();

    let mut is_3d_mode = true;
    let mut last_m_pressed = false;

    let target_frame_time = Duration::from_millis(1000 / 60); // ~16.6 ms per frame for 60 FPS

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let frame_start = Instant::now();

        let m_pressed = window.is_key_down(Key::M);
        if m_pressed && !last_m_pressed {
            is_3d_mode = !is_3d_mode;
        }
        last_m_pressed = m_pressed;
        process_events(&window, &mut player, &maze, BLOCK_SIZE);

        // ¿el jugador llegó a la meta? Se traduce su posición en píxeles a la
        // celda que ocupa y se revisa si esa celda es la marca `g`.
        let i = player.pos.x as usize / BLOCK_SIZE;
        let j = player.pos.y as usize / BLOCK_SIZE;
        if maze.get(j).and_then(|row| row.get(i)) == Some(&'g') {
            println!("¡Meta alcanzada! Fin del juego.");
            break;
        }

        framebuffer.clear();

        if is_3d_mode {
            render3d(&mut framebuffer, &maze, &player);
        } else {
            render2d(&mut framebuffer, &maze, &player);
        }

        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();

        // Controlar FPS a 60
        let elapsed = frame_start.elapsed();
        if elapsed < target_frame_time {
            std::thread::sleep(target_frame_time - elapsed);
        }

        // Mostrar FPS en el título
        let final_elapsed = frame_start.elapsed();
        let fps = 1.0 / final_elapsed.as_secs_f32();
        window.set_title(&format!("Maze Runner - {:.0} FPS", fps));
    }
}
