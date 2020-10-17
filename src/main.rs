mod input;
mod renderer;
mod texture;

use futures::executor::block_on;
use std::time::{Duration, Instant};

use input::Input;
use legion::{system, Resources, Schedule, World};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

#[system]
fn update_print(#[resource] game_clock: &GameClock) {
    println!(
        "update dt: {} ",
        game_clock.last_frame_duration.as_secs_f64()
    );
}

#[system]
fn fixed_update_print() {
    println!("fixed update");
}

#[derive(Debug)]
struct GameClock {
    game_start_instant: Instant,
    current_frame_instant: Instant,
    new_frame_instant: Instant,

    last_frame_duration: Duration,

    fixed_update_step_duration: f64,
}

impl GameClock {
    pub fn new(fixed_update_steps_per_second: usize) -> Self {
        Self {
            game_start_instant: Instant::now(),
            current_frame_instant: Instant::now(),
            new_frame_instant: Instant::now(),

            last_frame_duration: Duration::default(),

            fixed_update_step_duration: 1.0 / fixed_update_steps_per_second as f64,
        }
    }
}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    // Since main can't be async, we're going to need to block
    let mut render_state = block_on(renderer::State::new(&window));

    let mut world = World::default();
    let mut resources = Resources::default();

    let mut update_schedule = Schedule::builder()
        .add_system(update_print_system())
        .build();

    let mut fixed_update_schedule = Schedule::builder()
        .add_system(fixed_update_print_system())
        .build();

    let mut fixed_update_time_accumulator = 0.0;

    resources.insert(GameClock::new(60));
    resources.insert(Input::default());

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput { input, .. } => {
                    let mut input_manager = resources
                        .get_mut::<Input>()
                        .expect("failed getting input resource?");

                    input_manager.process_keyboard(input);

                    render_state.input(event);

                    if input_manager.key_held(VirtualKeyCode::Escape) {
                        *control_flow = ControlFlow::Exit
                    }
                }
                WindowEvent::Resized(physical_size) => {
                    render_state.resize(*physical_size);
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    render_state.resize(**new_inner_size);
                }
                _ => {}
            },
            Event::RedrawRequested(_) => {
                let mut game_clock = resources
                    .get_mut::<GameClock>()
                    .expect("failed getting game clock resource?");

                game_clock.new_frame_instant = Instant::now();
                game_clock.last_frame_duration = game_clock.current_frame_instant.elapsed();
                game_clock.current_frame_instant = game_clock.new_frame_instant;

                // Clone step duration to local variable as we need it in the while loop below
                let fixed_update_step_duration = game_clock.fixed_update_step_duration;

                // Add the duration of last frame to the time that needs to be simulated by fixed update
                fixed_update_time_accumulator += game_clock.last_frame_duration.as_secs_f64();

                // Drop game_clock reference so we can get a mutable reference to GameClock in the while loop
                drop(game_clock);

                // Do fixed updates while we have more than one fixed step of time available
                while fixed_update_time_accumulator >= fixed_update_step_duration {
                    fixed_update_time_accumulator -= fixed_update_step_duration;

                    fixed_update_schedule.execute(&mut world, &mut resources);
                }
                update_schedule.execute(&mut world, &mut resources);

                // TODO gaffer on games physics state lerping???

                // TODO convert this to ECS?
                render_state.update();
                render_state.render();
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => {}
        }
    });
}
