use crate::maze::Maze;
use crate::enemy::Enemy;

pub fn can_move_to_x(
    maze: &Maze,
    new_x: f32,
    dx: f32,
    current_y: f32,
    margin: f32,
    block_size: usize,
) -> bool {
    let check_x = new_x + if dx > 0.0 { margin } else { -margin };
    let i = check_x as usize / block_size;
    let j = current_y as usize / block_size;

    if let Some(&cell) = maze.get(j).and_then(|row| row.get(i)) {
        cell == ' ' || cell == 'g' || cell == 'G'
    } else {
        false
    }
}

pub fn can_move_to_y(
    maze: &Maze,
    new_y: f32,
    dy: f32,
    current_x: f32,
    margin: f32,
    block_size: usize,
) -> bool {
    let check_y = new_y + if dy > 0.0 { margin } else { -margin };
    let i = current_x as usize / block_size;
    let j = check_y as usize / block_size;

    if let Some(&cell) = maze.get(j).and_then(|row| row.get(i)) {
        cell == ' ' || cell == 'g' || cell == 'G'
    } else {
        false
    }
}

pub fn check_enemy_collision(
    new_x: f32,
    new_y: f32,
    enemies: &[Enemy],
    enemy_margin: f32,
) -> bool {
    for enemy in enemies {
        let dist = (new_x - enemy.pos.x).hypot(new_y - enemy.pos.y);
        if dist < enemy_margin {
            return true;
        }
    }
    false
}
