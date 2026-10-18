use bevy::prelude::*;

pub mod field;
pub mod gate;
pub mod map;
pub mod ron;
pub mod ship;

pub use crate::assets::gate::AssetGatePlugin;
use crate::assets::gate::GateAppExt as _;

use crate::assets::field::FieldProfile;
use crate::assets::map::MapData;
use crate::assets::ron::RonAssetPlugin;
use crate::environ::Baseline;
use crate::environ::Milli;

use crate::ActiveMap;
use crate::assets::field::FieldGate;
use crate::assets::field::FieldSnapshot;
use crate::assets::field::detect_field_sources;
use crate::assets::field::update_snapshot;
use crate::assets::field::validate_field_profiles;

use crate::assets::map::MapGate;
use crate::assets::map::detect_star_map;
use crate::assets::map::sync_star_map;

use crate::assets::ship::ShipClass;
use crate::assets::ship::ShipGate;
use crate::assets::ship::apply_new_mounts;
use crate::assets::ship::cook_ship_class;
use crate::assets::ship::detect_ship_class;

use crate::FixedGameSystem;
use crate::spawner::process_spawn_message;

pub struct AssetsPlugin;
impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MapConsts>()
            .init_resource::<FieldSnapshot>()
            .init_asset::<FieldProfile>()
            .add_plugins((
                RonAssetPlugin::<MapData>::default(),
                RonAssetPlugin::<FieldProfile>::default(),
                RonAssetPlugin::<ShipClass>::default(),
            ))
            .add_gate(
                MapGate,
                detect_star_map.run_if(resource_exists::<ActiveMap>),
                sync_star_map,
            )
            .add_gate(
                FieldGate,
                detect_field_sources,
                (
                    update_snapshot,
                    validate_field_profiles.run_if(resource_changed::<FieldSnapshot>),
                )
                    .chain(),
            )
            .add_gate(ShipGate, detect_ship_class, cook_ship_class)
            // TODO: i feel like this may be better over in the spawner module
            .add_systems(
                FixedUpdate,
                apply_new_mounts
                    .after(process_spawn_message)
                    .in_set(FixedGameSystem::Spawn),
            );
    }
}

#[derive(Resource, Default, Clone, Copy)]
pub struct MapConsts {
    // Baseline value ie X amount of flux no matter where you are for eg
    pub baseline: Baseline,
    // Absolute map wide map c_speed cap
    pub c_max: Milli,
    // Due to despawn being a gradent curve, this is the
    // despawn cutoff - suggested to be 0
    pub despawn_cutoff: Milli,
}

impl From<&MapData> for MapConsts {
    fn from(map: &MapData) -> Self {
        Self {
            baseline: map.baseline,
            c_max: map.c_max,
            despawn_cutoff: map.despawn_cutoff,
        }
    }
}
