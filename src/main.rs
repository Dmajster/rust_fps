mod camera;
mod code;
mod gltf;
mod input;
mod renderer;
mod texture;

use code::components::{position::Position, rotation::Rotation};

use camera::Camera;
use futures::executor::block_on;
use renderer::State;
use std::time::{Duration, Instant};
use ultraviolet::{Rotor3, Similarity3, Vec2, Vec3};

use input::Input;
use legion::component;
use legion::{system, Resources, Schedule, World};

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

#[system()]
fn update_mouse(#[resource] input: &mut Input) {
    input.mouse.acceleration.x = input.mouse.position.x - input.mouse.old_position.x;
    input.mouse.acceleration.y = input.mouse.position.y - input.mouse.old_position.y;
    input.mouse.old_position = input.mouse.position;

    // println!(
    //     "mouse acceleration x: {}, y: {}",
    //     input.mouse.acceleration.x, input.mouse.acceleration.y
    // );
}

#[system(for_each)]
#[filter(component::<Camera>())]
fn move_camera(
    #[resource] game_clock: &GameClock,
    #[resource] input: &Input,
    position: &mut Position,
    rotation: &mut Rotation,
) {
    let delta_time = game_clock.last_frame_duration.as_secs_f32();
    let movement_speed_in_meters_per_second = 5.0;

    let horizontal_rotor = Rotor3::from_rotation_xz(input.mouse.acceleration.x * delta_time);
    *rotation = *rotation * horizontal_rotor;

    let right_vector = Vec3::new(1.0, 0.0, 0.0).rotated_by(*rotation);
    let forward_vector = Vec3::new(0.0, 0.0, 1.0).rotated_by(*rotation);

    let mut movement = Vec2::default();

    if input.key_held(VirtualKeyCode::W) {
        movement.y -= 1.0;
    }
    if input.key_held(VirtualKeyCode::S) {
        movement.y += 1.0;
    }
    if input.key_held(VirtualKeyCode::A) {
        movement.x -= 1.0;
    }
    if input.key_held(VirtualKeyCode::D) {
        movement.x += 1.0;
    }

    *position += (movement.x * right_vector + movement.y * forward_vector)
        * movement_speed_in_meters_per_second
        * delta_time;

    // let forward = camera.target - camera.eye;
    // let forward_norm = forward.normalized();
    // let forward_mag = forward.mag();
    // let speed = 0.2;

    // if input.key_held(VirtualKeyCode::W) && forward_mag > speed {
    //     camera.eye += forward_norm * speed;
    // }
    // if input.key_held(VirtualKeyCode::S) {
    //     camera.eye -= forward_norm * speed;
    // }

    // let right = forward_norm.cross(camera.up);
    // let forward = camera.target - camera.eye;
    // let forward_mag = forward.mag();

    // if input.key_held(VirtualKeyCode::D) {
    //     camera.eye = camera.target - (forward + right * speed).normalized() * forward_mag;
    // }
    // if input.key_held(VirtualKeyCode::A) {
    //     camera.eye = camera.target - (forward - right * speed).normalized() * forward_mag;
    // }
}

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

    let mut world = World::default();
    let mut resources = Resources::default();
    resources.insert(block_on(renderer::State::new(&window)));
    resources.insert(GameClock::new(60));
    resources.insert(Input::default());

    world.push((
        Position::new(0.0, 0.0, 10.0),
        Rotation::from_euler_angles(0.0, 0.0, 0.0).normalized(),
        Camera::new(16.0 / 9.0, 45.0f32.to_radians(), 0.1, 100.0),
    ));

    let mut update_schedule = Schedule::builder()
        // .add_system(update_print_system())
        .add_system(update_mouse_system())
        .add_system(move_camera_system())
        .add_system(render_system())
        .build();

    let mut fixed_update_schedule = Schedule::builder()
        // .add_system(fixed_update_print_system())
        .build();

    let mut fixed_update_time_accumulator = 0.0;

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

                    if input_manager.key_held(VirtualKeyCode::Escape) {
                        *control_flow = ControlFlow::Exit
                    }
                }
                WindowEvent::CursorMoved { position, .. } => {
                    let mut input_manager = resources
                        .get_mut::<Input>()
                        .expect("failed getting input resource?");

                    input_manager.mouse.position.x = position.x as f32;
                    input_manager.mouse.position.y = position.y as f32;
                }
                // TODO implement with ECS
                // WindowEvent::Resized(physical_size) => {
                //     render_state.resize(*physical_size);
                // }
                // WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                //     render_state.resize(**new_inner_size);
                // }
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
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => {}
        }
    });
}

#[system(for_each)]
fn render(
    #[resource] renderer: &mut State,
    position: &Position,
    rotation: &mut Rotation,
    camera: &mut Camera,
) {
    let camera_matrix = Similarity3::new(position.clone(), rotation.clone(), 1.0)
        .into_homogeneous_matrix()
        .inversed();
    let projection_matrix = camera.projection_matrix;

    renderer.render(projection_matrix * camera_matrix)
}
