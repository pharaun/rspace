use avian2d::prelude::Position;
use bevy::asset::LoadContext;
use bevy::asset::LoadState;
use bevy::asset::UntypedAssetId;
use bevy::asset::VisitAssetDependencies;
use bevy::math::DVec2;
use bevy::prelude::*;

use serde::Deserialize;

use crate::assets::MapConsts;
use crate::assets::field::FieldProfile;
use crate::assets::field::FieldSource;
use crate::assets::gate::AssetGate;
use crate::assets::gate::AssetGateExt as _;
use crate::assets::gate::GateKey;
use crate::assets::gate::GateState;
use crate::assets::ron::ResolveError;
use crate::assets::ron::RonAsset;
use crate::environ::Baseline;
use crate::environ::Milli;

pub struct MapGate;
impl GateKey for MapGate {
    const NAME: &'static str = "star-map";
}

#[derive(TypePath, Deserialize, Debug, Clone)]
pub struct MapData {
    // Absolute map-wide max c_speed cap
    pub c_max: Milli,
    // Due to despawn being a gradent curve, this is the
    // despawn cutoff - suggested to be 0
    pub despawn_cutoff: Milli,
    // Baseline value ie X amount of flux no matter where you are for eg
    pub baseline: Baseline,
    // Locations of the stars on the map + profiles
    pub stars: Vec<StarData>,
}

// Can't implement a VisitAssetDependencies for Vec<StarData> so handroll
impl VisitAssetDependencies for MapData {
    fn visit_dependencies(&self, visit: &mut impl FnMut(UntypedAssetId)) {
        for star in &self.stars {
            visit(star.profile.id().untyped());
        }
    }
}

impl Asset for MapData {}

impl RonAsset for MapData {
    const EXTENSIONS: &'static [&'static str] = &["map"];

    fn resolve(&mut self, ctx: &mut LoadContext<'_>) -> Result<(), ResolveError> {
        for star in &mut self.stars {
            // Load the star-class profile data (same class stars shares handles)
            star.profile = ctx.load(star.class.clone());
        }
        Ok(())
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct StarData {
    pub pos: DVec2,
    // Path to the FieldProfile asset
    class: String,
    // `MapData::resolve` lets MapData resolve the handle into this struct
    #[serde(skip)]
    profile: Handle<FieldProfile>,
}

// TODO: not sure on this part
impl StarData {
    pub fn profile(&self) -> &Handle<FieldProfile> {
        &self.profile
    }
    pub fn profile_path(&self) -> &str {
        &self.class
    }
}

#[derive(Resource)]
pub struct ActiveMap(pub Handle<MapData>);

#[derive(Component)]
pub struct StarMarker;

// As per the rules in gate, this must only detect map-data changes only
// - We currently also delete+respawn the stars, so that triggers field_source detector too
pub fn detect_star_map(
    active: Res<ActiveMap>,
    mut events: MessageReader<AssetEvent<MapData>>,
    mut gate: ResMut<AssetGate>,
) {
    // Must always drain the MessageReader, this avoids a re-trigger
    let (added, modified) = {
        let mut added = false;
        let mut modified = false;

        for event in events.read() {
            // Only check for the map handle id
            added |= event.is_added(&active.0);
            modified |= event.is_modified(&active.0);
        }
        (added, modified)
    };

    // Skip the Added check unless we are recovering from a failed state.
    // - Cuz Added is fired in PostUpdate, and by then (Cook is in Update),
    //   which causes us to re-enter cook a second time spurriously
    let recovering = gate.state::<MapGate>() == GateState::Failed;

    // is_changed() handles ActiveMap being pointed at a new file
    if modified || (added && recovering) || active.is_changed() {
        gate.set_gate::<MapGate>(GateState::Pending);
    }
}

pub fn sync_star_map(
    asset_server: Res<AssetServer>,
    active: Option<Res<ActiveMap>>,
    maps: Res<Assets<MapData>>,
    stars: Query<Entity, With<StarMarker>>,
    mut data: ResMut<MapConsts>,
    mut gate: ResMut<AssetGate>,
    mut commands: Commands,
) {
    // Only recook when detect decides its time to recook
    if !gate.is_pending::<MapGate>() {
        return;
    }

    // No map is also valid (means there's no stars on the map for eg)
    let Some(active) = active else {
        gate.set_gate::<MapGate>(GateState::Ready);
        return;
    };

    if let LoadState::Failed(err) = asset_server.load_state(active.0.id().untyped()) {
        error!("map failed to load: {err}");
        gate.set_gate::<MapGate>(GateState::Failed);
        return;
    }

    // The map data also points to .prf and etc, make sure those are loaded too
    // TODO: do we even need this, this technically violates the single responsibility rule for
    // gates.
    if !asset_server.is_loaded_with_dependencies(&active.0) {
        return;
    }

    let Some(map) = maps.get(&active.0) else {
        return;
    };

    // Despawn the old stars
    // TODO: probs better to move it to a system that maps the star/update them, but for... 10?
    // stars, eh its fine for now
    for entity in &stars {
        commands.entity(entity).despawn();
    }

    // Update the map consts
    *data = MapConsts::from(map);

    // Iterate on the star field on the map data and instance them
    for star in &map.stars {
        commands.spawn((
            StarMarker,
            Position(star.pos),
            FieldSource(star.profile().clone()),
        ));
    }
    gate.set_gate::<MapGate>(GateState::Ready);
}
