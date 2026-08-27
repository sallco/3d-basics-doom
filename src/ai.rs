use std::collections::{HashSet, VecDeque};

use raylib::prelude::Vector2;

use crate::level::{Guard, GuardState, Tile};

pub const PATROL_SPEED: f32 = 1.2;
pub const CHASE_SPEED: f32 = 2.2;
pub const SLOWED_SPEED_MULTIPLIER: f32 = 0.5;
pub const DETECTION_RADIUS: f32 = 6.5;
pub const CAPTURE_RADIUS: f32 = 0.55;

pub fn is_cell_walkable(maze: &[Vec<Tile>], row: usize, col: usize) -> bool {
    if row >= maze.len() || col >= maze[row].len() {
        return false;
    }
    matches!(maze[row][col], Tile::Floor)
}

pub fn has_line_of_sight(maze: &[Vec<Tile>], from: Vector2, to: Vector2) -> bool {
    let diff = Vector2::new(to.x - from.x, to.y - from.y);
    let distance = (diff.x * diff.x + diff.y * diff.y).sqrt();
    if distance < 0.05 {
        return true;
    }

    let steps = (distance * 14.0).ceil() as usize;
    for i in 1..steps {
        let t = i as f32 / steps as f32;
        let p = Vector2::new(from.x + diff.x * t, from.y + diff.y * t);
        let cell_x = p.x.floor() as usize;
        let cell_y = p.y.floor() as usize;

        if !is_cell_walkable(maze, cell_y, cell_x) {
            return false;
        }
    }
    true
}

pub fn bfs_path(
    maze: &[Vec<Tile>],
    start: (usize, usize),
    goal: (usize, usize),
) -> Option<Vec<(usize, usize)>> {
    let height = maze.len();
    if height == 0 {
        return None;
    }
    let width = maze[0].len();
    if width == 0 || start.0 >= height || start.1 >= width || goal.0 >= height || goal.1 >= width {
        return None;
    }

    if start == goal {
        return Some(vec![start]);
    }

    let mut queue = VecDeque::new();
    let mut visited = HashSet::new();
    let mut parent = std::collections::HashMap::new();

    queue.push_back(start);
    visited.insert(start);

    let mut found = false;

    while let Some(current) = queue.pop_front() {
        if current == goal {
            found = true;
            break;
        }

        let (r, c) = current;
        let mut neighbors = Vec::with_capacity(4);
        if r > 0 {
            neighbors.push((r - 1, c));
        }
        if r + 1 < height {
            neighbors.push((r + 1, c));
        }
        if c > 0 {
            neighbors.push((r, c - 1));
        }
        if c + 1 < width {
            neighbors.push((r, c + 1));
        }

        for (nr, nc) in neighbors {
            if is_cell_walkable(maze, nr, nc) && visited.insert((nr, nc)) {
                parent.insert((nr, nc), current);
                queue.push_back((nr, nc));
            }
        }
    }

    if !found {
        return None;
    }

    let mut path = Vec::new();
    let mut curr = goal;
    path.push(curr);

    while let Some(&p) = parent.get(&curr) {
        path.push(p);
        curr = p;
        if curr == start {
            break;
        }
    }

    path.reverse();
    Some(path)
}

