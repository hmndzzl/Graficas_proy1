use crate::maze::Maze;
use crate::player::Player;
use nalgebra_glm::Vec2;
use rodio::{Decoder, OutputStreamHandle, Sink, Source};
use std::io::Cursor;
use std::sync::Arc;

pub struct Enemy {
    pub pos: Vec2,
    pub sink: Option<Sink>,
}

impl Enemy {
    pub fn new(
        x: f32,
        y: f32,
        stream_handle: Option<&OutputStreamHandle>,
        audio_data: Option<Arc<Vec<u8>>>,
    ) -> Self {
        let mut sink_opt = None;

        if let (Some(handle), Some(data)) = (stream_handle, audio_data) {
            if let Ok(sink) = Sink::try_new(handle) {
                let cursor = Cursor::new((*data).clone());
                if let Ok(source) = Decoder::new(cursor) {
                    sink.append(source.repeat_infinite());
                    sink.set_volume(0.0);
                    sink.play();
                    sink_opt = Some(sink);
                }
            }
        }

        Self {
            pos: Vec2::new(x, y),
            sink: sink_opt,
        }
    }

    pub fn update(&mut self, player: &mut Player, dt: f32, maze: &Maze, block_size: usize) {
        let speed = 90.0; // Píxeles por segundo
        let dx = player.pos.x - self.pos.x;
        let dy = player.pos.y - self.pos.y;
        let dist = (dx * dx + dy * dy).sqrt();

        // Actualizar volumen de audio por proximidad
        if let Some(sink) = &self.sink {
            // max_dist ajusta desde qué tan lejos se empieza a escuchar
            let max_dist = (block_size as f32) * 6.0;
            if dist < max_dist {
                let volume = 1.0 - (dist / max_dist);
                sink.set_volume(volume.max(0.0));
            } else {
                sink.set_volume(0.0);
            }
        }

        if dist > 50.0 {
            // Mover hacia el jugador
            let move_x = (dx / dist) * speed * dt;
            let move_y = (dy / dist) * speed * dt;

            // Colisiones simples
            let margin = 20.0;

            if crate::physics::can_move_to_x(
                maze,
                self.pos.x + move_x,
                move_x,
                self.pos.y,
                margin,
                block_size,
            ) {
                self.pos.x += move_x;
            }

            if crate::physics::can_move_to_y(
                maze,
                self.pos.y + move_y,
                move_y,
                self.pos.x,
                margin,
                block_size,
            ) {
                self.pos.y += move_y;
            }
        }

        // Daño si están cerca
        if dist < (block_size as f32) * 0.5 {
            let dps = 50.0; // Daño por segundo
            player.hp -= dps * dt;
            if player.hp < 0.0 {
                player.hp = 0.0;
            }
        }
    }
}
