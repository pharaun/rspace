#[macro_use]
extern crate rand;
extern crate cgmath;

// Asteroidish game (based off the ggez example)
use std::time::{Duration, Instant};
use std::thread;

use cgmath::prelude::*;
use cgmath::Vector2;
use cgmath::Rad;

// Vulkano uses
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::buffer::cpu_pool::CpuBufferPool;
use vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState};
use vulkano::command_buffer::pool::standard::StandardCommandPoolBuilder;
use vulkano::device::{Device, DeviceExtensions};
use vulkano::framebuffer::{Framebuffer, FramebufferAbstract, Subpass, RenderPassAbstract};
use vulkano::descriptor::descriptor_set::PersistentDescriptorSet;
use vulkano::image::SwapchainImage;
use vulkano::instance::{Instance, PhysicalDevice};
use vulkano::pipeline::GraphicsPipelineAbstract;
use vulkano::pipeline::GraphicsPipeline;
use vulkano::pipeline::viewport::Viewport;
use vulkano::pipeline::vertex::VertexSource;
use vulkano::swapchain::{AcquireError, PresentMode, SurfaceTransform, Swapchain, SwapchainCreationError};
use vulkano::swapchain;
use vulkano::sync::{GpuFuture, FlushError};
use vulkano::sync;

use vulkano_win::VkSurfaceBuild;

// win + kb
use winit::event_loop::{EventLoop, ControlFlow};
use winit::window::{Window, WindowBuilder};
use winit::event::{Event, WindowEvent, KeyboardInput, ElementState, VirtualKeyCode};


use std::sync::Arc;


/// *********************************************************************
/// Basic stuff.
/// First, we create a vector type.
/// You're probably better off using a real vector math lib but I
/// didn't want to add more dependencies and such.
/// **********************************************************************
trait Vec2Ext {
    fn from_angle(angle: Rad<f64>) -> Self;
    fn random(max_magnitude: f64) -> Self;
    fn scaled(&self, rhs: f64) -> Self;
    fn clamped(&self, max: f64) -> Self;
}

impl Vec2Ext for Vector2<f64> {
    /// Create a unit vector representing the
    /// given angle (in radians)
    fn from_angle(angle: Rad<f64>) -> Self {
        let (vx, vy) = angle.sin_cos();
        Vector2::new(vx, vy)
    }

    fn random(max_magnitude: f64) -> Self {
        let angle = Rad(rand::random::<f64>() * 2.0 * std::f64::consts::PI);
        let mag = rand::random::<f64>() * max_magnitude;
        Vector2::from_angle(angle).scaled(mag)
    }

    fn scaled(&self, rhs: f64) -> Self {
        let vx = self.x * rhs;
        let vy = self.y * rhs;
        Vector2::new(vx, vy)
    }

    /// Returns a vector whose magnitude is between
    /// 0 and max.
    fn clamped(&self, max: f64) -> Self {
        let mag = self.magnitude();
        if mag > max {
            self.normalize().scaled(max)
        } else {
            *self
        }
    }
}



/// *********************************************************************
/// Now we define our Actor's.
/// An Actor is anything in the game world.
/// We're not *quite* making a real entity-component system but it's
/// pretty close.  For a more complicated game you would want a
/// real ECS, but for this it's enough to say that all our game objects
/// contain pretty much the same data.
/// **********************************************************************
#[derive(Debug)]
enum ActorType {
    Player,
    Rock,
    Shot,
}

#[derive(Debug)]
struct Actor {
    tag: ActorType,
    pos: Vector2<f64>,
    facing: Rad<f64>,
    velocity: Vector2<f64>,
    rvel: f64,
    bbox_size: f64,

    // I am going to lazily overload "life" with a
    // double meaning rather than making a proper ECS;
    // for shots, it is the time left to live,
    // for players and such, it is the actual hit points.
    life: f64,
}

const PLAYER_LIFE: f64 = 1.0;
const SHOT_LIFE: f64 = 2.0;
const ROCK_LIFE: f64 = 1.0;

