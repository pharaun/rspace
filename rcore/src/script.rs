use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use rhai::{Engine, Scope, AST};

use std::boxed::Box;

use crate::ship::{Velocity, Rotation, Collision};

// Primitive "Scripting" Component. Will develop in a more sophsicated interface to hook up to a VM
// later on
#[derive(Component)]
pub struct Script {
    scope: Scope<'static>,
    ast: Box<AST>,
}

#[derive(Resource)]
struct ScriptTimer(Timer);

#[derive(Resource)]
pub struct ScriptEngine(Engine);

pub struct ScriptPlugins;
impl Plugin for ScriptPlugins {
    fn build(&self, app: &mut App) {
        app.insert_resource(ScriptTimer(Timer::from_seconds(1.0 / 1.0, TimerMode::Repeating)))
            .insert_resource(ScriptEngine(new_engine()))
            .add_systems(Update, process_scripts);
    }
}

// TODO: should also have a way to process events (ie on collision an event is emitted, invoke the
// entity's script on_collision function or something)
fn process_scripts(
    time: Res<Time>,
    mut timer: ResMut<ScriptTimer>,
    script_engine: Res<ScriptEngine>,
    mut collision_events: EventReader<CollisionEvent>,
    mut query: Query<(Entity, &mut Script)>,
    mut ship_query: Query<(&Velocity, &Collision, &mut Rotation, &Transform)>,
) {
    // Handle collision events first
    for collision_event in collision_events.read() {
        match collision_event {
            //struct Collision(u32);
            CollisionEvent::Started(e1, e2, _) => {
                if let Ok([(_, mut e1_script), (_, mut e2_script)]) = query.get_many_mut([*e1, *e2]) {

                    let e1_ast = e1_script.ast.clone();
                    let res = script_engine.0.call_fn::<()>(
                        &mut e1_script.scope,
                        &e1_ast,
                        "on_collision",
                        (),
                    );
                    println!("Script Result - {:?}", res);

                    let e2_ast = e2_script.ast.clone();
                    let res = script_engine.0.call_fn::<()>(
                        &mut e2_script.scope,
                        &e2_ast,
                        "on_collision",
                        (),
                    );
                    println!("Script Result - {:?}", res);

                } else {
                    println!("ERROR - SCRIPT - {:?}", collision_event);
                }
            },
            _ => (),
        }
    }

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
            let vel = ship_query.component::<Velocity>(entity).0;

            // TODO: probs want to have a place for scripts to store their states and supply it to
            // each run since functions can't access top level global variables and yeah...
            let ast = script.ast.clone();

            // TODO: accept the target rotation value out of the script via the return value
            let res = script_engine.0.call_fn::<f32>(
                &mut script.scope,
                &ast,
                "on_update",
                ( tran.truncate(), vel, rot.to_euler(EulerRot::ZYX).0 ),
            );

            match res {
                Ok(to_rot) => {
                    let mut rotation = ship_query.component_mut::<Rotation>(entity);
                    // TODO: update this only when the target changes
                    rotation.target = Quat::from_rotation_z(to_rot);
                    rotation.current = rot;
                    rotation.start_time = time.elapsed_seconds();
                },
                Err(e) => println!("Script Error - {:?}", e),
            }
        }
    }
}

fn new_engine() -> Engine {
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

    engine
}

pub fn new_script(script: &str, script_engine: &Res<ScriptEngine>) -> Script {
    // TODO: probs want to do initial run to initialize global values and stuff
    // before invoking all future runs via event-function calls
    let scope = Scope::new();
    let ast = match script_engine.0.compile(&script) {
        Ok(ast) => ast,
        Err(x) => panic!("AST: {:?}", x),
    };

    Script { scope, ast: Box::new(ast) }
}
