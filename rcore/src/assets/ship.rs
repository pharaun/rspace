use bevy::asset::AsAssetId;
use bevy::asset::LoadContext;
use bevy::asset::LoadState;
use bevy::prelude::*;

use serde::Deserialize;
use std::collections::BTreeMap;

use crate::assets::gate::AssetGate;
use crate::assets::gate::AssetGateExt as _;
use crate::assets::gate::GateKey;
use crate::assets::gate::GateState;
use crate::assets::ron::ResolveError;
use crate::assets::ron::RonAsset;
use crate::attach::AttachOffset;
use crate::attach::AttachedTo;
use crate::ship::ShipType;

#[cfg(feature = "render")]
use crate::render::shape::ShipMeshes;
#[cfg(feature = "render")]
use crate::render::shape::ShipVisual;

pub struct ShipGate;
impl GateKey for ShipGate {
    const NAME: &'static str = "ship-class";
}

// This is a slightly awkward asset type, it contains both:
// - Sim data (ie component/hull/drive/module loadout)
// - Visual data (paths/layers/etc)
#[derive(Asset, TypePath, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ShipClass {
    pub class: ShipType,

    #[serde(default)]
    pub mounts: BTreeMap<String, MountData>,

    #[cfg(feature = "render")]
    #[serde(default)]
    pub visual: ShipVisual,

    #[cfg(not(feature = "render"))]
    #[serde(default)]
    #[expect(dead_code, reason = "validates the visual syntax and discard")]
    visual: serde::de::IgnoredAny,

    // When we resolve this asset, we will tessellate it into meshes
    #[cfg(feature = "render")]
    #[serde(skip)]
    pub meshes: ShipMeshes,
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct MountData {
    // Ship-local offset
    pub pos: IVec2,
}

impl RonAsset for ShipClass {
    const EXTENSIONS: &'static [&'static str] = &["shp"];

    #[cfg(feature = "render")]
    fn resolve(&mut self, ctx: &mut LoadContext<'_>) -> Result<(), ResolveError> {
        self.meshes = self.visual.build(ctx)?;
        Ok(())
    }

    #[cfg(not(feature = "render"))]
    fn resolve(&mut self, _ctx: &mut LoadContext<'_>) -> Result<(), ResolveError> {
        Ok(())
    }
}

// TODO: This is a patch to have a default ship class visual and layout wise. Once we have
// a more robust module/builder system this might go away to be replaced with a script + map
// builder that builds the ship.
#[derive(Resource)]
pub struct DefaultShipClass(pub Handle<ShipClass>);

#[derive(Component, Clone)]
pub struct Class(pub Handle<ShipClass>);

// This lets us use AssetChanged<Class>
impl AsAssetId for Class {
    type Asset = ShipClass;

    fn as_asset_id(&self) -> AssetId<ShipClass> {
        self.0.id()
    }
}

// Name of the mount (so that we can find them when we need to hot reload changes to the visual +
// config of a ship class file)
#[derive(Component, Clone)]
pub struct Mount(pub String);

// Always render ship mounts slightly above the hull
// TODO: move this to the render when the attachment system gets reworked
const MOUNT_Z: f32 = 1.0;

// For more detail on this (see `detect_star_map`)
pub fn detect_ship_class(
    mut events: MessageReader<AssetEvent<ShipClass>>,
    mut gate: ResMut<AssetGate>,
) {
    let recovering = gate.state::<ShipGate>() == GateState::Failed;

    let mut dirty = false;
    for event in events.read() {
        dirty |= matches!(event, AssetEvent::Modified { .. })
            || (recovering && matches!(event, AssetEvent::Added { .. }));
    }

    if dirty {
        gate.set_gate::<ShipGate>(GateState::Pending);
    }
}

pub fn cook_ship_class(
    asset_server: Res<AssetServer>,
    default: Option<Res<DefaultShipClass>>,
    ship_class: Res<Assets<ShipClass>>,
    ships: Query<&Class>,
    mut mounts: Query<(&Mount, &AttachedTo, &mut AttachOffset)>,
    mut gate: ResMut<AssetGate>,
) {
    if !gate.is_pending::<ShipGate>() {
        return;
    }

    // Make sure the default class + all used designs are loaded
    let in_use = ships.iter().map(|class| &class.0);
    for handle in default.iter().map(|default| &default.0).chain(in_use) {
        match asset_server.load_state(handle.id()) {
            LoadState::Loaded => (),
            LoadState::Failed(err) => {
                error!("ship class failed to load: {err}");
                gate.set_gate::<ShipGate>(GateState::Failed);
                return;
            }
            _ => return,
        }
    }

    // TODO: this is the spot where we update the ship sim component states/settings when we
    // rebuild the ShipBuilder to enable us to setup ship-class component/config in the assets
    for (mount, attached, mut offset) in &mut mounts {
        sync_mount_offset(&ship_class, &ships, mount, attached, &mut offset);
    }
    gate.set_gate::<ShipGate>(GateState::Ready);
}

// Cook will take care of most, this is for stuff that cook missed
pub fn apply_new_mounts(
    ship_class: Res<Assets<ShipClass>>,
    parents: Query<&Class>,
    mut mounts: Query<(&Mount, &AttachedTo, &mut AttachOffset), Added<Mount>>,
) {
    for (mount, attached, mut offset) in &mut mounts {
        sync_mount_offset(&ship_class, &parents, mount, attached, &mut offset);
    }
}

fn sync_mount_offset(
    ship_class: &Assets<ShipClass>,
    parents: &Query<&Class>,
    mount: &Mount,
    attached: &AttachedTo,
    offset: &mut AttachOffset,
) {
    let Some(class) = parents
        .get(attached.0)
        .ok()
        .and_then(|class| ship_class.get(&class.0))
    else {
        return;
    };

    // AttachOffset will move around with the ship, however it won't rotate when the ship rotate
    // so that radar or other attachment can stay in the same direction as set by the player's
    // script
    if let Some(data) = class.mounts.get(&mount.0) {
        offset.0 = data.pos.as_vec2().extend(MOUNT_Z);
    } else {
        warn!("ship design has no mount named {:?}", mount.0);
    }
}
