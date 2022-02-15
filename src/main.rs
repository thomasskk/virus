use pixels::{Error, Pixels, SurfaceTexture};
use rand::{thread_rng, Rng};
use rayon::prelude::*;
use winit::{
    dpi::LogicalSize,
    event::{Event, MouseButton, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
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
    world.set_rand_cluster();

    event_loop.run(move |event, _, control_flow| {
        if let Event::RedrawRequested(_) = event {
            world.update(pixels.get_frame());
            pixels.render().unwrap();
        }

        if input.update(&event) {
            if input.key_pressed(VirtualKeyCode::Space) {
                paused = !paused;
            }

            if input.mouse_held(0) || input.mouse_released(0) {
                input.mouse().map(|(x, y)| {
                    let mouse = pixels.window_pos_to_pixel((x, y)).unwrap();
                    let (x, y) = mouse;
                    let index = y * WIDTH + x;

                    world.set_cluster(index);
                });
            }
            

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
                    total += v.contagion_rate;
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
    contagion_rate: i16,
    days_infected: usize,
    immune: bool,

}

impl Cell {
    fn new() -> Self {
        Cell {
            alive: true,
            contagious: false,
            infected: false,
            contagion_rate: thread_rng().gen_range(20..100),
            days_infected: 0,
            immune: false,
        }
    }

    fn infect(&mut self) {
        if self.alive && !self.immune && !self.infected {
            self.contagious = true;
            self.infected = true;
            self.days_infected = 0;
        }
    }

    fn update(&mut self) {
        if self.infected {
            self.days_infected += 1;
        }
        if self.days_infected == 15 {
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
const ORANGE: [u8; 4] = [255, 165, 0, 255];

impl World {
    fn new(width: usize, height: usize) -> Self {
        let size = width * height;
        let mut world = World {
            cells: vec![],
            height,
            width,
        };

        (0..size).for_each(|i| {
            world.cells.push(Cell::new());
        });

        world
    }

    fn set_cluster(&mut self, index: usize) {
        self.cells[index].infect();
    }

    fn set_rand_cluster(&mut self) {
        let index = self.cells.len() / 2 - (WIDTH / 2);
        self.cells[index].infect();
    }

    fn set_contagion(&self, index: usize, cell: &mut Cell) {
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

        let contagion_total = nbrs.get_contagion_total();
        let rng: i16 = thread_rng().gen_range(1..200);
        if rng < (contagion_total) {
            cell.infect();
        }
    }

    fn update(&mut self, frame: &mut [u8]) {
        let cells = &mut self.cells.clone();
        cells
            .par_iter_mut()
            .zip(frame.par_chunks_exact_mut(4).enumerate())
            .for_each(|(cell, (index, pixel))| {
                match () {
                    _ if cell.infected => cell.update(),
                    _ if cell.alive && !cell.immune => self.set_contagion(index, cell),
                    _ => (),
                };

                let rgb = match () {
                    _ if cell.immune => ORANGE,
                    _ if !cell.alive => BLACK,
                    _ if cell.infected => RED,
                    _ => GREEN,
                };

                pixel.copy_from_slice(&rgb);
            });
        self.cells = std::mem::take(cells);
    }
}
