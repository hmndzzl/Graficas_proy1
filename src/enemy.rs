use crate::maze::Maze;
use crate::player::Player;
use nalgebra_glm::Vec2;

pub struct Enemy {
    pub pos: Vec2,
}

impl Enemy {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            pos: Vec2::new(x, y),
        }
    }

    pub fn update(&mut self, player: &mut Player, dt: f32, maze: &Maze, block_size: usize) {
        let speed = 90.0; // Píxeles por segundo
        let dx = player.pos.x - self.pos.x;
        let dy = player.pos.y - self.pos.y;
        let dist = (dx * dx + dy * dy).sqrt();

        if dist > 0.0 {
            // Mover hacia el jugador
            let move_x = (dx / dist) * speed * dt;
            let move_y = (dy / dist) * speed * dt;

            // Colisiones simples
            let margin = 20.0;

            let check_x = self.pos.x + move_x + if move_x > 0.0 { margin } else { -margin };
            let i_x = check_x as usize / block_size;
            let j_x = self.pos.y as usize / block_size;

            if let Some(&cell) = maze.get(j_x).and_then(|row| row.get(i_x)) {
                if cell == ' ' || cell == 'g' || cell == 'G' {
                    self.pos.x += move_x;
                }
            }

            let check_y = self.pos.y + move_y + if move_y > 0.0 { margin } else { -margin };
            let i_y = self.pos.x as usize / block_size;
            let j_y = check_y as usize / block_size;

            if let Some(&cell) = maze.get(j_y).and_then(|row| row.get(i_y)) {
                if cell == ' ' || cell == 'g' || cell == 'G' {
                    self.pos.y += move_y;
                }
            }
        }

        // Daño si están cerca
        if dist < (block_size as f32) * 0.5 {
            let dps = 80.0; // Daño por segundo
            player.hp -= dps * dt;
            if player.hp < 0.0 {
                player.hp = 0.0;
            }
        }
    }
}
