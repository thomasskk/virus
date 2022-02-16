use pixels::{Error, Pixels, SurfaceTexture};
use virus::world::World;
use winit::{
    dpi::LogicalSize,
    event::{Event, VirtualKeyCode},
    event_loop::EventLoop,
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;

const WIDTH: usize = 650;
const HEIGHT: usize = 650;

fn main() -> Result<(), Error> {
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let mut paused = false;

    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        let inner_size = LogicalSize::new((WIDTH * 2) as f64, (HEIGHT * 2) as f64);
        WindowBuilder::new()
            .with_title("Virus")
            .with_inner_size(size)
            .with_min_inner_size(inner_size)
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