const PLAYER_BBOX: f64 = 12.0;
const ROCK_BBOX: f64 = 12.0;
const SHOT_BBOX: f64 = 6.0;


/// *********************************************************************
/// Now we have some initializer functions for different game objects.
/// **********************************************************************

fn create_player() -> Actor {
    Actor {
        tag: ActorType::Player,
        pos: Vector2::new(0., 0.),
        facing: Rad(0.),
        velocity: Vector2::new(0., 0.),
        rvel: 0.,
        bbox_size: PLAYER_BBOX,
        life: PLAYER_LIFE,
    }
}

fn create_rock() -> Actor {
    Actor {
        tag: ActorType::Rock,
        pos: Vector2::new(0., 0.),
        facing: Rad(0.),
        velocity: Vector2::new(0., 0.),
        rvel: 0.,
        bbox_size: ROCK_BBOX,
        life: ROCK_LIFE,
    }
}

fn create_shot() -> Actor {
    Actor {
        tag: ActorType::Shot,
        pos: Vector2::new(0., 0.),
        facing: Rad(0.),
        velocity: Vector2::new(0., 0.),
        rvel: SHOT_RVEL,
        bbox_size: SHOT_BBOX,
        life: SHOT_LIFE,
    }
}

const MAX_ROCK_VEL: f64 = 50.0;

/// Create the given number of rocks.
/// Makes sure that none of them are within the
/// given exclusion zone (nominally the player)
/// Note that this *could* create rocks outside the
/// bounds of the playing field, so it should be
/// called before `wrap_actor_position()` happens.
fn create_rocks(num: i32, exclusion: &Vector2<f64>, min_radius: f64, max_radius: f64) -> Vec<Actor> {
    assert!(max_radius > min_radius);
    let new_rock = |_| {
        let mut rock = create_rock();
        let r_angle = Rad(rand::random::<f64>() * 2.0 * std::f64::consts::PI);
        let r_distance = rand::random::<f64>() * (max_radius - min_radius) + min_radius;
        rock.pos = Vector2::from_angle(r_angle).scaled(r_distance) + *exclusion;
        rock.velocity = Vector2::random(MAX_ROCK_VEL);
        rock
    };
    (0..num).map(new_rock).collect()
}



/// *********************************************************************
/// Now we have functions to handle physics.  We do simple Newtonian
/// physics (so we do have inertia), and cap the max speed so that we
/// don't have to worry too much about small objects clipping through
/// each other.
///
/// Our unit of world space is simply pixels, though we do transform
/// the coordinate system so that +y is up and -y is down.
/// **********************************************************************

const SHOT_SPEED: f64 = 200.0;
const SHOT_RVEL: f64 = 0.1;
const SPRITE_SIZE: u32 = 32;

// Acceleration in pixels per second, more or less.
const PLAYER_THRUST: f64 = 100.0;
// Rotation in radians per second.
const PLAYER_TURN_RATE: f64 = 3.05;
// Seconds between shots
const PLAYER_SHOT_TIME: f64 = 0.5;


fn player_handle_input(actor: &mut Actor, input: &InputState, dt: f64) {
    actor.facing += Rad(dt * PLAYER_TURN_RATE * input.xaxis);

    if input.yaxis > 0.0 {
        player_thrust(actor, dt);
    }
}

fn player_thrust(actor: &mut Actor, dt: f64) {
    let direction_vector = Vector2::from_angle(actor.facing);
    let thrust_vector = direction_vector.scaled(PLAYER_THRUST);
    actor.velocity += thrust_vector.scaled(dt);
}

const MAX_PHYSICS_VEL: f64 = 250.0;

fn update_actor_position(actor: &mut Actor, dt: f64) {
    actor.velocity = actor.velocity.clamped(MAX_PHYSICS_VEL);
    let dv = actor.velocity.scaled(dt);
    actor.pos += dv;
    actor.facing += Rad(actor.rvel);
}

