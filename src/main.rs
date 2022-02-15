use pixels::{Error, Pixels, SurfaceTexture};
use rand::{thread_rng, Rng};
use rayon::prelude::*;
use winit::{
    dpi::LogicalSize,
    event::{Event, VirtualKeyCode},
    event_loop::EventLoop,
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;

const WIDTH: usize = 750;
const HEIGHT: usize = 750;

fn main() -> Result<(), Error> {
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let mut paused = false;

    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Virus")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH as u32, HEIGHT as u32, surface_texture)?
    };

    let mut world = World::new(WIDTH, HEIGHT);

    event_loop.run(move |event, _, _control_flow| {
        if let Event::RedrawRequested(_) = event {
            world.update(pixels.get_frame());
            pixels.render().unwrap();
        }

        if input.update(&event) {
            if input.key_pressed(VirtualKeyCode::Space) {
                paused = !paused;
            }

            [0, 1].into_iter().for_each(|mouse_btn| {
                if input.mouse_held(mouse_btn) || input.mouse_released(mouse_btn) {
                    input.mouse().into_iter().for_each(|(x, y)| {
                        let (x, y) = pixels.window_pos_to_pixel((x, y)).unwrap();
                        [
                            y * WIDTH + x,
                            (y - 1) * WIDTH + x,
                            (y + 1) * WIDTH + x,
                            y * WIDTH + x + 1,
                            y * WIDTH + x - 1,
                        ]
                        .into_iter()
                        .for_each(|index| {
                            match mouse_btn {
                                0 => world.set_virus_cluster(index),
                                1 => world.set_defense_cluster(index),
                                _ => unreachable!(),
                            };
                        });
                    });
                }
            });

            if let Some(size) = input.window_resized() {
                pixels.resize_surface(size.width, size.height);
            }
            if !paused {
                window.request_redraw();
            }
        }
    });
}

struct Nbrs {
    top: Option<Cell>,
    right: Option<Cell>,
    bottom: Option<Cell>,
    left: Option<Cell>,
}

impl Nbrs {
    fn get_contagion_total(&self) -> i16 {
        let mut total = 0;
        for item in [self.top, self.right, self.bottom, self.left] {
            item.map(|v| {
                if v.contagious {
                    total += thread_rng().gen_range(20..100);
                }
            });
        }
        total
    }
    fn get_defense_total(&self) -> i16 {
        let mut total = 0;
        for item in [self.top, self.right, self.bottom, self.left] {
            item.map(|v| {
                if v.defend {
                    total += thread_rng().gen_range(150..250);
                }
            });
        }
        total
    }
}

#[derive(Clone, Copy)]
struct Cell {
    alive: bool,
    infected: bool,
    contagious: bool,
    days_infected: usize,
    immune: bool,
    defend: bool,
}

impl Cell {
    fn new() -> Self {
        Cell {
            alive: true,
            contagious: false,
            infected: false,
            days_infected: 0,
            immune: thread_rng().gen_range(0..100) < 20,
            defend: thread_rng().gen_range(0..100) < 1,
        }
    }

    fn infect(&mut self) {
        if self.alive && !self.immune && !self.infected {
            self.contagious = true;
            self.infected = true;
            self.days_infected = 0;
        }
    }

    fn defend(&mut self) {
        if self.alive {
            self.defend = true;
            self.infected = false;
            self.contagious = false;
        }
    }

    fn update_infection(&mut self) {
        if self.infected {
            self.days_infected += 1;
        }
        if self.days_infected == thread_rng().gen_range(4..13) {
            let numb = thread_rng().gen_range(0..100);
            if numb > 40 {
                self.alive = false;
            } else {
                self.immune = true;
                self.infected = false;
                self.contagious = false;
            }
        }
    }
}

struct World {
    width: usize,
    height: usize,
    cells: Vec<Cell>,
}

const GREEN: [u8; 4] = [60, 179, 113, 255];
const BLACK: [u8; 4] = [0, 0, 0, 255];
const RED: [u8; 4] = [255, 0, 0, 255];
const BLUE: [u8; 4] = [0, 0, 255, 255];

impl World {
    fn new(width: usize, height: usize) -> Self {
        let size = width * height;
        let mut world = World {
            cells: vec![],
            height,
            width,
        };

        (0..size).for_each(|_| {
            world.cells.push(Cell::new());
        });

        world
    }

    fn set_virus_cluster(&mut self, index: usize) {
        self.cells.get_mut(index).map(|cell| cell.infect());
    }

    fn set_defense_cluster(&mut self, index: usize) {
        self.cells.get_mut(index).map(|cell| cell.defend());
    }

    fn get_nbrs(&self, index: usize) -> Nbrs {
        let mut nbrs = Nbrs {
            top: None,
            right: None,
            bottom: None,
            left: None,
        };

        if index > self.width {
            nbrs.top = Some(self.cells[index.saturating_sub(self.width)])
        }
        if (index + 1) % self.width != 0 && index < self.width * self.height {
            nbrs.right = Some(self.cells[index + 1])
        }
        if index < self.width * (self.height - 1) {
            nbrs.bottom = Some(self.cells[index + self.width])
        }
        if index % self.width != 0 {
            nbrs.left = Some(self.cells[index.saturating_sub(1)])
        }
        nbrs
    }

    fn set_contagion(&self, index: usize, cell: &mut Cell) {
        let nbrs = self.get_nbrs(index);

        let contagion_total = nbrs.get_contagion_total();
        let rng: i16 = thread_rng().gen_range(67..220);
        if contagion_total > rng {
            cell.infect()
        };
    }

    fn set_defense(&self, index: usize, cell: &mut Cell) {
        let nbrs = self.get_nbrs(index);
        let rng: i16 = thread_rng().gen_range(150..500);

        match () {
            _ if nbrs.get_defense_total() > rng && nbrs.get_contagion_total() != 0 => cell.defend(),
            _ => cell.update_infection(),
        }
    }

    fn update(&mut self, frame: &mut [u8]) {
        let cells = &mut self.cells.clone();
        cells
            .par_iter_mut()
            .zip(frame.par_chunks_exact_mut(4).enumerate())
            .for_each(|(cell, (index, pixel))| {
                match () {
                    _ if cell.infected => self.set_defense(index, cell),
                    _ if cell.alive && !cell.immune => self.set_contagion(index, cell),
                    _ => (),
                };

                let rgb = match () {
                    _ if cell.immune => GREEN,
                    _ if !cell.alive => BLACK,
                    _ if cell.infected => RED,
                    _ if cell.defend => BLUE,
                    _ => GREEN,
                };

                pixel.copy_from_slice(&rgb);
            });
        self.cells = std::mem::take(cells);
    }
}
