#[macro_use]
extern crate glium;
extern crate rand;

// Asteroidish game (based off the ggez example)
use std::time::Duration;
use std::ops::{Add, AddAssign, Sub};


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



/// Main
fn main() {
    use glium::{DisplayBuild, Surface};
    let display = glium::glutin::WindowBuilder::new().with_depth_buffer(24).build_glium().unwrap();

    let shape = glium::vertex::VertexBuffer::new(&display, &SHIP).unwrap();

    let vertex_shader_src = r#"
        #version 140

        in vec3 position;

        uniform mat4 perspective;
        uniform mat4 view;
        uniform mat4 model;

        void main() {
            mat4 modelview = view * model;
            gl_Position = perspective * modelview * vec4(position, 1.0);
        }
    "#;

    let fragment_shader_src = r#"
        #version 140

        out vec4 color;

        void main() {
            color = vec4(0.0, 1.0, 0.0, 1.0);
        }
    "#;

    let program = glium::Program::from_source(&display, vertex_shader_src, fragment_shader_src, None).unwrap();



    loop {
        let model = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0f32]
        ];

        let params = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::draw_parameters::DepthTest::IfLess,
                write: true,
                .. Default::default()
            },
            //backface_culling: glium::draw_parameters::BackfaceCullingMode::CullClockwise,
            .. Default::default()
        };

        let mut target = display.draw();
        //target.clear_color(0.0, 0.0, 1.0, 1.0);
        target.clear_color_and_depth((0.0, 0.0, 1.0, 1.0), 1.0);

        let view = view_matrix(&[0.0, 0.0, -3.0], &[0.0, 0.0, 3.0], &[0.0, 1.0, 0.0]);

        let (width, height) = target.get_dimensions();
        let perspective = prespective(width, height);



        // SHIP
        target.draw(&shape, glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip), &program, &uniform! { model: model, view: view, perspective: perspective }, &params).unwrap();


        target.finish().unwrap();

        for ev in display.poll_events() {
            match ev {
                glium::glutin::Event::Closed => return,
                _ => ()
            }
        }
    }
}



















