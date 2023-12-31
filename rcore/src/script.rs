use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use rhai::{Engine, Scope, AST, Dynamic, CallFnOptions, Map};

use std::boxed::Box;

use crate::ship::{Velocity, Rotation, Collision};

// Primitive "Scripting" Component. Will develop in a more sophsicated interface to hook up to a VM
// later on
#[derive(Component)]
pub struct Script {
    scope: Scope<'static>,
    state: Box<Dynamic>,
    ast: Box<AST>,
}

impl Script {
    pub fn new(script: &str, engine: &Res<ScriptEngine>) -> Script {
        // Compile script
        let ast = match engine.0.compile(&script) {
            Ok(ast) => ast,
            Err(x) => panic!("AST: {:?}", x),
        };

        let mut scope = Scope::new();
        let mut state: Box<Dynamic> = Box::new(Map::new().into());

        // Init the script
        let options = CallFnOptions::new()
            .eval_ast(false)
            .bind_this_ptr(&mut state);

        let res = engine.0.call_fn_with_options::<()>(
            options,
            &mut scope,
            &ast,
            "init",
            (),
        );

        match res {
            Ok(()) => (),
            Err(e) => println!("Script Error - init - {:?}", e),
        }

        Script { scope, state, ast: Box::new(ast) }
    }
}

#[derive(Resource)]
pub struct ScriptEngine(Engine);

impl ScriptEngine {
    pub fn new() -> ScriptEngine {
        let mut engine = Engine::new();

        engine.register_type_with_name::<Vec2>("Vec2")
            .register_fn("new_vec2", |x: f64, y: f64| {
                Vec2::new(x as f32, y as f32)
            })
            .register_fn("to_string", |vec: &mut Vec2| vec.to_string())
            .register_fn("to_debug", |vec: &mut Vec2| format!("{vec:?}"))
            .register_get("x", |vec: &mut Vec2| vec.x as f64)
            .register_get("y", |vec: &mut Vec2| vec.y as f64);

        engine.register_fn("log", |text: &str| {
            println!("{text}");
        });

        ScriptEngine(engine)
    }
}

#[derive(Resource)]
struct ScriptTimer(Timer);

pub struct ScriptPlugins;
impl Plugin for ScriptPlugins {
    fn build(&self, app: &mut App) {
        app.insert_resource(ScriptTimer(Timer::from_seconds(1.0 / 1.0, TimerMode::Repeating)))
            .insert_resource(ScriptEngine::new())
            .add_systems(Update, process_on_update)
            .add_systems(Update, process_on_collision);
    }
}

fn process_on_collision(
    engine: Res<ScriptEngine>,
    mut collision_events: EventReader<CollisionEvent>,
    mut query: Query<(Entity, &mut Script)>,
) {
    // Handle collision events first
    for collision_event in collision_events.read() {
        match collision_event {
            CollisionEvent::Started(e1, e2, _) => {
                if let Ok([(_, e1_script), (_, e2_script)]) = query.get_many_mut([*e1, *e2]) {
                    for mut script in [e1_script, e2_script] {
                        let ast = script.ast.clone();
                        let mut state = script.state.clone();

                        let options = CallFnOptions::new()
                            .eval_ast(false)
                            .bind_this_ptr(&mut state);

                        let res = engine.0.call_fn_with_options::<()>(
                            options,
                            &mut script.scope,
                            &ast,
                            "on_collision",
                            (),
                        );

                        match res {
                            Ok(()) => (),
                            Err(e) => println!("Script Error - on_collision - {:?}", e),
                        }
                    }
                } else {
                    println!("ERROR - SCRIPT - {:?}", collision_event);
                }
            },
            _ => (),
        }
    }
}

fn process_on_update(
    time: Res<Time>,
    mut timer: ResMut<ScriptTimer>,
    engine: Res<ScriptEngine>,
    mut query: Query<(Entity, &mut Script)>,
    mut ship_query: Query<(&mut Velocity, &Collision, &mut Rotation, &Transform)>,
) {
    // handle normal on_update ticks
    if timer.0.tick(time.delta()).just_finished() {
        // TODO:
        // Sum up the ship status/environment
        // Pass it into rhai somehow (callback or some sort of status object)
        // Run the script, and the script can return a list of changes to perform to the ship
        //  -or- invoke script functions directly to update a state that gets synchronized to the
        //  ship
        //  -or- just update the components directly?
        for (entity, mut script) in query.iter_mut() {

            let trans = ship_query.component::<Transform>(entity);

            let rot = trans.rotation;
            let tran = trans.translation;
            let vel = ship_query.component::<Velocity>(entity).target.length();

            let ast = script.ast.clone();
            let mut state = script.state.clone();

            let options = CallFnOptions::new()
                .eval_ast(false)
                .bind_this_ptr(&mut state);

            // [ to_rot, to_vel ]
            let res = engine.0.call_fn_with_options::<rhai::Array>(
                options,
                &mut script.scope,
                &ast,
                "on_update",
                ( tran.truncate(), vel, rot.to_euler(EulerRot::ZYX).0 ),
            );

            match res {
                Ok(data) => {
                    let to_rot: f32 = data[0].clone_cast();
                    let mut rotation = ship_query.component_mut::<Rotation>(entity);
                    // TODO: update this only when the target changes
                    rotation.target = Quat::from_rotation_z(to_rot);

                    let to_mov: f32 = data[1].clone_cast();
                    let mut velocity = ship_query.component_mut::<Velocity>(entity);
                    velocity.target = Quat::from_rotation_z(to_rot).mul_vec3(Vec3::Y * to_mov).truncate();
                },
                Err(e) => println!("Script Error - {:?}", e),
            }
        }
    }
}
