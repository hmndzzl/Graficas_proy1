use crate::maze::Maze;
use crate::player::Player;
use nalgebra_glm::Vec2;
use rodio::{Decoder, OutputStreamHandle, Sink, Source};
use std::io::Cursor;
use std::sync::Arc;

pub struct Enemy {
    pub pos: Vec2,
    pub sink: Option<Sink>,
    pub is_jumpscare: bool,
    pub animation_time: f32,
    pub active: bool,
    pub cooldown: f32,
}

impl Enemy {
    pub fn new(
        x: f32,
        y: f32,
        stream_handle: Option<&OutputStreamHandle>,
        audio_data: Option<Arc<Vec<u8>>>,
        is_jumpscare: bool,
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
            is_jumpscare,
            animation_time: 0.0,
            active: true,
            cooldown: 0.0,
        }
    }

    pub fn update(&mut self, player: &mut Player, dt: f32, maze: &Maze, block_size: usize) {
        if self.is_jumpscare && !self.active {
            self.cooldown -= dt;
            if self.cooldown <= 0.0 {
                self.active = true;
                // Respawn 10 blocks away behind the player
                let spawn_dist = (block_size as f32) * 5.0;
                let spawn_angle = player.a + std::f32::consts::PI; // behind
                self.pos.x = player.pos.x + spawn_angle.cos() * spawn_dist;
                self.pos.y = player.pos.y + spawn_angle.sin() * spawn_dist;
            }
            return;
        }

        if !self.active {
            return;
        }

        if self.is_jumpscare {
            self.animation_time += dt;
        }

        let dx = player.pos.x - self.pos.x;
        let dy = player.pos.y - self.pos.y;
        let dist = (dx * dx + dy * dy).sqrt();

        if self.is_jumpscare && dist > (block_size as f32) * 5.0 {
            // Esperar a que el jugador se acerque para asustarlo
            return;
        }

        let speed = if self.is_jumpscare { 200.0 } else { 90.0 };

        // Desactivar jumpscare si toca al jugador (no hace daño)
        if self.is_jumpscare && dist < (block_size as f32) {
            self.active = false;
            self.cooldown = 1.0; // 1 segundo de espera antes de volver a asustar
            return;
        }

        // Actualizar volumen de audio por proximidad
        if let Some(sink) = &self.sink {
            let max_dist = (block_size as f32) * 4.0;
            if dist < max_dist {
                let volume = 0.8 - (dist / max_dist);
                sink.set_volume(volume.max(0.0));
            } else {
                sink.set_volume(0.0);
            }
        }

        if dist > 5.0 {
            // Mover hacia el jugador
            let move_x = (dx / dist) * speed * dt;
            let move_y = (dy / dist) * speed * dt;

            let margin = 5.0; // Margen de colisión

            if self.is_jumpscare {
                // Jumpscare atraviesa paredes
                self.pos.x += move_x;
                self.pos.y += move_y;
            } else {
                // Movimiento normal con colisiones
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
        }

        // Daño si están cerca (solo normales)
        if !self.is_jumpscare && dist < (block_size as f32) * 0.5 {
            let dps = 50.0; // Daño por segundo
            player.hp -= dps * dt;
            if player.hp < 0.0 {
                player.hp = 0.0;
            }
        }
    }
}
