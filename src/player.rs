use crate::maze::Maze;
use minifb::{Key, Window};
use nalgebra_glm::Vec2;
use std::f32::consts::PI;

pub struct Player {
    pub pos: Vec2,
    pub a: f32,
    pub hp: f32,
}

pub fn process_events(
    window: &Window,
    player: &mut Player,
    maze: &Maze,
    enemies: &[crate::enemy::Enemy],
    block_size: usize,
    last_mouse_x: &mut Option<f32>,
) {
    const MOVE_SPEED: f32 = 3.0;
    const ROTATION_SPEED: f32 = PI / 45.0;

    if window.is_key_down(Key::A) || window.is_key_down(Key::Left) {
        player.a -= ROTATION_SPEED;
    }

    if window.is_key_down(Key::D) || window.is_key_down(Key::Right) {
        player.a += ROTATION_SPEED;
    }

    if let Some((mouse_x, _)) = window.get_mouse_pos(minifb::MouseMode::Pass) {
        if let Some(last_x) = last_mouse_x {
            let dx_mouse = mouse_x - *last_x;
            let sensitivity = 0.003; 
            player.a += dx_mouse * sensitivity;
        }
        *last_mouse_x = Some(mouse_x);
    } else {
        *last_mouse_x = None;
    }

    let mut dx = 0.0;
    let mut dy = 0.0;

    if window.is_key_down(Key::W) || window.is_key_down(Key::Up) {
        dx += MOVE_SPEED * player.a.cos();
        dy += MOVE_SPEED * player.a.sin();
    }

    if window.is_key_down(Key::S) || window.is_key_down(Key::Down) {
        dx -= MOVE_SPEED * player.a.cos();
        dy -= MOVE_SPEED * player.a.sin();
    }

    if dx != 0.0 || dy != 0.0 {
        // Margen para evitar que el jugador se acerque demasiado a la pared (y cause glitches visuales)
        let margin = 20.0;
        let enemy_margin = 50.0; // Distancia mínima entre jugador y enemigo

        // Comprobar colisión en el eje X
        let mut can_move_x = crate::physics::can_move_to_x(
            maze,
            player.pos.x + dx,
            dx,
            player.pos.y,
            margin,
            block_size,
        );

        if can_move_x && crate::physics::check_enemy_collision(player.pos.x + dx, player.pos.y, enemies, enemy_margin) {
            can_move_x = false;
        }

        if can_move_x {
            player.pos.x += dx;
        }

        // Comprobar colisión en el eje Y
        let mut can_move_y = crate::physics::can_move_to_y(
            maze,
            player.pos.y + dy,
            dy,
            player.pos.x,
            margin,
            block_size,
        );

        if can_move_y && crate::physics::check_enemy_collision(player.pos.x, player.pos.y + dy, enemies, enemy_margin) {
            can_move_y = false;
        }

        if can_move_y {
            player.pos.y += dy;
        }
    }
}