///// **********************************************************************
///// Now we're getting into the actual game loop.  The `MainState` is our
///// game's "global" state, it keeps track of everything we need for
///// actually running the game.
/////
///// Our game objects are simply a vector for each actor type, and we
///// probably mingle gameplay-state (like score) and hardware-state
///// (like gui_`dirty`) a little more than we should, but for something
///// this small it hardly matters.
///// **********************************************************************
//
//struct MainState {
//    player: Actor,
//    shots: Vec<Actor>,
//    rocks: Vec<Actor>,
//    level: i32,
//    score: i32,
//    screen_width: u32,
//    screen_height: u32,
//    input: InputState,
//    player_shot_timeout: f64,
//}
//
//
//impl MainState {
//    fn new(ctx: &mut Context) -> GameResult<MainState> {
//        ctx.print_resource_stats();
//        graphics::set_background_color(ctx, (0, 0, 0, 255).into());
//
//        println!("Game resource path: {:?}", ctx.filesystem);
//
//        print_instructions();
//
//        let assets = Assets::new(ctx)?;
//        //let score_disp = graphics::Text::new(ctx, "score", &assets.font)?;
//        //let level_disp = graphics::Text::new(ctx, "level", &assets.font)?;
//
//        let player = create_player();
//        let rocks = create_rocks(5, &player.pos, 100.0, 250.0);
//
//        let s = MainState {
//            player: player,
//            shots: Vec::new(),
//            rocks: rocks,
//            level: 0,
//            score: 0,
//            assets: assets,
//            screen_width: ctx.conf.window_width,
//            screen_height: ctx.conf.window_height,
//            input: InputState::default(),
//            player_shot_timeout: 0.0,
//        };
//
//        Ok(s)
//    }
//
//    fn fire_player_shot(&mut self) {
//        self.player_shot_timeout = PLAYER_SHOT_TIME;
//
//        let player = &self.player;
//        let mut shot = create_shot();
//        shot.pos = player.pos;
//        shot.facing = player.facing;
//        let direction = Vec2::from_angle(shot.facing);
//        shot.velocity.x = SHOT_SPEED * direction.x;
//        shot.velocity.y = SHOT_SPEED * direction.y;
//
//        self.shots.push(shot);
//    }
//
//
//
//    fn clear_dead_stuff(&mut self) {
//        self.shots.retain(|s| s.life > 0.0);
//        self.rocks.retain(|r| r.life > 0.0);
//    }
//
//    fn handle_collisions(&mut self) {
//        for rock in &mut self.rocks {
//            let pdistance = rock.pos - self.player.pos;
//            if pdistance.magnitude() < (self.player.bbox_size + rock.bbox_size) {
//                self.player.life = 0.0;
//            }
//            for shot in &mut self.shots {
//                let distance = shot.pos - rock.pos;
//                if distance.magnitude() < (shot.bbox_size + rock.bbox_size) {
//                    shot.life = 0.0;
//                    rock.life = 0.0;
//                    self.score += 1;
//                }
//            }
//        }
//    }
//
//    fn check_for_level_respawn(&mut self) {
//        if self.rocks.is_empty() {
//            self.level += 1;
//            let r = create_rocks(self.level + 5, &self.player.pos, 100.0, 250.0);
//            self.rocks.extend(r);
//        }
//    }
//}
//
//
//
//
///// **********************************************************************
///// A couple of utility functions.
///// **********************************************************************
//
//fn print_instructions() {
//    println!();
//    println!("Welcome to ASTROBLASTO!");
//    println!();
//    println!("How to play:");
//    println!("L/R arrow keys rotate your ship, up thrusts, space bar fires");
//    println!();
//}
//
//
//fn draw_actor(assets: &mut Assets,
//              ctx: &mut Context,
//              actor: &Actor,
//              world_coords: (u32, u32))
//              -> GameResult<()> {
//    let (screen_w, screen_h) = world_coords;
//    let pos = world_to_screen_coords(screen_w, screen_h, &actor.pos);
//    // let pos = Vec2::new(1.0, 1.0);
//    let px = pos.x as f32;
//    let py = pos.y as f32;
//    let dest_point = graphics::Point::new(px, py);
//    let image = assets.actor_image(actor);
//    graphics::draw(ctx, image, dest_point, actor.facing as f32)
//
//}
//
//
//
///// **********************************************************************
///// Now we implement the `EventHandler` trait from `ggez::event`, which provides
///// ggez with callbacks for updating and drawing our game, as well as
///// handling input events.
///// **********************************************************************
//impl EventHandler for MainState {
//    fn update(&mut self, ctx: &mut Context, _dt: Duration) -> GameResult<()> {
//        const DESIRED_FPS: u64 = 60;
//        if !timer::check_update_time(ctx, DESIRED_FPS) {
//            return Ok(());
//        }
//        let seconds = 1.0 / (DESIRED_FPS as f64);
//
//        // Update the player state based on the user input.
//        player_handle_input(&mut self.player, &self.input, seconds);
//        self.player_shot_timeout -= seconds;
//        if self.input.fire && self.player_shot_timeout < 0.0 {
//            self.fire_player_shot();
//        }
//
//        // Update the physics for all actors.
//        // First the player...
//        update_actor_position(&mut self.player, seconds);
//        wrap_actor_position(&mut self.player,
//                            self.screen_width as f64,
//                            self.screen_height as f64);
//
//        // Then the shots...
//        for act in &mut self.shots {
//            update_actor_position(act, seconds);
//            wrap_actor_position(act, self.screen_width as f64, self.screen_height as f64);
//            handle_timed_life(act, seconds);
//        }
//
//        // And finally the rocks.
//        for act in &mut self.rocks {
//            update_actor_position(act, seconds);
//            wrap_actor_position(act, self.screen_width as f64, self.screen_height as f64);
//        }
//
//        // Handle the results of things moving:
//        // collision detection, object death, and if
//        // we have killed all the rocks in the level,
//        // spawn more of them.
//        self.handle_collisions();
//
//        self.clear_dead_stuff();
//
//        self.check_for_level_respawn();
//
//        // Finally we check for our end state.
//        // I want to have a nice death screen eventually,
//        // but for now we just quit.
//        if self.player.life <= 0.0 {
//            println!("Game over!");
//            let _ = ctx.quit();
//        }
//        Ok(())
//    }
//
//    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
//        // Our drawing is quite simple.
//        // Just clear the screen...
//        graphics::clear(ctx);
//
//        // Loop over all objects drawing them...
//        {
//            let assets = &mut self.assets;
//            let coords = (self.screen_width, self.screen_height);
//
//            let p = &self.player;
//            draw_actor(assets, ctx, p, coords)?;
//
//            for s in &self.shots {
//                draw_actor(assets, ctx, s, coords)?;
//            }
//
//            for r in &self.rocks {
//                draw_actor(assets, ctx, r, coords)?;
//            }
//        }
//
//
//        // Then we flip the screen...
//        graphics::present(ctx);
//        timer::sleep(Duration::from_secs(0));
//        Ok(())
//    }
//
//    // Handle key events.  These just map keyboard events
//    // and alter our input state appropriately.
//    fn key_down_event(&mut self, keycode: Keycode, _keymod: Mod, _repeat: bool) {
//        match keycode {
//            Keycode::Up => {
//                self.input.yaxis = 1.0;
//            }
//            Keycode::Left => {
//                self.input.xaxis = -1.0;
//            }
//            Keycode::Right => {
//                self.input.xaxis = 1.0;
//            }
//            Keycode::Space => {
//                self.input.fire = true;
//            }
//            _ => (), // Do nothing
//        }
//    }
//
//
//    fn key_up_event(&mut self, keycode: Keycode, _keymod: Mod, _repeat: bool) {
//        match keycode {
//            Keycode::Up => {
//                self.input.yaxis = 0.0;
//            }
//            Keycode::Left | Keycode::Right => {
//                self.input.xaxis = 0.0;
//            }
//            Keycode::Space => {
//                self.input.fire = false;
//            }
//            _ => (), // Do nothing
//        }
//    }
//}
//
//
//
///// **********************************************************************
///// Finally our main function!  Which merely sets up a config and calls
///// `ggez::event::run()` with our `EventHandler` type.
///// **********************************************************************
//
//pub fn main_asdf() {
//    let mut c = conf::Conf::new();
//    c.window_title = "Astroblasto!".to_string();
//    c.window_width = 640;
//    c.window_height = 480;
//    c.window_icon = "/player.png".to_string();
//
//    let ctx = &mut Context::load_from_conf("astroblasto", "ggez", c).unwrap();
//
//    match MainState::new(ctx) {
//        Err(e) => {
//            println!("Could not load game!");
//            println!("Error: {}", e);
//        }
//        Ok(ref mut game) => {
//            let result = run(ctx, game);
//            if let Err(e) = result {
//                println!("Error encountered running game: {}", e);
//            } else {
//                println!("Game exited cleanly.");
//            }
//        }
//    }
//}



