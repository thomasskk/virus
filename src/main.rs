use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use rand::{prelude::ThreadRng, thread_rng, Rng};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

const WIDTH: usize = 500;
const HEIGHT: usize = 500;

fn main() -> Result<(), Error> {
    let event_loop = EventLoop::new();

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

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => *control_flow = ControlFlow::Exit,
        Event::MainEventsCleared => {
            window.request_redraw();
        }
        Event::RedrawRequested(_) => {
            world.update(pixels.get_frame());
            match pixels.render() {
                Ok(_) => (),
                Err(e) => {
                    error!("pixels.render() failed: {}", e);
                    *control_flow = ControlFlow::Exit;
                    return;
                }
            }
        }
        _ => (),
    });
}

#[derive(Clone, Copy, Default)]
struct Cell {
    alive: bool,
    infected: bool,
    contagious: bool,
    contagion_rate: i16,
    days_infected: usize,
    immune: bool,
}

impl Cell {
    fn default() -> Self {
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
        self.contagious = true;
        self.infected = true;
        self.days_infected = 0;
    }

    fn update(&mut self, rng: &mut ThreadRng) {
        if self.infected {
            self.days_infected += 1;
        }
        if self.days_infected == 15 {
            let numb = rng.gen_range(0..100);
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

struct World {
    width: usize,
    height: usize,
    cells: Vec<Cell>,
    rng: ThreadRng,
}

impl World {
    fn new(width: usize, height: usize) -> Self {
        let size = width * height;
        World {
            cells: vec![Cell::default(); size],
            height,
            width,
            rng: thread_rng(),
        }
    }

    fn set_rand_cluster(&mut self) {
        let rand = self.cells.len() / 2 - (WIDTH / 2);
        self.cells[rand].infect();
    }

    fn get_nbrs(&self, index: usize) -> Nbrs {
        let mut nbrs = Nbrs {
            top: None,
            right: None,
            bottom: None,
            left: None,
        };

        if index > self.width - 1 {
            nbrs.top = Some(self.cells[index.saturating_sub(self.width)])
        }
        if (index + 1) % self.width != 0 && index + 1 < (self.width * self.height - 1) {
            nbrs.right = Some(self.cells[index + 1])
        }
        if index + 1 < self.width * (self.height - 1) {
            nbrs.bottom = Some(self.cells[index + self.width])
        }
        if index != 0 && (index % self.width) != 0 {
            nbrs.left = Some(self.cells[index.saturating_sub(1)])
        }

        nbrs
    }

    fn get_cell_rgb(&mut self, cell: &mut Cell, index: usize) -> &[u8] {
        if cell.infected {
            cell.update(&mut self.rng);
        } else {
            let nbrs = self.get_nbrs(index);
            let contagion_total = nbrs.get_contagion_total();
            let rng: i16 = self.rng.gen_range(1..200);
            if rng < (contagion_total) {
                cell.infect();
            }
        }

        match () {
            _ if cell.infected => &[255, 0, 0, 0xff],
            _ if !cell.alive => &[0, 0, 0, 0xff],
            _ if cell.immune => &[255, 165, 0, 0xff],
            _ => &[60, 179, 113, 0xff],
        }
    }

    fn update(&mut self, frame: &mut [u8]) {
        for (index, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let mut cell = self.cells[index];
            {
                let rgb = match () {
                    _ if cell.immune => &[255, 165, 0, 0xff],
                    _ if !cell.alive => &[0, 0, 0, 0xff],
                    _ => self.get_cell_rgb(&mut cell, index),
                };
                pixel.copy_from_slice(rgb);
            }
            self.cells[index] = cell;
        }
    }
}
