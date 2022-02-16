use crate::cell::Cell;
use rand::{thread_rng, Rng};

#[derive(Clone, Copy)]
pub struct Nbrs {
    pub indexes: [usize; 4],
}

const INDEX_IGNORE: usize = 6000;

impl Nbrs {
    pub fn new(index: usize, width: usize, height: usize) -> Self {
        let mut nbrs = Nbrs {
            indexes: [INDEX_IGNORE, INDEX_IGNORE, INDEX_IGNORE, INDEX_IGNORE],
        };
        if index > width {
            nbrs.indexes[0] = index.saturating_sub(width);
        }
        if (index + 1) % width != 0 && index < width * height {
            nbrs.indexes[1] = index + 1;
        }
        if index < width * (height - 1) {
            nbrs.indexes[2] = index + width;
        }
        if index % width != 0 {
            nbrs.indexes[3] = index.saturating_sub(1);
        }
        nbrs
    }

    pub fn get_contagion_total(&self, cells: &Vec<Cell>) -> i16 {
        let mut total = 0;
        for index in self.indexes {
            if index != INDEX_IGNORE && cells[index].contagious {
                total += thread_rng().gen_range(20..100);
            }
        }
        total
    }

    pub fn get_defense_total(&self, cells: &Vec<Cell>) -> i16 {
        let mut total = 0;
        for index in self.indexes {
            if index != INDEX_IGNORE && cells[index].defend {
                total += thread_rng().gen_range(150..250);
            }
        }
        total
    }
}
