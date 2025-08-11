use bevy::prelude::*;

use crate::FixedGameSystem;
use crate::ship::StarterShip;
use crate::ship::add_ship;

// This system is for taking care of spawning in new ships and entities as needed for the game
// - spawn in a missile when a ship fires one
// - spawn in a fighter/etc when a ship builds one

pub struct SpawnerPlugin;
impl Plugin for SpawnerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnEvent>()
            .add_systems(
                FixedUpdate,
                process_spawn_event.in_set(FixedGameSystem::Spawn),
            );
    }
}

// Ship Spawning event
// TODO: provide better way of handling scripts attached to ships, for now yoink script from parent
// spawner ship
//
// TODO: provide a way to spawn a ship based off a parent_ship and also based off world/scene
// loader
//
// TODO: we have weird frame timing, where the ship spawns then rotate quickly, and sometime the
// explosion doesn't work
#[derive(Event)]
pub struct SpawnEvent (pub StarterShip);

pub fn process_spawn_event(
    mut commands: Commands,
    mut spawn_event: EventReader<SpawnEvent>,
) {
    for SpawnEvent(ship) in spawn_event.read() {
        add_ship(&mut commands, ship.clone());
    }
}
