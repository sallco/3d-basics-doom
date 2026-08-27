use std::fs::File;
use std::io::{BufRead, BufReader};

use raylib::prelude::Color;

use crate::framebuffer::Framebuffer;

pub type Maze = Vec<Vec<char>>;

pub fn load_maze(filename: &str) -> std::io::Result<Maze> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);

    reader
        .lines()
        .map(|line| line.map(|contents| contents.chars().collect()))
        .collect()
}

fn draw_cell(framebuffer: &mut Framebuffer, xo: usize, yo: usize, block_size: usize, cell: char) {
    let column = xo / block_size;
    let row = yo / block_size;
    let color = cell_color(cell, row, column);

    framebuffer.set_current_color(color);

    let max_x = (xo + block_size).min(framebuffer.width as usize);
    let max_y = (yo + block_size).min(framebuffer.height as usize);

    for y in yo..max_y {
        for x in xo..max_x {
            framebuffer.point(x as u32, y as u32);
        }
    }
}

pub fn cell_color(cell: char, row: usize, column: usize) -> Color {
    let checker_even = (row + column).is_multiple_of(2);

    match cell {
        '+' | '-' | '|' if checker_even => Color::LIGHTGRAY,
        '+' | '-' | '|' => Color::GRAY,
        'g' => Color::GREEN,
        _ => Color::BLACK,
    }
}

pub fn render_maze(framebuffer: &mut Framebuffer, maze: &Maze, block_size: usize) {
    for (row_index, row) in maze.iter().enumerate() {
        for (col_index, &cell) in row.iter().enumerate() {
            let xo = col_index * block_size;
            let yo = row_index * block_size;

            draw_cell(framebuffer, xo, yo, block_size, cell);
        }
    }
}
