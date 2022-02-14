#![forbid(unsafe_code)]

use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use rand::{thread_rng, Rng, prelude::ThreadRng};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use std::{thread, time};
use rand::distributions::{Distribution, Uniform};

const WIDTH: u32 = 1000;
const HEIGHT: u32 = 1000;

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
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };

    let mut world = World::new(WIDTH as usize, HEIGHT as usize);
    let rand: usize = world.cells.len() / 2 - (WIDTH / 2) as usize;
    world.cells.get_mut(rand ).unwrap().infect();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit
            }
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
        }
    });
}

#[derive(Clone, Default)]
struct Cell {
    alive: bool,
    infected: bool,
    contagious: bool,
    contagion_rate: usize,
    days_infected: usize,
    rng: ThreadRng
}

impl Cell {
    fn default() -> Self {
        let contagion_rate: usize = thread_rng(). gen_range(20..100);
        Cell {
            alive: true,
            contagious: false,
            infected: false,
            contagion_rate,
            days_infected: 0,
            rng: thread_rng()
        }
    }

    fn infect(&mut self) {
        self.contagious = true;
        self.infected = true;
        self.days_infected = 0;
    }

    fn update(&mut self, rng: &ThreadRng)  {
        if self.infected {
            self.days_infected += 1;
        }
        if self.days_infected == 15 {
            let numb = self.rng.gen_range(0..100);
            if numb > 50 {
                self.alive = false;
            }
        }
    }
}
struct World {
    width: usize,
    height: usize,
    cells: Vec<Cell>,
    rng: ThreadRng,
}

struct Nbrs {
    top: usize,
    right: usize,
    bottom: usize,
    left: usize,
}

impl World {
    fn new(width: usize, height: usize) -> Self {
        let size = width * height ;
        World {
            cells: vec![Cell::default(); size],
            height,
            width,
            rng: thread_rng(),
        }
    }

    fn get_nbrs(&mut self, index: usize) -> Nbrs {
        let get_rate = |i: usize, condition| {
            if condition {
                let cell = self.cells.get(i).unwrap();
                if cell.contagious {
                    cell.contagion_rate
                } else {
                    0 
                }
            } else {
                0
            }
        };

        Nbrs {
            top: get_rate(index.saturating_sub(self.width), index > self.width -1 ),
            right: get_rate(index.saturating_add(1), ((index+1) % self.width != 0) && index+1 < (self.width * self.height -1)),
            bottom: get_rate(index.saturating_add(self.width), index +1 < (self.width) * (self.height - 1)),
            left: get_rate(index.saturating_sub(1), index != 0 && (index  % self.width) != 0),
        }
    }

    fn get_updated_cell_rgb(&mut self, index: usize) -> &[u8] {
        let nbrs = self.get_nbrs(index);
        let cell = self.cells.get_mut(index).unwrap();
        cell.update(&self.rng);
        if cell.alive && !cell.infected {
            let rng: u16 = self.rng.gen_range(1..150);
            if rng < (nbrs.top + nbrs.right + nbrs.bottom + nbrs.left) as u16 {
                cell.infect();
            }
        }
        match () {
            _ if !cell.alive => &[0, 0, 0, 0xff],
            _ if cell.infected => &[255, 0, 0, 0xff],
            _ => &[0, 255, 0, 0xff],
        }
    }

    fn update(&mut self, frame: &mut [u8]) {
        for (index, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let rgb = self.get_updated_cell_rgb(index);
            pixel.copy_from_slice(rgb);
        }
    }
}
