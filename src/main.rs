mod caster;
mod enemy;
mod framebuffer;
mod maze;
mod physics;
mod player;
mod texture;
mod ui;

use minifb::{Key, Window, WindowOptions};
use rodio::Source;
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
    enemies: &[crate::enemy::Enemy],
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

    // Dibujar enemigos en rojo
    framebuffer.set_current_color(0xFF0000);
    for enemy in enemies {
        let dx = enemy.pos.x - player.pos.x;
        let dy = enemy.pos.y - player.pos.y;
        let dist = dx.hypot(dy);

        let angle_to_enemy = dy.atan2(dx);
        let mut angle_diff = (angle_to_enemy - player.a).rem_euclid(2.0 * PI);
        if angle_diff > PI {
            angle_diff -= 2.0 * PI;
        }

        // Si está dentro del FOV (con un pequeño margen)
        if angle_diff.abs() < FOV / 1.5 {
            let (hit_dist, _, _, _) =
                crate::caster::cast_ray_3d(maze, player, angle_to_enemy, BLOCK_SIZE);

            // Si la distancia a la pared es mayor que la distancia al enemigo, el enemigo es visible
            if hit_dist >= dist {
                let ex = offset_x + (enemy.pos.x * scale) as usize;
                let ey = offset_y + (enemy.pos.y * scale) as usize;
                for x in ex.saturating_sub(p_radius)..=ex + p_radius {
                    for y in ey.saturating_sub(p_radius)..=ey + p_radius {
                        framebuffer.point(x, y);
                    }
                }
            }
        }
    }
}

