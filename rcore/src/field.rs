use avian2d::prelude::*;
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;

use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;

use crate::FixedGameSystem;

pub struct FieldPlugin;
impl Plugin for FieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RonAssetPlugin::<FieldProfile>::new(&["ron"]))
            .init_resource::<MapConsts>()
            .init_resource::<FieldSnapshot>()
            .init_asset::<FieldProfile>()
            .add_systems(FixedUpdate, validate_field_profiles)
            .add_systems(
                FixedUpdate,
                (update_snapshot.run_if(field_sources_dirty), sample_environ)
                    .chain()
                    .in_set(FixedGameSystem::Sampler),
            );
    }
}

// Velocity per FP_MILI (v_cap = c/fp_milli * cref)
const C_REF: f32 = 1.0;
const FP_MILLI: i32 = 1000;
type Milli = i32;

// Enum key for each table (table types)
macro_rules! define_fields {
    ($($variant:ident),+ $(,)?) => {
        #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Deserialize)]
        pub enum Field {
            $($variant),+
        }

        impl Field {
            const COUNT: usize = [$(Field::$variant),+].len();
            const ALL: [Field; Self::COUNT] = [$(Field::$variant),+];
        }
    };
}
define_fields!(
    // This is for despawn & arena boundary (Presence)
    // - Field to permit for metaball despawning (joining 2 or more stars)
    // - Field to permit a fade/warning ie <50 that its about to despawn
    // - Field to adjust the despawn distance outside the game area, before despawn have c_boost
    //   approach zero so that there's little incentive to be out there.
    Despawn,
    // "Stellar Flux" aka power/matter a starlifter station can raise from the star (Wellspring)
    Flux,
    // Local speed cap (closer you are to a star the higher your speed cap is).
    // All sources must be budgeted so that the CBoost merged together doesn't clamp the max c
    //  - Clamp is a safety/map/engine defined max where things start to go wonky,
    //
    // Example:
    //  - 2 stars with 40% of c_max at half separation.
    //  - Corridor between both runs at 80% c_max,
    //  - Then there's a high speed c_boost zone without the risk-trade off of heat of being close
    //  to a star
    //
    // Budget: c_max / (1 + overlap) - check midpoint value to make sure its not too high
    CBoost,
    // Heat - Ships need to be able to cool down, discourage ships lottering too close to a star
    Heat,
    // Brightness - Aka how easy it is to observe a ship close to the star
    Luminance,
    // Radar noise - The noise from the star
    Noise,
    // Simulates the "veil" aka how dense an gas cloud/asteroid belt is to permit ships to hide
    // behind them. (OpticalDepth)
    VeilDensity,
);

// Its better to space the knots out where it "matters" aka around steep parts of an curve, ie for
// eg: if you sample it linearly with a 1/r² ish curve, it will waste most of the knot on flat
// tail, and make the steep part of the curve... way too rough.
//
// Better to sample it where the value steps by a "roughly constant" fraction: r[i+1] = r[i] * k
// and k ~= 1.3
//
// This allow you to have like 16 log-spaced knots instead of 60 linear ones.
#[derive(Clone, Debug, Default, Deserialize)]
struct Knot {
    r: f32,
    v: Milli,
}

// Behavior past the last knot
#[derive(Clone, Debug, Default, Deserialize)]
enum Tail {
    // Clamped to last value
    #[default]
    Clamp,
    // Clamped to zero
    Zero,
    // Inverse square law (i = 1 / d²)
    InvSquare,
}

// TODO: add a phanton type to this so that we can separate it into several domain:
// - Radius
// - Field value
// - Velocity Excess - cherknov
//
// This will permit reusing this table + sampler logic in other domains.
#[derive(Clone, Debug, Default, Deserialize)]
struct CurveTable {
    // Strictly increasing radii
    points: Vec<Knot>,
    tail: Tail,
}

impl CurveTable {
    pub fn sample(&self, radius: f32) -> Milli {
        debug_assert!(!radius.is_nan(), "NaN radius");
        if radius.is_nan() {
            return 0;
        }
        // This permits the radial table to be empty and sample to zero.
        if self.points.is_empty() {
            return 0;
        }
        // This permits the first value to be defined at any radii and not
        // be strictly required to be at 0.
        if radius <= self.points[0].r {
            return self.points[0].v;
        }
        // If we are past the last knot radii... then the result depends on
        // what is set in tail.
        let last_idx = self.points.len() - 1;
        if radius >= self.points[last_idx].r {
            return match self.tail {
                Tail::Clamp => self.points[last_idx].v,
                Tail::Zero => 0,
                Tail::InvSquare => {
                    let ratio = f64::from(self.points[last_idx].r / radius);
                    (f64::from(self.points[last_idx].v) * ratio.powi(2)).round() as Milli
                }
            };
        }
        // For low number of knot points a linear scan is fast enough
        let idx = self
            .points
            .windows(2)
            .position(|p| radius < p[1].r)
            .expect("radius has been verified to be within the knot range");
        // Linear regression
        let ratio = f64::from(
            (radius - self.points[idx].r) / (self.points[idx + 1].r - self.points[idx].r),
        );
        let (a, b) = (
            f64::from(self.points[idx].v),
            f64::from(self.points[idx + 1].v),
        );
        (a + (b - a) * ratio).round() as Milli
    }
}

#[derive(Deserialize, Debug)]
#[serde(from = "HashMap<Field, CurveTable>")]
struct FieldTables([CurveTable; Field::COUNT]);

