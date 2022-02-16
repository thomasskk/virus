use crate::cell::Cell;
use rayon::prelude::*;

#[derive(Clone)]
pub struct World {
    cells: Vec<Cell>,
    init: bool,
}

const GREEN: [u8; 4] = [60, 179, 113, 255];
const BLACK: [u8; 4] = [0, 0, 0, 255];
const RED: [u8; 4] = [255, 0, 0, 255];
const BLUE: [u8; 4] = [0, 0, 255, 255];

impl World {
    pub fn new(width: usize, height: usize) -> Self {
        let size = width * height;
        let mut world = World {
            cells: vec![],
            init: true,
        };

        for index in 0..size {
            world.cells.push(Cell::new(index, width, height));
        }

        world
    }

    pub fn set_virus_cluster(&mut self, index: usize) {
        self.cells.get_mut(index).map(|cell| cell.infect());
    }

    pub fn set_defense_cluster(&mut self, index: usize) {
        self.cells.get_mut(index).map(|cell| cell.defend());
    }

    pub fn update(&mut self, frame: &mut [u8]) {
        let cells = &self.cells.clone();
        self.cells
            .par_iter_mut()
            .zip(frame.par_chunks_exact_mut(4))
            .for_each(|(cell, pixel)| {
                match () {
                    _ if cell.infected => cell.set_defense(cells),
                    _ if cell.alive && !cell.immune => cell.set_contagion(cells),
                    _ => cell.changed = false,
                };
                if cell.changed || self.init {
                    let rgb = match () {
                        _ if cell.immune => GREEN,
                        _ if !cell.alive => BLACK,
                        _ if cell.infected => RED,
                        _ if cell.defend => BLUE,
                        _ => GREEN,
                    };
                    pixel.copy_from_slice(&rgb);
                }
            });
        self.init = false;
    }
}
