use avian2d::prelude::*;
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::math::DVec2;
use bevy::prelude::*;

use serde::Deserialize;
use std::collections::HashMap;
use std::ops::Index;
use std::ops::IndexMut;

use crate::FixedGameSystem;
use crate::assets::MapConsts;
use crate::assets::field::FieldProfile;
use crate::assets::field::FieldSnapshot;

// Velocity per FP_MILI (v_cap = c/fp_milli * cref)
const C_REF: f32 = 1.0;
const FP_MILLI: i32 = 1000;
pub type Milli = i32;

pub struct EnvironPlugin;
impl Plugin for EnvironPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, sample_environ.in_set(FixedGameSystem::Sampler));
    }
}

// Enum key for each table (table types)
macro_rules! define_fields {
    ($($variant:ident),+ $(,)?) => {
        #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Deserialize)]
        pub enum Field {
            $($variant),+
        }

        impl Field {
            pub const COUNT: usize = [$(Field::$variant),+].len();
            pub const ALL: [Field; Self::COUNT] = [$(Field::$variant),+];
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
pub struct CurveTable {
    // Strictly increasing radii
    points: Vec<Knot>,
    tail: Tail,
}

impl CurveTable {
    #[expect(clippy::cast_possible_truncation)]
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

// This converts all of the HashMap lookup into array-index via the
// enforcement of Field ordering/config/etc via the Field Macro above.
pub type Baseline = PerField<Milli>;
pub type FieldTables = PerField<CurveTable>;
pub type EnvironField = PerField<Milli>;

#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[serde(
    from = "HashMap<Field, T>",
    bound(deserialize = "T: Deserialize<'de> + Default + Clone")
)]
pub struct PerField<T>([T; Field::COUNT]);

impl<T: Default + Clone> From<HashMap<Field, T>> for PerField<T> {
    fn from(m: HashMap<Field, T>) -> Self {
        Self(std::array::from_fn(|i| {
            // TODO: swap to remove to move the vec?
            m.get(&Field::ALL[i]).cloned().unwrap_or_default()
        }))
    }
}

impl<T> Index<Field> for PerField<T> {
    type Output = T;
    fn index(&self, field: Field) -> &T {
        &self.0[field as usize]
    }
}

impl<T> IndexMut<Field> for PerField<T> {
    fn index_mut(&mut self, field: Field) -> &mut T {
        &mut self.0[field as usize]
    }
}

impl<T> From<[T; Field::COUNT]> for PerField<T> {
    fn from(a: [T; Field::COUNT]) -> Self {
        Self(a)
    }
}

impl<T> PerField<T> {
    pub const fn new(a: [T; Field::COUNT]) -> Self {
        Self(a)
    }

    pub fn iter(&self) -> impl Iterator<Item = (Field, &T)> {
        Field::ALL.into_iter().zip(self.0.iter())
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Field, &mut T)> {
        Field::ALL.into_iter().zip(self.0.iter_mut())
    }
}

// TODO: Update it to use the PerField too
#[derive(Component, Default, Debug)]
#[require(Position)]
#[component(on_add = on_environ_add)]
pub struct Environ {
    raw: EnvironField,
    // TODO: Nearest entity?
    pub nearest_source: f64,
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
        compose(&snapshot.table, consts, pos)
    };

    // At this point we should have everything needed to compose a updated environ
    let Some(mut environ) = world.get_mut::<Environ>(context.entity) else {
        return;
    };
    *environ = new_environ;
}

// Vec2 (for now cuz stars are probs going to be close to origin?)
// DVec2 for ships
fn compose(sources: &[(Vec2, FieldProfile)], consts: &MapConsts, p: DVec2) -> Environ {
    let mut raw = consts.baseline;
    let mut nearest_source = f64::INFINITY;

    for (pos, profile) in sources {
        let r = pos.as_dvec2().distance(p);
        nearest_source = nearest_source.min(r - profile.body_radius as f64);

        // Iterate through all of the fields for each source
        for field in Field::ALL {
            // TODO: fix this to accept f64 for distance
            let v = profile.tables[field].sample(r as f32);
            raw[field] = raw[field].saturating_add(v);
        }
    }

    // post compose mapping:
    // - CBoost is not a linear sum
    raw[Field::CBoost] = raw[Field::CBoost].min(consts.c_max);

    Environ {
        raw,
        nearest_source,
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
