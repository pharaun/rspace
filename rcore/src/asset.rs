use avian2d::prelude::*;
use bevy::prelude::*;

use bevy::asset::io::Reader;
use bevy::asset::AssetLoader;
use bevy::asset::LoadContext;
use bevy::asset::UntypedAssetId;
use bevy::asset::VisitAssetDependencies;

use std::marker::PhantomData;
use std::collections::HashMap;
use std::ops::Index;

use serde::Deserialize;
use thiserror::Error;

use crate::field::Milli;
use crate::field::Field;
use crate::FieldProfile;
use crate::field::Baseline;
use crate::field::MapConsts;
use crate::field::FieldSource;

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
        .add_systems(Startup, load_map)
        .add_systems(
            Update,
            star_setup.run_if(map_ready).run_if(run_once),
        );
    }
}


// Validation
fn load_map(
    mut commands: Commands,
    asset_server: Res<AssetServer>
) {
    commands.insert_resource(
        ActiveMap(asset_server.load("maps/level-0.map"))
    );
}

fn map_ready(
    map: Option<Res<ActiveMap>>,
    asset_server: Res<AssetServer>
) -> bool {
    map.is_some_and(|m| asset_server.is_loaded_with_dependencies(&m.0))
}

#[derive(Resource)]
pub struct ActiveMap(pub Handle<MapData>);

#[derive(Component)]
struct StarMarker;

// TODO: fix up ordering since we do need the star config up before ships gets created so they can
// have fresh environ data
pub fn star_setup(
    active: Res<ActiveMap>,
    maps: Res<Assets<MapData>>,
    profiles: Res<Assets<FieldProfile>>,
    mut data: ResMut<MapConsts>,
    mut commands: Commands,
) {
    let Some(map) = maps.get(&active.0) else {
        return;
    };

    // Update the map consts
    *data = MapConsts::from(map);

    // Iterate on the star field on the map data and instance them
    for star in &map.stars {
        let mut viz_star = commands.spawn((
            StarMarker,
            Transform::from_translation(star.pos.extend(0.)),
            FieldSource(star.profile().clone())
        ));

        #[cfg(feature = "render")]
        {
            let Some(prof) = profiles.get(star.profile()) else {
                return;
            };

            viz_star.insert(
                Sprite::from_color(prof.color, Vec2::splat(prof.body_radius))
            );
        }
    }
}
