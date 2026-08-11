use avian2d::prelude::*;
use bevy::prelude::*;

use bevy::asset::AssetLoader;
use bevy::asset::LoadContext;
use bevy::asset::UntypedAssetId;
use bevy::asset::VisitAssetDependencies;
use bevy::asset::io::Reader;

use std::collections::HashMap;
use std::marker::PhantomData;
use std::ops::Index;

use serde::Deserialize;
use thiserror::Error;

use crate::FieldProfile;
use crate::field::Baseline;
use crate::field::Field;
use crate::field::FieldSource;
use crate::field::MapConsts;
use crate::field::Milli;

// NOTE: Ron loader heavily inspired by: https://github.com/NiklasEi/bevy_common_assets
// Ron Asset Loader logic
pub trait RonAsset: Asset + for<'de> Deserialize<'de> {
    // Asset Extension (ie .map, .prf, ...)
    const EXTENSIONS: &'static [&'static str];
    // For ron files with nested dependencies, overload this
    // to force path fields into handles.
    fn resolve(&mut self, _ctx: &mut LoadContext<'_>) {}
}

#[derive(Debug, Error)]
pub enum RonLoaderError {
    #[error("failed to read asset: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to parse ron: {0}")]
    Ron(#[from] ron::de::SpannedError),
}

#[derive(TypePath)]
pub struct RonAssetLoader<A: RonAsset>(PhantomData<A>);

// Do the default impl manually so that we can avoid forcing Assets to implement Default
impl<A: RonAsset> Default for RonAssetLoader<A> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<A: RonAsset> AssetLoader for RonAssetLoader<A> {
    type Asset = A;
    type Settings = ();
    type Error = RonLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        ctx: &mut LoadContext<'_>,
    ) -> Result<A, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let mut asset: A = ron::de::from_bytes(&bytes)?;
        asset.resolve(ctx);
        Ok(asset)
    }

    fn extensions(&self) -> &[&str] {
        A::EXTENSIONS
    }
}

pub struct RonAssetPlugin<A: RonAsset>(PhantomData<A>);

// Do the default impl manually so that we can avoid forcing Assets to implement Default
impl<A: RonAsset> Default for RonAssetPlugin<A> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<A: RonAsset> Plugin for RonAssetPlugin<A> {
    fn build(&self, app: &mut App) {
        app.init_asset::<A>()
            .init_asset_loader::<RonAssetLoader<A>>();
    }
}

// --------------------------------------------------------------------------------

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

#[derive(Deserialize, Debug, Clone)]
pub struct StarData {
    pub pos: Vec2,
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

    fn resolve(&mut self, ctx: &mut LoadContext<'_>) {
        for star in &mut self.stars {
            // Load the star-class profile data (same class stars shares handles)
            star.profile = ctx.load(star.class.clone());
        }
    }
}

// TODO: unifiy this into fields.rs file
impl RonAsset for FieldProfile {
    const EXTENSIONS: &'static [&'static str] = &["prf"];
}

// TODO: lift out of this file too
impl From<&MapData> for MapConsts {
    fn from(map: &MapData) -> Self {
        Self {
            baseline: map.baseline.clone(),
            c_max: map.c_max,
            despawn_cutoff: map.despawn_cutoff,
        }
    }
}

pub struct AssetPlugin;
impl Plugin for AssetPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            RonAssetPlugin::<MapData>::default(),
            RonAssetPlugin::<FieldProfile>::default(),
        ))
        .add_systems(Update, sync_star_map.run_if(resource_exists::<ActiveMap>));
    }
}

#[derive(Resource)]
pub struct ActiveMap(pub Handle<MapData>);

#[derive(Component)]
pub struct StarMarker;

// TODO: fix up ordering since we do need the star config up before ships gets created so they can
// have fresh environ data
pub fn sync_star_map(
    asset_server: Res<AssetServer>,
    active: Res<ActiveMap>,
    maps: Res<Assets<MapData>>,
    profiles: Res<Assets<FieldProfile>>,
    stars: Query<Entity, With<StarMarker>>,
    mut events: MessageReader<AssetEvent<MapData>>,
    mut data: ResMut<MapConsts>,
    mut dirty: Local<bool>,
    mut commands: Commands,
) {
    // This block is for checking if map is ready to be loaded or reloaded
    let changed = events.read().fold(false, |acc, event| {
        acc || event.is_loaded_with_dependencies(&active.0) || event.is_modified(&active.0)
    });

    // This check is for updating active map to point to different file
    if changed || active.is_changed() {
        *dirty = true;
    }

    if !*dirty {
        return;
    }

    // Make sure that even if the map is loaded/updated, that we also load all of its
    // dependency...
    if !asset_server.is_loaded_with_dependencies(&active.0) {
        return;
    }

    let Some(map) = maps.get(&active.0) else {
        return;
    };

    *dirty = false;

    // Despawn the old stars
    for entity in &stars {
        commands.entity(entity).despawn();
    }

    // Update the map consts
    *data = MapConsts::from(map);

    // Iterate on the star field on the map data and instance them
    for star in &map.stars {
        let mut viz_star = commands.spawn((
            StarMarker,
            Transform::from_translation(star.pos.extend(0.)),
            FieldSource(star.profile().clone()),
        ));
    }
}
