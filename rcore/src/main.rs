use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use bevy_rapier2d::prelude::*;

use rhai::{Engine, Scope, AST};

use std::iter::zip;
use std::boxed::Box;

// TODO:
// - faction
// - heat
// - hp
// - radar + shield -> Arc (direction + arc width)
#[derive(Component)]
struct Ship;

#[derive(Component)]
struct Velocity(Vec2);

#[derive(Component)]
struct Rotation(f32);

// Ref-counted collision, if greater than zero, its colloding, otherwise
#[derive(Component)]
struct Collision(u32);

// Primitive "Scripting" Component. Will develop in a more sophsicated interface to hook up to a VM
// later on
#[derive(Component)]
struct Script {
    scope: Scope<'static>,
    ast: Box<AST>,
}

#[derive(Resource)]
struct ScriptTimer(Timer);

fn process_scripts(
    time: Res<Time>,
    mut timer: ResMut<ScriptTimer>,
    mut query: Query<(Entity, &mut Script)>,
    mut ship_query: Query<(&mut Velocity, &mut Rotation, &Collision, &Transform)>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        // TODO:
        // Sum up the ship status/environment
        // Pass it into rhai somehow (callback or some sort of status object)
        // Run the script, and the script can return a list of changes to perform to the ship
        //  -or- invoke script functions directly to update a state that gets synchronized to the
        //  ship
        //  -or- just update the components directly?
        let mut engine = Engine::new();
        for (entity, mut script) in query.iter_mut() {

            let rot = ship_query.component::<Rotation>(entity).0;
            let tran = ship_query.component::<Transform>(entity).translation;
            let vel = ship_query.component::<Velocity>(entity).0;

            engine.register_fn("get_rotation", move || -> f32 { rot })
                .register_fn("get_position", move || -> (f32, f32) { (tran.x, tran.y) })
                .register_fn("get_velocity", move || -> (f32, f32) { (vel.x, vel.y) })
                .register_fn("log", |text: &str| {
                    println!("{text}");
                });

            let ast = script.ast.clone();
            let res = engine.run_ast_with_scope(&mut script.scope, &ast);

            println!("Script Result - {:?}", res);
        }
    }
}

fn new_script() -> Script {
    let script = r#"
    let pos = get_position();
    let vel = get_velocity();
    let rot = get_rotation();

    log("pos - " + pos + " vel - " + vel + " rot - " + rot);
    "#;

    let engine = Engine::new();
    let mut scope = Scope::new();
    let ast = match engine.compile_with_scope(&mut scope, &script) {
        Ok(ast) => ast,
        Err(x) => panic!("AST: {:?}", x),
    };

    Script { scope, ast: Box::new(ast) }
}

fn add_ships(mut commands: Commands) {
    let poss = vec![Vec2::new(50.0, 200.0), Vec2::new(300.0, 0.0), Vec2::new(-200., 0.), Vec2::new(200., 0.)];
    let velo = vec![Vec2::new(-3.0, 1.0), Vec2::new(-2.0, -3.0), Vec2::new(1.0, 0.), Vec2::new(-1.0, 0.)];
    let roto = vec![1.0, 2.0, 0.0, 0.0];

    for (pos, (vel, rot)) in zip(poss, zip(velo, roto)) {
        let path = {
            let mut path = PathBuilder::new();
            let _ = path.move_to(Vec2::new(0.0, 20.0));
            let _ = path.line_to(Vec2::new(10.0, -20.0));
            let _ = path.line_to(Vec2::new(0.0, -10.0));
            let _ = path.line_to(Vec2::new(-10.0, -20.0));
            let _ = path.close();
            path.build()
        };

        commands.spawn((
            ShapeBundle {
                path: path,
                spatial: SpatialBundle {
                    transform: Transform::from_xyz(pos.x, pos.y, 0.),
                    ..default()
                },
                ..default()
            },
            Stroke::new(Color::BLACK, 2.0),
            Fill::color(Color::GREEN),
        ))
            .insert(Ship)
            .insert(Velocity(vel))
            .insert(Rotation(rot))

            .insert(new_script())

            // TODO: probs want collision groups (ie ship vs missile vs other ships)
            .insert(Collider::cuboid(10.0, 20.0))
            .insert(ActiveCollisionTypes::empty() | ActiveCollisionTypes::STATIC_STATIC)
            .insert(ActiveEvents::COLLISION_EVENTS)
            .insert(Sensor)

            .insert(Collision(0));
    }
}

fn apply_velocity(mut query: Query<(&Velocity, &mut Transform)>) {
    for (vec, mut tran) in query.iter_mut() {
        tran.translation.x += vec.0.x;
        tran.translation.y += vec.0.y;
    }
}

fn apply_rotation(mut query: Query<(&Rotation, &mut Transform)>) {
    for (rot, mut tran) in query.iter_mut() {
        tran.rotation *= Quat::from_rotation_z(0.0174533 * rot.0);
    }
}