fn view_matrix(position: &[f32; 3], direction: &[f32; 3], up: &[f32; 3]) -> [[f32; 4]; 4] {
    let f = {
        let f = direction;
        let len = f[0] * f[0] + f[1] * f[1] + f[2] * f[2];
        let len = len.sqrt();
        [f[0] / len, f[1] / len, f[2] / len]
    };

    let s = [up[1] * f[2] - up[2] * f[1],
             up[2] * f[0] - up[0] * f[2],
             up[0] * f[1] - up[1] * f[0]];

    let s_norm = {
        let len = s[0] * s[0] + s[1] * s[1] + s[2] * s[2];
        let len = len.sqrt();
        [s[0] / len, s[1] / len, s[2] / len]
    };

    let u = [f[1] * s_norm[2] - f[2] * s_norm[1],
             f[2] * s_norm[0] - f[0] * s_norm[2],
             f[0] * s_norm[1] - f[1] * s_norm[0]];

    let p = [-position[0] * s_norm[0] - position[1] * s_norm[1] - position[2] * s_norm[2],
             -position[0] * u[0] - position[1] * u[1] - position[2] * u[2],
             -position[0] * f[0] - position[1] * f[1] - position[2] * f[2]];

    [
        [s[0], u[0], f[0], 0.0],
        [s[1], u[1], f[1], 0.0],
        [s[2], u[2], f[2], 0.0],
        [p[0], p[1], p[2], 1.0],
    ]
}

fn prespective(width: u32, height: u32) -> [[f32; 4]; 4] {
    let aspect_ratio = height as f32 / width as f32;

    let fov: f32 = 3.141592 / 3.0;
    let zfar = 1024.0;
    let znear = 0.1;

    let f = 1.0 / (fov / 2.0).tan();

    [
        [f *   aspect_ratio   ,    0.0,              0.0              ,   0.0],
        [         0.0         ,     f ,              0.0              ,   0.0],
        [         0.0         ,    0.0,  (zfar+znear)/(zfar-znear)    ,   1.0],
        [         0.0         ,    0.0, -(2.0*zfar*znear)/(zfar-znear),   0.0],
    ]
}

fn othographic(left: i32, right: i32, top: i32, bottom: i32) -> [[f32; 4]; 4] {
    let l = left as f32;
    let r = right as f32;
    let t = top as f32;
    let b = bottom as f32;

    let zfar  = 1024.0;
    let znear = 0.1;

    [
        [ 2.0 / (r - l) , 0.0           ,  0.0                  , -((r + l) / (r - l))               ],
        [ 0.0           , 2.0 / (t - b) ,  0.0                  , -((t + b) / (t - b))               ],
        [ 0.0           , 0.0           , -2.0 / (zfar - znear) , -((zfar + znear) / (zfar - znear)) ],
        [ 0.0           , 0.0           ,  0.0                  ,  1.0                               ],
    ]
}
