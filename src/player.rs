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
) {
    const MOVE_SPEED: f32 = 3.0;
    const ROTATION_SPEED: f32 = PI / 55.0;

    if window.is_key_down(Key::A) || window.is_key_down(Key::Left) {
        player.a -= ROTATION_SPEED;
    }

    if window.is_key_down(Key::D) || window.is_key_down(Key::Right) {
        player.a += ROTATION_SPEED;
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
        let check_x = player.pos.x + dx + if dx > 0.0 { margin } else { -margin };
        let i_x = check_x as usize / block_size;
        let j_x = player.pos.y as usize / block_size;

        let mut can_move_x = false;
        if let Some(&cell) = maze.get(j_x).and_then(|row| row.get(i_x)) {
            if cell == ' ' || cell == 'g' || cell == 'G' {
                can_move_x = true;
            }
        }

        if can_move_x {
            for enemy in enemies {
                let dist = ((player.pos.x + dx) - enemy.pos.x).hypot(player.pos.y - enemy.pos.y);
                if dist < enemy_margin {
                    can_move_x = false;
                    break;
                }
            }
        }

        if can_move_x {
            player.pos.x += dx;
        }

        // Comprobar colisión en el eje Y
        let check_y = player.pos.y + dy + if dy > 0.0 { margin } else { -margin };
        let i_y = player.pos.x as usize / block_size;
        let j_y = check_y as usize / block_size;

        let mut can_move_y = false;
        if let Some(&cell) = maze.get(j_y).and_then(|row| row.get(i_y)) {
            if cell == ' ' || cell == 'g' || cell == 'G' {
                can_move_y = true;
            }
        }

        if can_move_y {
            for enemy in enemies {
                let dist = (player.pos.x - enemy.pos.x).hypot((player.pos.y + dy) - enemy.pos.y);
                if dist < enemy_margin {
                    can_move_y = false;
                    break;
                }
            }
        }

        if can_move_y {
            player.pos.y += dy;
        }
    }
}