/// Takes an actor and wraps its position to the bounds of the
/// screen, so if it goes off the left side of the screen it
/// will re-enter on the right side and so on.
fn wrap_actor_position(actor: &mut Actor, sx: f64, sy: f64) {
    // Wrap screen
    let screen_x_bounds = sx / 2.0;
    let screen_y_bounds = sy / 2.0;
    let sprite_half_size = (SPRITE_SIZE / 2) as f64;
    let actor_center = actor.pos - Vector2::new(-sprite_half_size, sprite_half_size);
    if actor_center.x > screen_x_bounds {
        actor.pos.x -= sx;
    } else if actor_center.x < -screen_x_bounds {
        actor.pos.x += sx;
    };
    if actor_center.y > screen_y_bounds {
        actor.pos.y -= sy;
    } else if actor_center.y < -screen_y_bounds {
        actor.pos.y += sy;
    }
}

fn handle_timed_life(actor: &mut Actor, dt: f64) {
    actor.life -= dt;
}


/// Translates the world coordinate system, which
/// has Y pointing up and the origin at the center,
/// to the screen coordinate system, which has Y
/// pointing downward and the origin at the top-left,
fn world_to_screen_coords(screen_width: u32, screen_height: u32, point: &Vector2<f64>) -> Vector2<f64> {
    let width = screen_width as f64;
    let height = screen_height as f64;
    let x = point.x + width / 2.0;
    let y = height - (point.y + height / 2.0);
    Vector2::new(x, y)
}






/// **********************************************************************
/// The `InputState` is exactly what it sounds like, it just keeps track of
/// the user's input state so that we turn keyboard events into something
/// state-based and device-independent.
/// **********************************************************************
#[derive(Debug)]
struct InputState {
    xaxis: f64,
    yaxis: f64,
    fire: bool,
}

impl Default for InputState {
    fn default() -> Self {
        InputState {
            xaxis: 0.0,
            yaxis: 0.0,
            fire: false,
        }
    }
}





/// **********************************************************************
/// Now we're getting into the actual game loop.  The `MainState` is our
/// game's "global" state, it keeps track of everything we need for
/// actually running the game.
///
/// Our game objects are simply a vector for each actor type, and we
/// probably mingle gameplay-state (like score) and hardware-state
/// (like gui_`dirty`) a little more than we should, but for something
/// this small it hardly matters.
/// **********************************************************************

struct MainState {
    player: Actor,
    shots: Vec<Actor>,
    rocks: Vec<Actor>,
    input: InputState,
    player_shot_timeout: f64,
}

impl MainState {
    fn new() -> MainState {
        let player = create_player();
        let rocks = create_rocks(5, &player.pos, 100.0, 250.0);

        MainState {
            player: player,
            shots: Vec::new(),
            rocks: rocks,
            input: InputState::default(),
            player_shot_timeout: 0.0,
        }
    }

    fn fire_player_shot(&mut self) {
        self.player_shot_timeout = PLAYER_SHOT_TIME;

        let player = &self.player;
        let mut shot = create_shot();
        shot.pos = player.pos;
        shot.facing = player.facing;
        let direction = Vector2::from_angle(shot.facing);
        shot.velocity.x = SHOT_SPEED * direction.x;
        shot.velocity.y = SHOT_SPEED * direction.y;

        self.shots.push(shot);
    }

    fn clear_dead_stuff(&mut self) {
        self.shots.retain(|s| s.life > 0.0);
        self.rocks.retain(|r| r.life > 0.0);
    }

    fn handle_collisions(&mut self) {
        for rock in &mut self.rocks {
            let pdistance = rock.pos - self.player.pos;
            if pdistance.magnitude() < (self.player.bbox_size + rock.bbox_size) {
                self.player.life = 0.0;
            }
            for shot in &mut self.shots {
                let distance = shot.pos - rock.pos;
                if distance.magnitude() < (shot.bbox_size + rock.bbox_size) {
                    shot.life = 0.0;
                    rock.life = 0.0;
                }
            }
        }
    }

    fn check_for_level_respawn(&mut self) {
        if self.rocks.is_empty() {
            let r = create_rocks(5, &self.player.pos, 100.0, 250.0);
            self.rocks.extend(r);
        }
    }
}





