#[macro_use]
extern crate glium;
extern crate glutin;
extern crate rand;
extern crate cgmath;

// Asteroidish game (based off the ggez example)
use std::time::{Duration, Instant};
use std::ops::{Add, AddAssign, Sub};
use std::thread;

pub use glium::backend::glutin_backend::GlutinFacade as Display;
use glium::Surface;

use cgmath::prelude::*;

/// *********************************************************************
/// Basic stuff.
/// First, we create a vector type.
/// You're probably better off using a real vector math lib but I
/// didn't want to add more dependencies and such.
/// **********************************************************************
#[derive(Debug, Copy, Clone)]
struct Vec2 {
    x: f64,
    y: f64,
}

impl Vec2 {
    fn new(x: f64, y: f64) -> Self {
        Vec2 { x: x, y: y }
    }

    /// Create a unit vector representing the
    /// given angle (in radians)
    fn from_angle(angle: f64) -> Self {
        let vx = angle.sin();
        let vy = angle.cos();
        Vec2 { x: vx, y: vy }
    }

    fn random(max_magnitude: f64) -> Self {
        let angle = rand::random::<f64>() * 2.0 * std::f64::consts::PI;
        let mag = rand::random::<f64>() * max_magnitude;
        Vec2::from_angle(angle).scaled(mag)
    }

    fn magnitude(&self) -> f64 {
        ((self.x * self.x) + (self.y * self.y)).sqrt()
    }

    fn normalized(&self) -> Self {
        let mag = self.magnitude();
        self.scaled(1.0 / mag)
    }

