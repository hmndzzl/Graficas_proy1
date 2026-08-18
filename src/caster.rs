use crate::framebuffer::Framebuffer;
use crate::maze::Maze;
use crate::player::Player;

pub fn cast_ray(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    player: &Player,
    a: f32,
    block_size: usize,
    draw_offset_x: usize,
    draw_offset_y: usize,
    draw_scale: f32,
) {
    let mut d = 0.0;

    framebuffer.set_current_color(0xFFDDDD);

    loop {
        let x = player.pos.x + d * a.cos();
        let y = player.pos.y + d * a.sin();

        let i = x as usize / block_size;
        let j = y as usize / block_size;

        if j >= maze.len() || i >= maze[j].len() {
            return;
        }

        if maze[j][i] != ' ' {
            return;
        }

        let draw_x = draw_offset_x + (x * draw_scale) as usize;
        let draw_y = draw_offset_y + (y * draw_scale) as usize;
        framebuffer.point(draw_x, draw_y);

        d += 1.0;
    }
}

pub fn cast_ray_3d(maze: &Maze, player: &Player, a: f32, block_size: usize) -> (f32, char) {
    let mut d = 0.0;
    loop {
        let x = player.pos.x + d * a.cos();
        let y = player.pos.y + d * a.sin();

        let i = x as usize / block_size;
        let j = y as usize / block_size;

        if j >= maze.len() || i >= maze[j].len() {
            return (d, ' ');
        }

        let cell = maze[j][i];
        if cell != ' ' {
            // we hit a wall or object
            return (d, cell);
        }

        d += 1.0;
    }
}