//********************************************************************************
// Start main loop + vulkan setup stuff
//
// This code is from the vulkano triagle example
//********************************************************************************
fn main() {
    let instance = Instance::new(None, &vulkano_win::required_extensions(), None).unwrap();

    // Be lazy and pick first device
    let physical = PhysicalDevice::enumerate(&instance).next().unwrap();
    println!("Using device: {} (type: {:?})", physical.name(), physical.ty());

    // Get a window + window surface
    let events_loop = EventLoop::new();
    let surface = WindowBuilder::new().build_vk_surface(&events_loop, instance.clone()).unwrap();
    let window = surface.window();

    // Init the device + first queue
    let (device, queue) = {
        // GPU queue, for now just one, good enough for us
        let queue_family = physical.queue_families().find(|&q| {
            // We take the first queue that supports drawing to our window.
            q.supports_graphics() && surface.is_supported(q).unwrap_or(false)
        }).unwrap();

        let (device, mut queues) = Device::new(
            physical,
            physical.supported_features(),
            &DeviceExtensions { khr_swapchain: true, .. DeviceExtensions::none() },
            [(queue_family, 0.5)].iter().cloned()
        ).unwrap();

        // Since we can get more than 1 queue.... but here we only request the first one so
        // grab that and toss the rest away
        (device, queues.next().unwrap())
    };

    // Swapchain setup + Images
    let (mut swapchain, images) = {
        let caps = surface.capabilities(physical).unwrap();

        Swapchain::new(
            device.clone(),
            surface.clone(),
            caps.min_image_count,
            caps.supported_formats[0].0,
            get_dimensions(&window),
            1,
            caps.supported_usage_flags,
            &queue,
            SurfaceTransform::Identity,
            caps.supported_composite_alpha.iter().next().unwrap(),
            PresentMode::Fifo,
            true,
            None
        ).unwrap()
    };

    // We now create a buffer that will store the shape of our triangle.
    let vertex_buffer = {
        #[derive(Default, Debug, Clone)]
        struct Vertex { position: [f32; 2] }
        vulkano::impl_vertex!(Vertex, position);

        CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage::all(), [
            Vertex { position: [-0.3, -0.3] },
            Vertex { position: [ 0.0,  0.5] },
            Vertex { position: [ 0.3, -0.3] }
        ].iter().cloned()).unwrap()
    };

    // Uniform buffer for model view
    let uniform_buffer = CpuBufferPool::<vs::ty::Data>::new(device.clone(), BufferUsage::all());

    // Shader load
    let vs = vs::Shader::load(device.clone()).unwrap();
    let fs = fs::Shader::load(device.clone()).unwrap();

    // Render pass setup
    let render_pass = Arc::new(vulkano::single_pass_renderpass!(
        device.clone(),
        attachments: {
            // We use the attachment named `color` as the one and only color attachment.
            color: { load: Clear, store: Store, format: swapchain.format(), samples: 1, }
        },
        pass: {
            // We use the attachment named `color` as the one and only color attachment.
            color: [color],
            depth_stencil: {}
        }
    ).unwrap());

    // Pipeline (similiar to opengl program)
    let pipeline = Arc::new(GraphicsPipeline::start()
        .vertex_input_single_buffer()
        .vertex_shader(vs.main_entry_point(), ())
        .triangle_list()
        .viewports_dynamic_scissors_irrelevant(1)
        .fragment_shader(fs.main_entry_point(), ())
        .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
        .build(device.clone())
        .unwrap());

    // Dynamic viewpoint, let us be lazy and not recreate the whole pipeline everytime the window is resized
    let mut dynamic_state = DynamicState { line_width: None, viewports: None, scissors: None };

    // Framebuffers
    let mut framebuffers = window_size_dependent_setup(&images, render_pass.clone(), &mut dynamic_state);

    // Swapchain invalidation (ie resize window, we need to recreate)
    let mut recreate_swapchain = false;

    // When we submit a command, we get a gpu future, and it will block till gpu is done, so hold on it here
    let mut previous_frame_end = Some(Box::new(sync::now(device.clone())) as Box<dyn GpuFuture>);


    // Start Game state initalization
    let mut state = MainState::new();

    // Game loop bits
    let mut accumulator = Duration::new(0, 0);
    let mut previous_clock = Instant::now();
    let speed = 1;


    // Event loop for the game
    // TODO: set it so that game logic runs inside EventsCleared
    events_loop.run(move |event, _, control_flow| {
        // Set it to poll so that it will spin nonstop
        *control_flow = ControlFlow::Poll;

        let window = surface.window();

        // Call this, so that garbage can be cleared
        previous_frame_end.as_mut().unwrap().cleanup_finished();

        // Handle window resize
        if recreate_swapchain {
            let (new_swapchain, new_images) = match swapchain.recreate_with_dimension(get_dimensions(&window)) {
                Ok(r) => r,
                // If user is resizing the window manually, that means its continiously changing so keep trying
                Err(SwapchainCreationError::UnsupportedDimensions) => return,
                Err(err) => panic!("{:?}", err)
            };

            swapchain = new_swapchain;
            framebuffers = window_size_dependent_setup(&new_images, render_pass.clone(), &mut dynamic_state);

            recreate_swapchain = false;
        }

        // Acquire image from the swapchain, if no image is available, block. (BLOCKABLE)
        let (image_num, acquire_future) = match swapchain::acquire_next_image(swapchain.clone(), None) {
            Ok(r) => r,
            Err(AcquireError::OutOfDate) => {
                recreate_swapchain = true;
                return;
            },
            Err(err) => panic!("{:?}", err)
        };

        // Clear the framebuffer with this color
        let clear_values = vec!([0.0, 0.0, 1.0, 1.0].into());



        // GAME LOGIC
        const DESIRED_FPS: u64 = 60;
        let seconds = 1.0 / (DESIRED_FPS as f64);

        let dimensions = get_dimensions(&window);
        let width = dimensions[0];
        let height = dimensions[1];

        // Update the player state based on the user input.
        player_handle_input(&mut state.player, &state.input, seconds);
        state.player_shot_timeout -= seconds;
        if state.input.fire && state.player_shot_timeout < 0.0 {
            state.fire_player_shot();
        }

        // Update the physics for all actors.
        // First the player...
        update_actor_position(&mut state.player, seconds);
        wrap_actor_position(&mut state.player, width as f64, height as f64);

        // Then the shots...
        for act in &mut state.shots {
            update_actor_position(act, seconds);
            wrap_actor_position(act, width as f64, height as f64);
            handle_timed_life(act, seconds);
        }

        // And finally the rocks.
        for act in &mut state.rocks {
            update_actor_position(act, seconds);
            wrap_actor_position(act, width as f64, height as f64);
        }

        // Handle the results of things moving:
        // collision detection, object death, and if
        // we have killed all the rocks in the level,
        // spawn more of them.
        state.handle_collisions();

        state.clear_dead_stuff();

        state.check_for_level_respawn();

        // Finally we check for our end state.
        // I want to have a nice death screen eventually,
        // but for now we just quit.
        if state.player.life <= 0.0 {
            *control_flow = ControlFlow::Exit;
        }


        // Build a command buffer (BLOCKABLE)
        let mut command_buffer_builder = AutoCommandBufferBuilder::primary_one_time_submit(device.clone(), queue.family()).unwrap()
            // Enter render pass and clear the screen
            .begin_render_pass(framebuffers[image_num].clone(), false, clear_values).unwrap();

        // Start drawing stuff
        let coords = (width, height);

        command_buffer_builder = draw_actor(
            pipeline.clone(),
            &dynamic_state,
            vertex_buffer.clone(),
            uniform_buffer.clone(),
            command_buffer_builder,
            &state.player,
            coords
        );

        for s in &state.shots {
            command_buffer_builder = draw_actor(
                pipeline.clone(),
                &dynamic_state,
                vertex_buffer.clone(),
                uniform_buffer.clone(),
                command_buffer_builder,
                s,
                coords
            );
        }

        for r in &state.rocks {
            command_buffer_builder = draw_actor(
                pipeline.clone(),
                &dynamic_state,
                vertex_buffer.clone(),
                uniform_buffer.clone(),
                command_buffer_builder,
                r,
                coords
            );
        }

        // Finish rendering
        // Finish building the command buffer by calling `build`.
        let command_buffer = command_buffer_builder.end_render_pass().unwrap().build().unwrap();

		let prev = previous_frame_end.take();
        let future = prev.unwrap().join(acquire_future)
            .then_execute(queue.clone(), command_buffer).unwrap()

            // Present the swapchain to the screen, will block till gpu is done drawing (BLOCKABLE)
            .then_swapchain_present(queue.clone(), swapchain.clone(), image_num)
            .then_signal_fence_and_flush();

        match future {
            Ok(future) => {
                // This wait is required when using NVIDIA or running on macOS.
                // See https://github.com/vulkano-rs/vulkano/issues/1247
                future.wait(None).unwrap();
                previous_frame_end = Some(Box::new(future) as Box<_>);
            }
            Err(FlushError::OutOfDate) => {
                recreate_swapchain = true;
                previous_frame_end = Some(Box::new(sync::now(device.clone())) as Box<_>);
            }
            Err(e) => {
                println!("{:?}", e);
                previous_frame_end = Some(Box::new(sync::now(device.clone())) as Box<_>);
            }
        }

        // Handle events + keyboard events
        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => *control_flow = ControlFlow::Exit,
            Event::WindowEvent { event: WindowEvent::Resized(_), .. } => recreate_swapchain = true,

            // Game Keyboard
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput {
                    input: KeyboardInput {
                        state: element_state,
                        virtual_keycode: Some(key), ..
                    }, ..
                }, ..
            } => {
                match key {
                    // Fire
                    VirtualKeyCode::Space => {
                        match element_state {
                            ElementState::Pressed => {
                                state.input.fire = true;
                            },
                            ElementState::Released => {
                                state.input.fire = false;
                            },
                        }
                    },

                    //  Up
                    VirtualKeyCode::Up => {
                        match element_state {
                            ElementState::Pressed => {
                                state.input.yaxis = 1.0;
                            },
                            ElementState::Released => {
                                state.input.yaxis = 0.0;
                            },
                        }
                    },

                    //  Left
                    VirtualKeyCode::Left => {
                        match element_state {
                            ElementState::Pressed => {
                                state.input.xaxis = -1.0;
                            },
                            ElementState::Released => {
                                state.input.xaxis = 0.0;
                            },
                        }
                    },

                    //  Right
                    VirtualKeyCode::Right => {
                        match element_state {
                            ElementState::Pressed => {
                                state.input.xaxis = 1.0;
                            },
                            ElementState::Released => {
                                state.input.xaxis = 0.0;
                            },
                        }
                    },
                    _ => (),
                }
            },
            _ => {},
        }

        // Handle clock
        let now = Instant::now();
        accumulator += now - previous_clock;
        previous_clock = now;

        let fixed_time_stamp = Duration::new(0, 16666667 / speed);
        while accumulator >= fixed_time_stamp {
            accumulator -= fixed_time_stamp;
        }

        thread::sleep(fixed_time_stamp - accumulator);
    });
}

