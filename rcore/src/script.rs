use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use std::boxed::Box;
use std::any::Any;

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
    on_update: Box<dyn Fn(Vec2, Vec2, f32) -> (f32, f32) + Send + Sync>,
    on_collision: Box<dyn Fn() + Send + Sync>,
}

impl Script {
    pub fn new<U, C>(on_update: U, on_collision: C) -> Script
    where
        U: Fn(Vec2, Vec2, f32) -> (f32, f32) + Send + Sync + 'static,
        C: Fn() -> () + Send + Sync + 'static,
    {
        Script {
            on_update: Box::new(on_update),
            on_collision: Box::new(on_collision),
        }
    }
}

#[derive(Resource)]
struct ScriptTimer(Timer);

pub struct ScriptPlugins;
impl Plugin for ScriptPlugins {
    fn build(&self, app: &mut App) {
        app.insert_resource(ScriptTimer(Timer::from_seconds(1.0 / 1.0, TimerMode::Repeating)))
            .add_systems(Update, process_on_update)
            // TODO: on_sensor
            .add_systems(Update, process_on_collision);
    }
}

fn process_on_collision(
    mut collision_events: EventReader<CollisionEvent>,
    mut query: Query<(Entity, &mut Script)>,
) {
    // Handle collision events first
    for collision_event in collision_events.read() {
        match collision_event {
            CollisionEvent::Started(e1, e2, _) => {
                if let Ok([(_, e1_script), (_, e2_script)]) = query.get_many_mut([*e1, *e2]) {
                    for mut script in [e1_script, e2_script] {
                        // Invoke collision handler
                        (script.on_collision)();
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
            let res = (script.on_update)(
                tran.truncate(),
                vel,
                rot.to_euler(EulerRot::ZYX).0,
            );

            let to_rot: f32 = res.0;
            if to_rot > f32::EPSILON {
                // Is greater than zero, apply
                let mut rotation = ship_query.get_mut(entity).unwrap().2;
                rotation.target = rot * Quat::from_rotation_z(to_rot);
            }

            let to_accelerate: f32 = res.1;
            if to_accelerate > f32::EPSILON {
                // Is greater than zero, apply
                let mut velocity = ship_query.get_mut(entity).unwrap().0;
                velocity.acceleration = to_accelerate;
            }
        }
    }
}