fn render2d(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    player: &Player,
    enemies: &[crate::enemy::Enemy],
) {
    let maze_width = maze.first().map_or(0, |row| row.len());
    let maze_height = maze.len();

    let block_w = framebuffer.width / maze_width.max(1);
    let block_h = framebuffer.height / maze_height.max(1);

    let block_size = block_w.min(block_h);

    let map_w = maze_width * block_size;
    let map_h = maze_height * block_size;
    let offset_x = (framebuffer.width - map_w) / 2;
    let offset_y = (framebuffer.height - map_h) / 2;

    render_map(
        framebuffer,
        maze,
        player,
        enemies,
        block_size,
        offset_x,
        offset_y,
    );
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
    skeleton_texture: &Texture,
) {
    let num_rays = framebuffer.width;
    let hw = framebuffer.width as f32 / 2.0;
    let hh = framebuffer.height as f32 / 2.0;
    let d_to_plane = hw / (FOV / 2.0).tan();

    let mut z_buffer = vec![0.0; framebuffer.width];

    for i in 0..num_rays {
        let ray_fraction = i as f32 / (num_rays - 1).max(1) as f32;
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

        let is_jump = enemy.is_jumpscare;
        let tex = if is_jump {
            skeleton_texture
        } else {
            enemy_texture
        };

        let mut tex_w = tex.width as f32;
        let mut tex_h = tex.height as f32;
        let mut tex_x_offset = 0;
        let mut tex_y_offset = 0;

        if is_jump {
            tex_w = 48.0; // Ancho aproximado del cuadro del esqueleto
            tex_h = 48.0; // Alto aproximado del cuadro
            let frame = (enemy.animation_time * 15.0) as u32 % 10; // 10 cuadros de animación

            // La fila empieza en X=6 y termina en X=458 (10 cuadros). 452 / 9 = ~50.22 píxeles de espaciado
            let spacing = 50.22;
            tex_x_offset = (6.0 + (frame as f32) * spacing) as u32;
            tex_y_offset = 118; // La 3era fila empieza en Y=118
        }

        for x in sprite_left..sprite_right {
            if x >= 0 && x < framebuffer.width as isize {
                let ux = x as usize;

                // Z-buffer check
                if corrected_dist < z_buffer[ux] {
                    let tx =
                        ((x - sprite_left) as f32 / sprite_width * tex_w) as u32 + tex_x_offset;

                    let y_top = sprite_top.max(0) as usize;
                    let y_bottom = sprite_bottom.min(framebuffer.height as isize) as usize;

                    for y in y_top..y_bottom {
                        let ty = ((y as isize - sprite_top) as f32 / sprite_height * tex_h) as u32
                            + tex_y_offset;

                        if tx < tex.width && ty < tex.height {
                            let color = tex.get_pixel_color(tx, ty);

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
    }

    // Dibujar minimapa
    let minimap_block_size = BLOCK_SIZE / 5;
    let minimap_width = maze.first().map_or(0, |row| row.len()) * minimap_block_size;
    let offset_x = framebuffer.width.saturating_sub(minimap_width);
    render_map(
        framebuffer,
        maze,
        player,
        enemies,
        minimap_block_size,
        offset_x,
        0,
    );
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
    let (filename, max_enemies) = match level {
        1 => ("./maze.txt", 3),
        2 => ("./maze2.txt", 5),
        _ => ("./maze3.txt", 7),
    };
    let (maze, player) = load_maze(filename, BLOCK_SIZE);

    let px = player.pos.x / BLOCK_SIZE as f32;
    let py = player.pos.y / BLOCK_SIZE as f32;

    let mut empty_spaces = Vec::new();
    for (j, row) in maze.iter().enumerate() {
        for (i, &cell) in row.iter().enumerate() {
            let dist = ((i as f32 - px).powi(2) + (j as f32 - py).powi(2)).sqrt();
            // Evitar spawns muy cerca del jugador
            if cell == ' ' && dist > 8.0 {
                empty_spaces.push((i, j));
            }
        }
    }

    let mut enemies = Vec::new();
    if !empty_spaces.is_empty() {
        let num_enemies = empty_spaces.len().min(max_enemies);
        let mut seed = level as usize * 1234567;

        for _ in 0..num_enemies {
            seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
            let pos_idx = seed % empty_spaces.len();
            let (i, j) = empty_spaces.remove(pos_idx);

            let ex = (i * BLOCK_SIZE + BLOCK_SIZE / 2) as f32;
            let ey = (j * BLOCK_SIZE + BLOCK_SIZE / 2) as f32;
            enemies.push(crate::enemy::Enemy::new(
                ex,
                ey,
                stream_handle,
                audio_data.clone(),
                false,
            ));
        }

        if (level == 2 || level == 3) && !empty_spaces.is_empty() {
            // Buscar la posición de la meta
            let mut goal_x = 0;
            let mut goal_y = 0;
            for (j, row) in maze.iter().enumerate() {
                for (i, &cell) in row.iter().enumerate() {
                    if cell == 'g' || cell == 'G' {
                        goal_x = i;
                        goal_y = j;
                    }
                }
            }

            // Encontrar el espacio vacío más cercano a la meta
            let mut closest_idx = 0;
            let mut min_dist = f32::MAX;
            for (idx, &(i, j)) in empty_spaces.iter().enumerate() {
                let dist = ((i as f32 - goal_x as f32).powi(2) + (j as f32 - goal_y as f32).powi(2)).sqrt();
                if dist < min_dist {
                    min_dist = dist;
                    closest_idx = idx;
                }
            }
            
            let (i, j) = empty_spaces.remove(closest_idx);

            let ex = (i * BLOCK_SIZE + BLOCK_SIZE / 2) as f32;
            let ey = (j * BLOCK_SIZE + BLOCK_SIZE / 2) as f32;
            enemies.push(crate::enemy::Enemy::new(
                ex, ey, None, // Sin sonido
                None, true,
            ));
        }
    }

    (maze, player, enemies)
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

    let vent_audio_data = std::fs::read("./assets/vent.mp3")
        .ok()
        .map(std::sync::Arc::new);

    let ootw_audio_data = std::fs::read("./assets/ootw_ts.mp3")
        .ok()
        .map(std::sync::Arc::new);

    let rfi_audio_data = std::fs::read("./assets/rfi_ts.mp3")
        .ok()
        .map(std::sync::Arc::new);

    let idsb_audio_data = std::fs::read("./assets/idsb_ts.mp3")
        .ok()
        .map(std::sync::Arc::new);

    let ll_audio_data = std::fs::read("./assets/ll_ts.mp3")
        .ok()
        .map(std::sync::Arc::new);

    let play_bgm = |stream: Option<&rodio::OutputStreamHandle>,
                    data: &Option<std::sync::Arc<Vec<u8>>>|
     -> Option<rodio::Sink> {
        if let (Some(handle), Some(d)) = (stream, data) {
            if let Ok(sink) = rodio::Sink::try_new(handle) {
                if let Ok(source) = rodio::Decoder::new(std::io::Cursor::new((**d).clone())) {
                    sink.set_volume(1.0);
                    sink.append(source.repeat_infinite());
                    sink.play();
                    return Some(sink);
                }
            }
        }
        None
    };

    let mut bgm_sink: Option<rodio::Sink> = None;

    let mut current_level = 1;
    let (mut maze, mut player, mut enemies) =
        load_level(current_level, stream_handle.as_ref(), audio_data.clone());

    // Cargar texturas
    let texture =
        Texture::new("./assets/textures.png").expect("No se pudo cargar ./assets/textures.png");
    let enemy_texture =
        Texture::new("./assets/sat.png").expect("No se pudo cargar ./assets/sat.png");
    let skeleton_texture =
        Texture::new("./assets/skeleton.png").expect("No se pudo cargar ./assets/skeleton.png");

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
                if current_level == 1 && bgm_sink.is_none() {
                    bgm_sink = play_bgm(stream_handle.as_ref(), &ootw_audio_data);
                }
            }
        } else if !win_state && !game_over_state {
            let p_pressed = window.is_key_down(Key::P);
            if p_pressed && !last_p_pressed {
                is_paused = !is_paused;
                if is_paused {
                    if let Some(sink) = &bgm_sink {
                        sink.set_volume(0.4);
                    }
                } else {
                    if let Some(sink) = &bgm_sink {
                        sink.set_volume(1.0);
                    }
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
                    if let Some(sink) = bgm_sink.take() {
                        sink.stop();
                    }
                }

                // ¿el jugador llegó a la meta? Se traduce su posición en píxeles a la
                // celda que ocupa y se revisa si esa celda es la marca `g`.
                let i = player.pos.x as usize / BLOCK_SIZE;
                let j = player.pos.y as usize / BLOCK_SIZE;
                if maze.get(j).and_then(|row| row.get(i)) == Some(&'g') {
                    if let (Some(handle), Some(data)) = (stream_handle.as_ref(), &vent_audio_data) {
                        if let Ok(sink) = rodio::Sink::try_new(handle) {
                            let cursor = std::io::Cursor::new((**data).clone());
                            if let Ok(source) = rodio::Decoder::new(cursor) {
                                sink.set_volume(6.0);
                                sink.append(source);
                                sink.detach();
                            }
                        }
                    }

                    if current_level < 3 {
                        println!(
                            "¡Nivel {} completado! Cargando nivel {}...",
                            current_level,
                            current_level + 1
                        );
                        current_level += 1;
                        if let Some(sink) = bgm_sink.take() {
                            sink.stop();
                        }
                        if current_level == 2 {
                            bgm_sink = play_bgm(stream_handle.as_ref(), &rfi_audio_data);
                        } else if current_level == 3 {
                            bgm_sink = play_bgm(stream_handle.as_ref(), &idsb_audio_data);
                        }
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
                        if let Some(sink) = bgm_sink.take() {
                            sink.stop();
                        }
                        bgm_sink = play_bgm(stream_handle.as_ref(), &ll_audio_data);
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
                bgm_sink = play_bgm(stream_handle.as_ref(), &ootw_audio_data);
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
                ui::draw_controls_screen(&mut framebuffer);
            } else {
                ui::draw_main_menu(&mut framebuffer);
            }
        } else if win_state {
            ui::draw_success_screen(&mut framebuffer);
        } else if game_over_state {
            ui::draw_game_over_screen(&mut framebuffer);
        } else {
            if is_3d_mode {
                render3d(
                    &mut framebuffer,
                    &maze,
                    &player,
                    &texture,
                    &enemies,
                    &enemy_texture,
                    &skeleton_texture,
                );
            } else {
                render2d(&mut framebuffer, &maze, &player, &enemies);
            }
            ui::draw_health_bar(&mut framebuffer, player.hp);

            if is_paused {
                ui::draw_paused_screen(&mut framebuffer);
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