// TODO: may be better to move the player state/etc in here so that i can accumulate all of the
// data needed, and then create a buffer, and shove a single draw at vulkan (instanced draw) and
// that should do the trick i think?
fn draw_actor<V, Gp>(
    pipeline: Gp,
    dynamic: &DynamicState,
    vertex_buffer: V,
    uniform_buffer: CpuBufferPool::<vs::ty::Data>,
    command_buffer_builder: AutoCommandBufferBuilder<StandardCommandPoolBuilder>,
    actor: &Actor,
    world_coords: (u32, u32)
) -> AutoCommandBufferBuilder<StandardCommandPoolBuilder>
    where Gp: GraphicsPipelineAbstract + VertexSource<V> + Send + Sync + 'static + Clone
{
    // TODO: improve this to batch up all of this, but for now (?) dump the data one by one at the
    // gpu, this shoul be getting a buffer where it put the per intance data onto (ie postion +
    // rotation matrixes) and instancing the shaders/draw with the model/view matrix
    //
    // Setup the Model View World matrixes
    let uniform_buffer_subbuffer = {
        // projection - world space
        //
        // The bounds clamp assumpes a ortho projection of -width/2 to width/2 and -height/2 to height/2
        // So that's where that is coming from. Still unclear on why the view needs a translation (i
        // think its due to the world_to_screen_coords)
        let projection: cgmath::Matrix4<f32> = cgmath::ortho(
            -320.0, 320.0,
            240.0, -240.0,
            0.1, 1024.0
        );


        // Screen
        let (width, height) = world_coords;

        // Fixed for now (view matrix) - identity
        // TODO: not sure why we needed the offset, but that fixed the othographic projection tho
        //let view = cgmath::Matrix4::identity();
        let view = cgmath::Matrix4::from_translation(cgmath::Vector3::new(
            -320.0,
            -240.0,
            0.0
        ));

        let pos = world_to_screen_coords(width, height, &actor.pos);
        let px = pos.x as f32;
        let py = pos.y as f32;

        // Model matrix
        let t = cgmath::Matrix4::from_translation(cgmath::Vector3::new(px, py, 0.0));
        let r = cgmath::Matrix4::from_angle_z(Rad((*&actor.facing).0 as f32));
        let s = cgmath::Matrix4::from_scale(20.0);

        let model: cgmath::Matrix4<f32> = t * r * s;


        // TODO: at least i know the model works (re rotation) going to work up from here
        let model = cgmath::Matrix4::identity() * cgmath::Matrix4::from_angle_z(Rad((*&actor.facing).0 as f32));
        let view = cgmath::Matrix4::identity();
        let projection = cgmath::Matrix4::identity();

        let uniform_data = vs::ty::Data {
            world: model.into(),
            view: view.into(),
            proj: projection.into(),
        };

        uniform_buffer.next(uniform_data).unwrap()
    };

    let set = Arc::new(PersistentDescriptorSet::start(pipeline.clone(), 0)
        .add_buffer(uniform_buffer_subbuffer).unwrap()
        .build().unwrap()
    );

    command_buffer_builder.draw(pipeline, dynamic, vertex_buffer, set.clone(), ()).unwrap()
}