impl From<HashMap<Field, CurveTable>> for FieldTables {
    fn from(m: HashMap<Field, CurveTable>) -> Self {
        FieldTables(std::array::from_fn(|i| {
            m.get(&Field::ALL[i]).cloned().unwrap_or_default()
        }))
    }
}

// Loaded set of value to represent an spectral class star
// - G class star
// - Dark star? (Despawn+CBoost only for eg)
#[derive(Asset, TypePath, Deserialize, Debug, Clone)]
pub struct FieldProfile {
    body_radius: f32,
    flux: Milli,
    tables: Arc<FieldTables>,
}

// TODO: Do we use transform or Position here, since
// we can move Position to f64...
#[derive(Component)]
#[require(Transform)]
pub struct FieldSource(pub Handle<FieldProfile>);

#[derive(Resource, Default)]
struct MapConsts {
    // Baseline value ie X amount of flux no matter where you are for eg
    baseline: [Milli; Field::COUNT],
    // Absolute map wide map c_speed cap
    c_max: Milli,
    // Due to despawn being a gradent curve, this is the
    // despawn cutoff - suggested to be 0
    despawn_cutoff: Milli,
}

// This resource updates on change of any of the handle, it rebuilds
#[derive(Resource, Default)]
struct FieldSnapshot {
    table: Vec<(Vec2, FieldProfile)>,
}

#[derive(Component, Default, Debug)]
#[require(Position)]
#[component(on_add = on_environ_add)]
pub struct Environ {
    raw: [Milli; Field::COUNT],
    // TODO: Nearest entity?
    pub nearest_source: f32,
}

impl Environ {
    pub fn get(&self, f: Field) -> Milli {
        self.raw[f as usize]
    }
    // TODO: other fields
    pub fn c(&self) -> Milli {
        self.get(Field::CBoost)
    }
}

fn on_environ_add(mut world: DeferredWorld, context: HookContext) {
    let new_environ = {
        let Some(&Position(pos)) = world.get::<Position>(context.entity) else {
            return;
        };
        let Some(consts) = world.get_resource::<MapConsts>() else {
            return;
        };
        let Some(snapshot) = world.get_resource::<FieldSnapshot>() else {
            return;
        };
        compose(&snapshot.table, &consts, pos)
    };

    // At this point we should have everything needed to compose a updated environ
    let Some(mut environ) = world.get_mut::<Environ>(context.entity) else {
        return;
    };
    *environ = new_environ;
}

fn compose(sources: &[(Vec2, FieldProfile)], consts: &MapConsts, p: Vec2) -> Environ {
    let mut raw = consts.baseline;
    let mut nearest_source = f32::INFINITY;

    for (pos, profile) in sources {
        let r = pos.distance(p);
        nearest_source = nearest_source.min(r - profile.body_radius);

        // Iterate through all of the fields for each source
        for i in 0..Field::COUNT {
            let v = profile.tables.0[i].sample(r);
            raw[i] = raw[i].saturating_add(v);
        }
    }

    // post compose mapping:
    // - CBoost is not a linear sum
    let c = &mut raw[Field::CBoost as usize];
    *c = (*c).min(consts.c_max);

    Environ {
        raw,
        nearest_source,
    }
}

fn validate_field_profiles(
    mut asset_message: MessageReader<AssetEvent<FieldProfile>>,
    profiles: Res<Assets<FieldProfile>>,
    consts: Res<MapConsts>,
) {
    for msg in asset_message.read() {
        let (AssetEvent::Added { id } | AssetEvent::Modified { id }) = msg else {
            continue;
        };
        let Some(profile) = profiles.get(*id) else {
            continue;
        };

        // TODO: move this to asset loading or other spots, not every single frame do we need to
        // validate this.
        // Double check that the despawn table last knot is below the despawn_cutoff
        let last = profile.tables.0[Field::Despawn as usize].sample(f32::INFINITY);
        if last > consts.despawn_cutoff {
            // It will never despawn, print log warning
            error!(
                "despawn table never falls below cutoff: {last} > {}",
                consts.despawn_cutoff
            );
        }
    }
}

type FieldDirty = (
    With<FieldSource>,
    Or<(Changed<Transform>, Changed<FieldSource>)>,
);

fn field_sources_dirty(
    dirty: Query<(), FieldDirty>,
    mut removed: RemovedComponents<FieldSource>,
    mut asset_message: MessageReader<AssetEvent<FieldProfile>>,
) -> bool {
    // Check if any of these ran (that means that something changed and we need to rebuild)
    let any_removed = removed.read().count() > 0;
    let any_asset = asset_message.read().count() > 0;
    any_removed || any_asset || !dirty.is_empty()
}

fn update_snapshot(
    sources: Query<(&Transform, &FieldSource)>,
    profiles: Res<Assets<FieldProfile>>,
    mut snapshot: ResMut<FieldSnapshot>,
) {
    snapshot.table.clear();

    for (tran, src) in &sources {
        let Some(profile) = profiles.get(&src.0) else {
            continue;
        };
        snapshot
            .table
            .push((tran.translation.truncate(), profile.clone()));
    }
}

fn sample_environ(
    consts: Res<MapConsts>,
    snapshot: Res<FieldSnapshot>,
    mut ships: Query<(&Position, &mut Environ)>,
) {
    // Compute the environ for everything that needs it
    ships.par_iter_mut().for_each(|(pos, mut env)| {
        *env = compose(&snapshot.table, &consts, pos.0);
    });
}