pub fn update_guards(
    guards: &mut [Guard],
    maze: &[Vec<Tile>],
    player_pos: Vector2,
    delta_time: f32,
) -> bool {
    let mut player_captured = false;

    for guard in guards.iter_mut() {
        if guard.slowed_timer > 0.0 {
            guard.slowed_timer = (guard.slowed_timer - delta_time).max(0.0);
            if guard.slowed_timer == 0.0 && guard.state == GuardState::Slowed {
                guard.state = GuardState::Chase;
            }
        }

        let to_player = Vector2::new(
            player_pos.x - guard.position.x,
            player_pos.y - guard.position.y,
        );
        let dist_to_player = (to_player.x * to_player.x + to_player.y * to_player.y).sqrt();

        if dist_to_player <= CAPTURE_RADIUS {
            player_captured = true;
        }

        match guard.state {
            GuardState::Patrol | GuardState::Resetting => {
                if dist_to_player <= DETECTION_RADIUS
                    && has_line_of_sight(maze, guard.position, player_pos)
                {
                    guard.state = GuardState::Chase;
                }
            }
            GuardState::Alerted => {
                guard.state = GuardState::Chase;
            }
            GuardState::Chase | GuardState::Slowed => {}
        }

        let (target_pos, base_speed) = match guard.state {
            GuardState::Chase => (player_pos, CHASE_SPEED),
            GuardState::Slowed => (player_pos, CHASE_SPEED * SLOWED_SPEED_MULTIPLIER),
            GuardState::Alerted => (player_pos, PATROL_SPEED),
            GuardState::Patrol | GuardState::Resetting => {
                let dist_to_spawn =
                    (guard.spawn.x - guard.position.x).hypot(guard.spawn.y - guard.position.y);
                if dist_to_spawn > 0.3 {
                    (guard.spawn, PATROL_SPEED)
                } else {
                    (guard.spawn, 0.0)
                }
            }
        };

        if base_speed <= 0.0 {
            continue;
        }

        let next_waypoint = if has_line_of_sight(maze, guard.position, target_pos) {
            target_pos
        } else {
            let start_cell = (
                guard.position.y.floor() as usize,
                guard.position.x.floor() as usize,
            );
            let goal_cell = (target_pos.y.floor() as usize, target_pos.x.floor() as usize);

            if let Some(path) = bfs_path(maze, start_cell, goal_cell) {
                if path.len() > 1 {
                    Vector2::new(path[1].1 as f32 + 0.5, path[1].0 as f32 + 0.5)
                } else {
                    target_pos
                }
            } else {
                target_pos
            }
        };

        let move_dir = Vector2::new(
            next_waypoint.x - guard.position.x,
            next_waypoint.y - guard.position.y,
        );
        let move_dist = (move_dir.x * move_dir.x + move_dir.y * move_dir.y).sqrt();

        if move_dist > 0.05 {
            let normalized_dir = Vector2::new(move_dir.x / move_dist, move_dir.y / move_dist);
            let step = base_speed * delta_time;

            let next_x = guard.position.x + normalized_dir.x * step;
            let next_y = guard.position.y + normalized_dir.y * step;

            let margin = 0.22;
            let can_move_x = is_cell_walkable(
                maze,
                (guard.position.y - margin).floor() as usize,
                (next_x - margin).floor() as usize,
            ) && is_cell_walkable(
                maze,
                (guard.position.y + margin).floor() as usize,
                (next_x + margin).floor() as usize,
            );

            let can_move_y = is_cell_walkable(
                maze,
                (next_y - margin).floor() as usize,
                (guard.position.x - margin).floor() as usize,
            ) && is_cell_walkable(
                maze,
                (next_y + margin).floor() as usize,
                (guard.position.x + margin).floor() as usize,
            );

            if can_move_x {
                guard.position.x = next_x;
            }
            if can_move_y {
                guard.position.y = next_y;
            }
        }
    }

    player_captured
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::level::WallMaterial;

    fn test_grid() -> Vec<Vec<Tile>> {
        let wall = Tile::Wall(WallMaterial::Gallery);
        let mut grid = vec![vec![Tile::Floor; 6]; 6];
        grid[0].fill(wall);
        grid[5].fill(wall);
        for row in &mut grid {
            row[0] = wall;
            row[5] = wall;
        }
        // Barrier in the middle with opening at (3, 3)
        grid[2][1] = wall;
        grid[2][2] = wall;
        grid[2][3] = wall;
        grid
    }

    #[test]
    fn bfs_finds_path_around_obstacles() {
        let grid = test_grid();
        let path = bfs_path(&grid, (1, 1), (3, 1));
        assert!(path.is_some());
        let p = path.unwrap();
        assert_eq!(*p.first().unwrap(), (1, 1));
        assert_eq!(*p.last().unwrap(), (3, 1));
        assert!(p.len() >= 4);
    }

    #[test]
    fn line_of_sight_detects_walls_and_clear_corridors() {
        let grid = test_grid();
        // Clear horizontal line
        assert!(has_line_of_sight(
            &grid,
            Vector2::new(1.5, 1.5),
            Vector2::new(4.5, 1.5)
        ));
        // Blocked vertical line through barrier at (2, 2)
        assert!(!has_line_of_sight(
            &grid,
            Vector2::new(2.5, 1.5),
            Vector2::new(2.5, 3.5)
        ));
    }

    #[test]
    fn guard_chases_player_and_detects_capture() {
        let grid = test_grid();
        let mut guards = vec![Guard::new(Vector2::new(1.5, 1.5))];
        guards[0].state = GuardState::Chase;

        let player_pos = Vector2::new(1.6, 1.5);
        let captured = update_guards(&mut guards, &grid, player_pos, 0.1);
        assert!(captured);
    }
}
