use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use std::sync::Arc;
use std::sync::Mutex;
use std::fmt;

use std::boxed::Box;
use std::collections::HashMap;
use rust_dynamic::value::Value;

use crate::ship::{
    movement::Velocity,
    rotation::TargetRotation,
    movement::Position,
    rotation::Rotation,
    radar::ContactEvent,
    radar::Radar,
};

use crate::math::AbsRot;
use crate::math::RelRot;

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
    state: Arc<Mutex<HashMap<&'static str, Value>>>,
    on_update: Box<dyn Fn(&mut HashMap<&'static str, Value>, IVec2, IVec2, AbsRot) -> (RelRot, i32, RelRot) + Send + Sync>,
    on_contact: Box<dyn Fn(&mut HashMap<&'static str, Value>, IVec2) + Send + Sync>,
    on_collision: Box<dyn Fn(&mut HashMap<&'static str, Value>) + Send + Sync>,
}

impl fmt::Debug for Script {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<Script>")
    }
}

impl Script {
    pub fn new<I, U, R, C>(on_init: I, on_update: U, on_contact: R, on_collision: C) -> Script
    where
        I: Fn() -> HashMap<&'static str, Value>,
        U: Fn(&mut HashMap<&'static str, Value>, IVec2, IVec2, AbsRot) -> (RelRot, i32, RelRot) + Send + Sync + 'static,
        R: Fn(&mut HashMap<&'static str, Value>, IVec2) + Send + Sync + 'static,
        C: Fn(&mut HashMap<&'static str, Value>) -> () + Send + Sync + 'static,
    {
        Script {
            state: Arc::new(Mutex::new(on_init())),
            on_update: Box::new(on_update),
            on_contact: Box::new(on_contact),
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
            .add_systems(Update, process_on_collision)
            .add_systems(Update, process_on_contact);
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
                    for script in [e1_script, e2_script] {
                        // Invoke collision handler
                        let state = script.state.clone();
                        let mut mut_state = state.lock().unwrap();
                        (script.on_collision)(&mut mut_state);
                    }
                } else {
                    println!("ERROR - SCRIPT - {:?}", collision_event);
                }
            },
            _ => (),
        }
    }
}

fn process_on_contact(
    mut contact_events: EventReader<ContactEvent>,
    mut query: Query<(Entity, &Position, &mut Script)>,
) {
    // Invoke the script for contact
    for contact_event in contact_events.read() {
        let ContactEvent(e1, e2) = contact_event;
        // TODO: right now with the ContactEvent being copies it leads to aliased query here,
        // This should be fixed once we have proper contact event that does not refer to self
        if let Ok([(_, _, e1_script), (_, e2_pos, _)]) = query.get_many_mut([*e1, *e2]) {
            // E1 knows where e2 is
            let state = e1_script.state.clone();
            let mut mut_state = state.lock().unwrap();
            (e1_script.on_contact)(&mut mut_state, e2_pos.0);
        } else {
            println!("ERROR - SCRIPT - {:?}", contact_event);
        }
    }
}

fn process_on_update(
    time: Res<Time>,
    mut timer: ResMut<ScriptTimer>,
    mut query: Query<(Entity, &mut Script)>,
    mut ship_query: Query<(
        &mut Velocity, &Position,
        &mut TargetRotation, &Rotation,
        &Children
    )>,
    mut radar_query: Query<&mut Radar>,
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
        for (entity, script) in query.iter_mut() {
            let ship = ship_query.get(entity).unwrap();

            let vel = ship.0;
            let pos = ship.1.0;
            let rot = ship.3.0;

            let state = script.state.clone();
            let mut mut_state = state.lock().unwrap();

            // [ to_rot, to_vel , to_rdr_rot ]
            let res = (script.on_update)(
                &mut mut_state,
                pos,
                vel.velocity,
                rot,
            );

            // Always apply
            let mut velocity = ship_query.get_mut(entity).unwrap().0;
            velocity.acceleration = res.1;

            let mut rotation = ship_query.get_mut(entity).unwrap().2;
            rotation.target += res.0;

            // Radar is on the children entity of the ship
            let children = ship_query.get(entity).unwrap().4;
            for child_entity in children {
                if let Ok(mut radar) = radar_query.get_mut(*child_entity) {
                    radar.target += res.2;
                }
            }
        }
    }
}
