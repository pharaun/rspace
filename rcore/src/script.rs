use bevy::prelude::*;
use avian2d::prelude::*;

use dyn_clone::DynClone;

use std::fmt;

use crate::movement::Velocity;
use crate::movement::Position;
use crate::rotation::TargetRotation;
use crate::rotation::Rotation;

use crate::radar::ContactEvent;
use crate::radar::Radar;
use crate::radar::Arc as CompArc;
use crate::weapon::FireDebugWeaponEvent;
use crate::weapon::FireDebugWarheadEvent;
use crate::weapon::FireDebugMissileEvent;

use crate::FixedGameSystem;
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

// This is the ship status object that gives the current status of the ship
pub struct ShipStatus {
    pub position: IVec2,
    pub velocity: IVec2,
    pub acceleration: i32,
    pub heading: AbsRot
}

// Initial attempt of building a ship action structure for what to do
pub struct ShipAction {
    pub heading: RelRot,
    pub acceleration: i32,
    pub radar_heading: RelRot,
    pub target_entity: Option<Entity>
}

impl ShipAction {
    pub fn new() -> ShipAction {
        ShipAction {
            heading: RelRot(0),
            acceleration: 0,
            radar_heading: RelRot(0),
            target_entity: None
        }
    }

    pub fn heading(mut self, hdr: RelRot) -> ShipAction {
        self.heading = hdr;
        self
    }

    pub fn acceleration(mut self, acc: i32) -> ShipAction {
        self.acceleration = acc;
        self
    }

    pub fn radar_heading(mut self, hdr: RelRot) -> ShipAction {
        self.radar_heading = hdr;
        self
    }

    pub fn target_entity(mut self, target: Option<Entity>) -> ShipAction {
        self.target_entity = target;
        self
    }
}

pub trait ShipScript: DynClone + Send + Sync + 'static {
    fn on_update(
        &mut self,
        status: &ShipStatus,
    ) -> ShipAction;

    // TODO: add ship status to these as well
    fn on_contact(&mut self, target_pos: IVec2, target_entity: Entity);
    fn on_collision(&mut self);
}
dyn_clone::clone_trait_object!(ShipScript);

#[derive(Component, Clone)]
pub struct Script {
    pub script: Box<dyn ShipScript>
}

impl fmt::Debug for Script {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<Script>")
    }
}

#[derive(Resource)]
struct ScriptTimer(Timer);

pub struct ScriptPlugins;
impl Plugin for ScriptPlugins {
    fn build(&self, app: &mut App) {
        app.insert_resource(ScriptTimer(Timer::from_seconds(1.0 / 1.0, TimerMode::Repeating)))
            .add_systems(FixedUpdate, (
                process_on_update.in_set(FixedGameSystem::ShipLogic),
                process_on_collision.in_set(FixedGameSystem::ShipLogic),
                process_on_contact.in_set(FixedGameSystem::ShipLogic),
            ));
    }
}

fn process_on_collision(
    mut collision_events: EventReader<CollisionStarted>,
    mut query: Query<(Entity, &mut Script)>,
) {
    // Handle collision events first
    for CollisionStarted(e1, e2) in collision_events.read() {
        if let Ok([(_, e1_script), (_, e2_script)]) = query.get_many_mut([*e1, *e2]) {
            for mut ship_script in [e1_script, e2_script] {
                // Invoke collision handler
                ship_script.script.on_collision();
            }
        } else {
            println!("ERROR - SCRIPT - CollisionStarted({:?}, {:?})", e1, e2);
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
        if let Ok([(_, _, mut e1_script), (e2_entity, e2_pos, _)]) = query.get_many_mut([*e1, *e2]) {
            // E1 knows where e2 is
            e1_script.script.on_contact(e2_pos.0, e2_entity);
        } else {
            println!("ERROR - SCRIPT - {:?}", contact_event);
        }
    }
}

// TODO: add emitting the fire-debug-weapon-event code here, need
// to figure out how to find the target entity and stuff, maybe update the
// on contact code to query+give the entity id for now for the target so
// that its easier to just 'fire' the weapon.
//
// TODO: consider maybe some sort of like ship-status cache on each ship (maybe part of scripts)
// that gets updated with relevant information in all sort of systems then we can
// query for it and pass it right into the script rather than having a ton of extra
// queries here.
//
// TODO: find a better way to handle ship updates post script since we may need to have many many
// events and queries to update all sort of entities, this is tightly coupled. Maybe have a
// per frame ShipAction event that gets sent out to all sort of subsystem and they check
// if its relevant, and if so they update theirselves, otherwise they skip, this may
// be a better way since it would let us to self-contain the logic into each area's systems?
fn process_on_update(
    time: Res<Time>,
    mut timer: ResMut<ScriptTimer>,
    mut query: Query<(Entity, &mut Script)>,
    mut ship_query: Query<(
        &mut Velocity, &Position,
        &mut TargetRotation, &Rotation,
        &Children
    )>,
    target_query: Query<Entity>,
    mut radar_query: Query<&mut CompArc, With<Radar>>,
    mut events: EventWriter<FireDebugWeaponEvent>,
    mut w_events: EventWriter<FireDebugWarheadEvent>,
    mut m_events: EventWriter<FireDebugMissileEvent>,
) {
    // handle normal on_update ticks
    if timer.0.tick(time.delta()).just_finished() {
        for (entity, mut ship_script) in query.iter_mut() {
            let ship = ship_query.get(entity).unwrap();

            let ship_status = ShipStatus {
                position: ship.1.0,
                velocity: ship.0.velocity,
                acceleration: ship.0.acceleration,
                heading: ship.3.0,
            };

            let res = ship_script.script.on_update(&ship_status);

            // Always apply
            let mut velocity = ship_query.get_mut(entity).unwrap().0;
            velocity.acceleration = res.acceleration;

            let mut rotation = ship_query.get_mut(entity).unwrap().2;
            rotation.target += res.heading;

            // Radar is on the children entity of the ship
            let children = ship_query.get(entity).unwrap().4;
            for child_entity in children {
                if let Ok(mut radar) = radar_query.get_mut(*child_entity) {
                    radar.target += res.radar_heading;
                }
            }

            // For now emit a fire event
            if let Some(target) = res.target_entity {
                if let Ok(target_entity) = target_query.get(target) {
                    events.write(FireDebugWeaponEvent(entity, target_entity));
                }
            }

            // For now spam the warhead fire event
            if let Some(_) = res.target_entity {
                w_events.write(FireDebugWarheadEvent(entity));
                m_events.write(FireDebugMissileEvent(entity));
            }
        }
    }
}
