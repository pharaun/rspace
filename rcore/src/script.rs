use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use rhai::{Engine, Scope, AST, Dynamic, CallFnOptions, Map, EvalAltResult, Variant, FuncArgs};

use std::boxed::Box;

use crate::ship::{
    movement::Velocity,
    rotation::Rotation,
    collision::Collision,
};

// TODO: Design
//
// I'm not sure bout the precise design right now but I think some sort of system that has this
// flow could work
// 1. iterate over all entity with an Script AI
// 2. encode relevant "hardware" state from the ECS
// 3. feed it to the VM, the vm + script does stuff to it
// 4. script exits/suspend
// 5. reconcile changes to "hardware" state back to ECS
//
// I think that basic loop is fine, the problem is how to have a good usable experience in the
// scripting language. Like it would be nice to be able at some level be able to tell the ship to
// go to a specific heading and wait till it is before proceeding with the program.
//
// The alternative is to run the program everytime till the heading matches the one you want, then
// you resume your code.
//
// This i think leads to a more event-based code design which leads to breaking up of the logic
// sometime and more state management in the scripting, tho thisi s kinda closer to hardware, where
// you can poll the hardware till a desirable state happen.
//
// however in hardware you can also tell the cpu to sleep till it gets woken up (interrupt).
//
// The advantage of the event flow is that you can enter it every tick, and execute some code, till
// the event you are interested in happen then you do the thing. But it can make reasoning about
// certain code flows harder instead of being like:
// go to a - wait
// go to b - wait
// go to c - wait
//
// in the code, it leads you to be like
// go to a, return
// event -> is at A yet, no return
// ....
// event -> is at A, yes, goto b return

// Use this to develop what we need for the future alternative language/VM but for now rhai will do
#[derive(Component)]
pub struct Script {
    scope: Scope<'static>,
    state: Dynamic,
    ast: AST,
}

impl Script {
    pub fn new(script: &str, engine: &Res<ScriptEngine>) -> Script {
        // Compile script
        let ast = match engine.0.compile(&script) {
            Ok(ast) => ast,
            Err(x) => panic!("AST: {:?}", x),
        };

        let scope = Scope::new();
        let state: Dynamic = Map::new().into();
        let mut script = Script { scope, state, ast };

        // Init the script
        let res = script.invoke::<()>("init", (), engine);

        match res {
            Ok(()) => (),
            Err(e) => println!("Script Error - init - {:?}", e),
        }

        script
    }

    pub fn invoke<T: Variant + Clone>(
        &mut self,
        name: &str,
        args: impl FuncArgs,
        engine: &Res<ScriptEngine>
    ) -> Result<T, Box<EvalAltResult>> {
        let options = CallFnOptions::new()
            .eval_ast(false)
            .bind_this_ptr(&mut self.state);

        engine.0.call_fn_with_options(
            options,
            &mut self.scope,
            &self.ast,
            &name,
            args,
        )
    }
}

#[derive(Resource)]
pub struct ScriptEngine(Engine);

impl ScriptEngine {
    pub fn new() -> ScriptEngine {
        let mut engine = Engine::new();

        // TODO: register a function that gets an angle from a vec2
        engine.register_type_with_name::<Vec2>("Vec2")
            .register_fn("new_vec2", |x: f32, y: f32| {
                Vec2::new(x as f32, y as f32)
            })
            .register_fn("to_string", |vec: &mut Vec2| vec.to_string())
            .register_fn("to_debug", |vec: &mut Vec2| format!("{vec:?}"))
            .register_get("x", |vec: &mut Vec2| vec.x as f32)
            .register_get("y", |vec: &mut Vec2| vec.y as f32)
            .register_fn("length", |vec: &mut Vec2| vec.length() as f32);

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
            // TODO: on_sensor
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
                        let res = script.invoke::<()>("on_collision", (), &engine);

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

            let trans = ship_query.get(entity).unwrap().3;

            let rot = trans.rotation;
            let tran = trans.translation;
            let vel = ship_query.get(entity).unwrap().0.velocity;

            // [ to_rot, to_vel ]
            let res = script.invoke::<rhai::Array>(
                "on_update",
                ( tran.truncate(), vel, rot.to_euler(EulerRot::ZYX).0 ),
                &engine
            );

            match res {
                Ok(data) => {
                    let to_rot: f32 = data[0].clone_cast();
                    if to_rot > f32::EPSILON {
                        // Is greater than zero, apply
                        let mut rotation = ship_query.get_mut(entity).unwrap().2;
                        rotation.target = rot * Quat::from_rotation_z(to_rot);
                    }

                    let to_accelerate: f32 = data[1].clone_cast();
                    if to_accelerate > f32::EPSILON {
                        // Is greater than zero, apply
                        let mut velocity = ship_query.get_mut(entity).unwrap().0;
                        velocity.acceleration = to_accelerate;
                    }
                },
                Err(e) => println!("Script Error - {:?}", e),
            }
        }
    }
}