fn apply_collision(mut query: Query<(&Collision, &mut Fill)>) {
    for (collision, mut fill) in query.iter_mut() {
        if collision.0 == 0 {
            fill.color = Color::GREEN;
        } else {
            fill.color = Color::RED;
        }
    }
}

// collision detection
fn process_events(
    mut collision_events: EventReader<CollisionEvent>,
    mut query: Query<&mut Collision>,
) {
    for collision_event in collision_events.read() {
        match collision_event {
            //struct Collision(u32);
            CollisionEvent::Started(e1, e2, _) => {
                if let Ok([mut e1_collision, mut e2_collision]) = query.get_many_mut([*e1, *e2]) {
                    e1_collision.0 += 1;
                    e2_collision.0 += 1;
                } else {
                    println!("ERROR - ECS - {:?}", collision_event);
                }
            },
            CollisionEvent::Stopped(e1, e2, _) => {
                if let Ok([mut e1_collision, mut e2_collision]) = query.get_many_mut([*e1, *e2]) {
                    e1_collision.0 -= 1;
                    e2_collision.0 -= 1;
                } else {
                    println!("ERROR - ECS - {:?}", collision_event);
                }
            },
        }
    }
}

// TODO: decouple the rendering stuff somewhat from the rest of the system. Ie we
// still bundle the assets in the ECS, but have all of the system interact within
// the ECS then after things settle -> have a system that takes the ship plugin content
// system and update the sprite/assets/etc to display that information on the screen
struct ShipPlugins;
impl Plugin for ShipPlugins {
    fn build(&self, app: &mut App) {
        app.add_plugins(ShapePlugin)
            .add_systems(Startup, add_ships)

            .insert_resource(Time::<Fixed>::from_hz(64.0))

            .add_systems(
                FixedUpdate,
                (
                    apply_velocity,
                    apply_rotation,
                ),
            )
            .add_systems(Update, process_events)
            .add_systems(Update, apply_collision.after(process_events))

            .insert_resource(ScriptTimer(Timer::from_seconds(1.0 / 5.0, TimerMode::Repeating)))
            .add_systems(Update, process_scripts);
    }
}


// TODO: Temp size for now
pub const ARENA_WIDTH: f32 = 1024.0;
pub const ARENA_HEIGHT: f32 = 640.0;

struct ArenaPlugins;
impl Plugin for ArenaPlugins {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, camera_setup)
            .add_systems(Startup, add_arena_bounds)
            .add_systems(PostUpdate, wrap_arena);
    }
}

#[derive(Component)]
struct CameraMarker;

fn camera_setup(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle::default(),
        CameraMarker,
    ));
}

// Take care of any existing Transform to make sure it wraps around into the arena again
fn wrap_arena(mut query: Query<&mut Transform, Changed<Transform>>) {
    for mut tran in query.iter_mut() {
        if tran.translation.y < -(ARENA_HEIGHT / 2.0) {
            tran.translation.y += ARENA_HEIGHT;
        } else if tran.translation.y > (ARENA_HEIGHT / 2.0) {
            tran.translation.y -= ARENA_HEIGHT;
        }

        if tran.translation.x < -(ARENA_WIDTH / 2.0) {
            tran.translation.x += ARENA_WIDTH;
        } else if tran.translation.x > (ARENA_WIDTH / 2.0) {
            tran.translation.x -= ARENA_WIDTH;
        }
    }
}

#[derive(Component)]
struct ArenaMarker;
fn add_arena_bounds(mut commands: Commands) {
    let path = {
        let mut path = PathBuilder::new();
        let _ = path.move_to(Vec2::new(-(ARENA_WIDTH / 2.0), -(ARENA_HEIGHT / 2.0)));
        let _ = path.line_to(Vec2::new(ARENA_WIDTH / 2.0, -(ARENA_HEIGHT / 2.0)));
        let _ = path.line_to(Vec2::new(ARENA_WIDTH / 2.0, ARENA_HEIGHT / 2.0));
        let _ = path.line_to(Vec2::new(-(ARENA_WIDTH / 2.0), ARENA_HEIGHT / 2.0));
        let _ = path.close();
        path.build()
    };

    commands.spawn((
        ShapeBundle {
            path: path,
            spatial: SpatialBundle {
                transform: Transform::from_xyz(0., 0., -1.),
                ..default()
            },
            ..default()
        },
        Stroke::new(Color::RED, 1.0),
        Fill::color(Color::BLUE),
        ArenaMarker,
    ));
}

fn main() {
    App::new()
        .insert_resource(Msaa::default())
        .add_plugins(DefaultPlugins)
        .add_plugins(ArenaPlugins)
        .add_plugins(ShipPlugins)

        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(10.0))
        //.add_plugins(RapierDebugRenderPlugin::default())

        .run();
}
