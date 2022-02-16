use crate::nbrs::Nbrs;
use rand::{thread_rng, Rng};

#[derive(Clone, Copy)]
pub struct Cell {
    pub alive: bool,
    pub infected: bool,
    pub contagious: bool,
    pub days_infected: usize,
    pub immune: bool,
    pub defend: bool,
    pub nbrs: Nbrs,
    pub changed: bool,
}

impl Cell {
    pub fn new(index: usize, width: usize, height: usize) -> Self {
        Cell {
            alive: true,
            contagious: false,
            infected: false,
            days_infected: 0,
            immune: thread_rng().gen_range(0..100) < 20,
            defend: thread_rng().gen_range(0..100) < 1,
            nbrs: Nbrs::new(index, width, height),
            changed: true,
        }
    }

    pub fn set_contagion(&mut self, cells: &Vec<Cell>) {
        let contagion_total = self.nbrs.get_contagion_total(cells);
        let rng: i16 = thread_rng().gen_range(67..220);
        if contagion_total > rng {
            self.infect()
        };
    }

    pub fn set_defense(&mut self, cells: &Vec<Cell>) {
        let rng: i16 = thread_rng().gen_range(150..500);

        match () {
            _ if self.nbrs.get_defense_total(cells) > rng
                && self.nbrs.get_contagion_total(cells) != 0 =>
            {
                self.defend()
            }
            _ => self.update_infection(),
        }
    }

    pub fn infect(&mut self) {
        if self.alive && !self.immune && !self.infected {
            self.contagious = true;
            self.infected = true;
            self.days_infected = 0;
            self.changed = true;
        }
    }

    pub fn defend(&mut self) {
        if self.alive {
            self.defend = true;
            self.infected = false;
            self.contagious = false;
            self.changed = true;
        }
    }

    pub fn update_infection(&mut self) {
        if self.infected {
            self.days_infected += 1;
        }
        if self.days_infected == thread_rng().gen_range(4..13) {
            let numb = thread_rng().gen_range(0..100);
            if numb > 40 {
                self.alive = false;
                self.changed = true;
            } else {
                self.immune = true;
                self.infected = false;
                self.contagious = false;
                self.changed = true;
            }
        }
    }
}