/// This method is called once during initialization, then again whenever the window is resized
fn window_size_dependent_setup(
    images: &[Arc<SwapchainImage<Window>>],
    render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
    dynamic_state: &mut DynamicState
) -> Vec<Arc<dyn FramebufferAbstract + Send + Sync>> {
    let dimensions = images[0].dimensions();

    let viewport = Viewport {
        origin: [0.0, 0.0],
        dimensions: [dimensions[0] as f32, dimensions[1] as f32],
        depth_range: 0.0 .. 1.0,
    };
    dynamic_state.viewports = Some(vec!(viewport));

    images.iter().map(|image| {
        Arc::new(
            Framebuffer::start(render_pass.clone())
                .add(image.clone()).unwrap()
                .build().unwrap()
        ) as Arc<dyn FramebufferAbstract + Send + Sync>
    }).collect::<Vec<_>>()
}

fn get_dimensions(window: &Window) -> [u32; 2] {
    let dimensions: (u32, u32) = window.inner_size().to_physical(window.hidpi_factor()).into();
    [dimensions.0, dimensions.1]
}

mod vs {
    vulkano_shaders::shader!{
        ty: "vertex",
        src: "
#version 450

layout(location = 0) in vec2 position;

layout(set = 0, binding = 0) uniform Data {
    mat4 world;
    mat4 view;
    mat4 proj;
} uniforms;

void main() {
    mat4 worldview = uniforms.view * uniforms.world;
    gl_Position = uniforms.proj * worldview * vec4(position, 0.0, 1.0);
}"
    }
}

mod fs {
    vulkano_shaders::shader!{
        ty: "fragment",
        src: "
#version 450

layout(location = 0) out vec4 f_color;

void main() {
    f_color = vec4(0.0, 1.0, 0.0, 1.0);
}"
    }
}
