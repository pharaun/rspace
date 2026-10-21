use avian2d::physics_transform::ApplyPosToTransform;
use avian2d::prelude::Position;
use avian2d::prelude::Rotation;
use bevy::asset::AsAssetId;
use bevy::math::DVec2;
use bevy::prelude::*;

use serde::Deserialize;
use std::sync::Arc;

use crate::assets::MapConsts;
use crate::assets::gate::AssetGate;
use crate::assets::gate::AssetGateExt as _;
use crate::assets::gate::GateKey;
use crate::assets::gate::GateState;
use crate::assets::ron::RonAsset;
use crate::environ::Field;
use crate::environ::FieldTables;
use crate::environ::Milli;

pub struct FieldGate;
impl GateKey for FieldGate {
    const NAME: &'static str = "field-snapshot";
}

// Loaded set of value to represent an spectral class star
// - G class star
// - Dark star? (Despawn+CBoost only for eg)
#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
pub struct FieldProfile {
    pub body_radius: f32,
    pub flux: Milli,
    #[cfg(feature = "render")]
    pub color: Srgba,
    pub tables: Arc<FieldTables>,
}

impl RonAsset for FieldProfile {
    const EXTENSIONS: &'static [&'static str] = &["prf"];
}

// ApplyPosToTransform is required to get Avian to writeback to a transform
#[derive(Component)]
#[require(Position, Rotation, ApplyPosToTransform)]
pub struct FieldSource(pub Handle<FieldProfile>);

// This lets us use AssetChanged<FieldSource>
impl AsAssetId for FieldSource {
    type Asset = FieldProfile;

    fn as_asset_id(&self) -> AssetId<FieldProfile> {
        self.0.id()
    }
}

// This resource updates on change of any of the handle, it rebuilds
#[derive(Resource, Default)]
pub struct FieldSnapshot {
    pub table: Vec<(DVec2, FieldProfile)>,
}

pub fn validate_field_profiles(snapshot: Res<FieldSnapshot>, consts: Res<MapConsts>) {
    for (_, profile) in &snapshot.table {
        let last = profile.tables[Field::Despawn].sample(f32::INFINITY);

        // Double check that the despawn table last knot is below the despawn_cutoff
        if last > consts.despawn_cutoff {
            error!(
                "despawn table never falls below cutoff: {last} > {}",
                consts.despawn_cutoff
            );
        }
    }
}

// Force it to recook whenever one of these following item changes:
// - Changed<Position>      star moved
// - Changed<FieldSource>   star spawned, or repointed at a different profile
// - AssetChanged<..>       the profile behind the handle hot reloaded
//
// WARNING: `Changed<Position>` assumes static FieldSource, if we add this to
// anything that moves, it will break the sim+gate badly (ie flip-flop). So if we ever
// setup mobile FieldSources, we need a live-sampler at this point cuz the cache isn't really
// helping us.
type FieldDirty = (
    With<FieldSource>,
    Or<(
        Changed<Position>,
        Changed<FieldSource>,
        AssetChanged<FieldSource>,
    )>,
);

pub fn detect_field_sources(
    dirty: Query<(), FieldDirty>,
    mut removed: RemovedComponents<FieldSource>,
    mut gate: ResMut<AssetGate>,
) {
    let dirty = !dirty.is_empty();
    let removed = removed.read().count() > 0;

    if dirty || removed {
        gate.set_gate::<FieldGate>(GateState::Pending);
    }
}

pub fn update_snapshot(
    sources: Query<(&Position, &FieldSource)>,
    profiles: Res<Assets<FieldProfile>>,
    mut snapshot: ResMut<FieldSnapshot>,
    mut gate: ResMut<AssetGate>,
) {
    if !gate.is_pending::<FieldGate>() {
        return;
    }
    let mut table = Vec::with_capacity(sources.iter().len());

    for (pos, src) in &sources {
        // Commit once we make it through this loop
        let Some(profile) = profiles.get(&src.0) else {
            return;
        };
        table.push((pos.0, profile.clone()));
    }

    snapshot.table = table;
    gate.set_gate::<FieldGate>(GateState::Ready);
}
