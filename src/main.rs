mod framebuffer;
mod maze;

use maze::load_maze;

fn main() {
    let _maze = load_maze("maze.txt");
}
