use bevy::prelude::*;

use bevy::asset::AssetLoader;
use bevy::asset::LoadContext;
use bevy::asset::io::Reader;

use std::marker::PhantomData;

use serde::Deserialize;
use thiserror::Error;

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