    fn scaled(&self, rhs: f64) -> Self {
        Vec2 {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }

    /// Returns a vector whose magnitude is between
    /// 0 and max.
    fn clamped(&self, max: f64) -> Self {
        let mag = self.magnitude();
        if mag > max {
            self.normalized().scaled(max)
        } else {
            *self
        }
    }
}

impl Add for Vec2 {
    type Output = Self;
    fn add(self, rhs: Vec2) -> Self {
        Vec2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}


impl AddAssign for Vec2 {
    fn add_assign(&mut self, rhs: Vec2) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}


impl Sub for Vec2 {
    type Output = Self;
    fn sub(self, rhs: Vec2) -> Self {
        Vec2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Default for Vec2 {
    fn default() -> Self {
        Self::new(0., 0.)
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
    pos: Vec2,
    facing: f64,
    velocity: Vec2,
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
        pos: Vec2::default(),
        facing: 0.,
        velocity: Vec2::default(),
        rvel: 0.,
        bbox_size: PLAYER_BBOX,
        life: PLAYER_LIFE,
    }
}

fn create_rock() -> Actor {
    Actor {
        tag: ActorType::Rock,
        pos: Vec2::default(),
        facing: 0.,
        velocity: Vec2::default(),
        rvel: 0.,
        bbox_size: ROCK_BBOX,
        life: ROCK_LIFE,
    }
}

fn create_shot() -> Actor {
    Actor {
        tag: ActorType::Shot,
        pos: Vec2::default(),
        facing: 0.,
        velocity: Vec2::default(),
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
fn create_rocks(num: i32, exclusion: &Vec2, min_radius: f64, max_radius: f64) -> Vec<Actor> {
    assert!(max_radius > min_radius);
    let new_rock = |_| {
        let mut rock = create_rock();
        let r_angle = rand::random::<f64>() * 2.0 * std::f64::consts::PI;
        let r_distance = rand::random::<f64>() * (max_radius - min_radius) + min_radius;
        rock.pos = Vec2::from_angle(r_angle).scaled(r_distance) + *exclusion;
        rock.velocity = Vec2::random(MAX_ROCK_VEL);
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
    actor.facing += dt * PLAYER_TURN_RATE * input.xaxis;

    if input.yaxis > 0.0 {
        player_thrust(actor, dt);
    }
}

fn player_thrust(actor: &mut Actor, dt: f64) {
    let direction_vector = Vec2::from_angle(actor.facing);
    let thrust_vector = direction_vector.scaled(PLAYER_THRUST);
    actor.velocity += thrust_vector.scaled(dt);
}

const MAX_PHYSICS_VEL: f64 = 250.0;

fn update_actor_position(actor: &mut Actor, dt: f64) {
    actor.velocity = actor.velocity.clamped(MAX_PHYSICS_VEL);
    let dv = actor.velocity.scaled(dt);
    actor.pos += dv;
    actor.facing += actor.rvel;
}

/// Takes an actor and wraps its position to the bounds of the
/// screen, so if it goes off the left side of the screen it
/// will re-enter on the right side and so on.
fn wrap_actor_position(actor: &mut Actor, sx: f64, sy: f64) {
    // Wrap screen
    let screen_x_bounds = sx / 2.0;
    let screen_y_bounds = sy / 2.0;
    let sprite_half_size = (SPRITE_SIZE / 2) as f64;
    let actor_center = actor.pos - Vec2::new(-sprite_half_size, sprite_half_size);
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
fn world_to_screen_coords(screen_width: u32, screen_height: u32, point: &Vec2) -> Vec2 {
    let width = screen_width as f64;
    let height = screen_height as f64;
    let x = point.x + width / 2.0;
    let y = height - (point.y + height / 2.0);
    Vec2 { x: x, y: y }
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



#[derive(Copy, Clone)]
pub struct Vertex {
    position: [f32; 3]
}

implement_vertex!(Vertex, position);

pub const SHIP: [Vertex; 9] = [
    Vertex { position: [-0.3, -0.3,   0.0] },
    Vertex { position: [ 0.0,  0.5,   0.0] },
    Vertex { position: [ 0.3, -0.3,   0.0] },
    Vertex { position: [-0.3, -0.3,   0.0] },
    Vertex { position: [ 0.0,  0.25,  0.0] },
    Vertex { position: [ 0.3, -0.3,   0.0] },
    Vertex { position: [-0.3, -0.3,   0.0] },
    Vertex { position: [ 0.0,  0.0,   0.0] },
    Vertex { position: [ 0.3, -0.3,   0.0] },
];

pub const THRUST: [Vertex; 6] = [
    Vertex { position: [-0.3,  -0.3,   0.0] },
    Vertex { position: [-0.15, -0.45,  0.0] },
    Vertex { position: [ 0.0,  -0.3,   0.0] },
    Vertex { position: [-0.0,  -0.3,   0.0] },
    Vertex { position: [ 0.15, -0.45,  0.0] },
    Vertex { position: [ 0.3,  -0.3,   0.0] },
];





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
        let direction = Vec2::from_angle(shot.facing);
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













// Main stuff
fn main() {
    use glium::{DisplayBuild, Surface};
    let display = glium::glutin::WindowBuilder::new().with_depth_buffer(24).build_glium().unwrap();


    // Shaders
    let vertex_shader_src = r#"
        #version 140

        in vec3 position;

        uniform mat4 projection;
        uniform mat4 view;
        uniform mat4 model;

        void main() {
            mat4 modelview = view * model;
            //gl_Position = projection * modelview * vec4(position, 1.0);
            gl_Position = modelview * vec4(position, 1.0) * projection;
        }
    "#;

    let fragment_shader_src = r#"
        #version 140

        out vec4 color;

        void main() {
            color = vec4(0.0, 1.0, 0.0, 1.0);
        }
    "#;

    // Fucking &display
    let program = glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).unwrap();

    // Ship
    let ship_shape = glium::vertex::VertexBuffer::new(&display, &SHIP).unwrap();


    // Main state
    let mut state = MainState::new();

    // Game loop bits
    let mut accumulator = Duration::new(0, 0);
    let mut previous_clock = Instant::now();
    let speed = 1;

    loop {
        let mut target = display.draw();
        //let (width, height) = target.get_dimensions();
        let width = 640;
        let height = 480;

        const DESIRED_FPS: u64 = 60;
        let seconds = 1.0 / (DESIRED_FPS as f64);

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
            target.finish().unwrap();
            return;
        }


        // Our drawing is quite simple.
        // Just clear the screen...
        target.clear_color_and_depth((0.0, 0.0, 1.0, 1.0), 1.0);

        // Loop over all objects drawing them...
        {
            let coords = (width, height);

            let p = &state.player;
            draw_actor(&mut target, &program, &ship_shape, p, coords);

            for s in &state.shots {
                draw_actor(&mut target, &program, &ship_shape, s, coords);
            }

            for r in &state.rocks {
                draw_actor(&mut target, &program, &ship_shape, r, coords);
            }
        }

        target.finish().unwrap();

        for ev in display.poll_events() {
            match ev {
                glium::glutin::Event::Closed => return,

                glutin::Event::KeyboardInput(glutin::ElementState::Pressed, _, Some(glutin::VirtualKeyCode::Space)) => {
                    state.input.fire = true;
                },
                glutin::Event::KeyboardInput(glutin::ElementState::Released, _, Some(glutin::VirtualKeyCode::Space)) => {
                    state.input.fire = false;
                },

                glutin::Event::KeyboardInput(glutin::ElementState::Pressed, _, Some(glutin::VirtualKeyCode::Up)) => {
                    state.input.yaxis = 1.0;
                },
                glutin::Event::KeyboardInput(glutin::ElementState::Released, _, Some(glutin::VirtualKeyCode::Up)) => {
                    state.input.yaxis = 0.0;
                },

                glutin::Event::KeyboardInput(glutin::ElementState::Pressed, _, Some(glutin::VirtualKeyCode::Left)) => {
                    state.input.xaxis = -1.0;
                },
                glutin::Event::KeyboardInput(glutin::ElementState::Released, _, Some(glutin::VirtualKeyCode::Left)) => {
                    state.input.xaxis = 0.0;
                },

                glutin::Event::KeyboardInput(glutin::ElementState::Pressed, _, Some(glutin::VirtualKeyCode::Right)) => {
                    state.input.xaxis = 1.0;
                },
                glutin::Event::KeyboardInput(glutin::ElementState::Released, _, Some(glutin::VirtualKeyCode::Right)) => {
                    state.input.xaxis = 0.0;
                },

                _ => ()
            }
        }

        let now = Instant::now();
        accumulator += now - previous_clock;
        previous_clock = now;

        let fixed_time_stamp = Duration::new(0, 16666667 / speed);
        while accumulator >= fixed_time_stamp {
            accumulator -= fixed_time_stamp;
        }

        thread::sleep(fixed_time_stamp - accumulator);
    }
}

fn draw_actor<R>(target: &mut glium::Frame, program: &glium::Program, shape: &glium::VertexBuffer<R>, actor: &Actor, world_coords: (u32, u32)) -> () where R: std::marker::Copy {
    // Parameters
    let params = glium::DrawParameters {
        depth: glium::Depth {
            test: glium::draw_parameters::DepthTest::IfLess,
            write: true,
            .. Default::default()
        },
        //backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
        .. Default::default()
    };

    // Fixed for now (view matrix) - identity
    // TODO: not sure why we needed the offset, but that fixed the othographic projection tho
    let view = cgmath::Matrix4::from_translation(cgmath::Vector3::new(-320.0, -240.0, 0.0));

    // Projection
    let projection: cgmath::Matrix4<f32> = cgmath::ortho(-320.0, 320.0, 240.0, -240.0, 0.1, 1024.0);

    let (screen_w, screen_h) = world_coords;
    let pos = world_to_screen_coords(screen_w, screen_h, &actor.pos);
    let px = pos.x as f32;
    let py = pos.y as f32;

    // Model matrix
    let t = cgmath::Matrix4::from_translation(cgmath::Vector3::new(px, py, 0.0));
    let r = cgmath::Matrix4::from_angle_z(cgmath::Rad(*&actor.facing as f32));
    let s = cgmath::Matrix4::from_scale(20.0);

    let model: cgmath::Matrix4<f32> = t * r * s;

    // SHIP
    target.draw(
        shape,
        glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip),
        program,
        &uniform! {
            model: Into::<[[f32; 4]; 4]>::into(model),
            view: Into::<[[f32; 4]; 4]>::into(view),
            projection: Into::<[[f32; 4]; 4]>::into(projection)
        },
        &params
    ).unwrap();
}
