use avian2d::prelude::*;
use bevy::prelude::*;
use std::collections::HashMap;

use crate::FixedGameSystem;

pub struct StarPlugin;
impl Plugin for StarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (sample_environ,).chain().in_set(FixedGameSystem::Sampler),
        )
        .add_systems(Startup, star_setup);
    }
}

// Velocity per FP_MILI (v_cap = c/fp_milli * cref)
const C_REF: f32 = 1.0;
const FP_MILLI: i32 = 1000;
type Milli = i32;

// Enum key for each table (table types)
macro_rules! define_fields {
    ($($variant:ident),+ $(,)?) => {
        #[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
        enum Field {
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
    // TODO:
    // - Field mostly to permit "metaball behavior" for defining the boundary.
    // - Not sure if we want a field or if a strict boundary is better?
    Despawn,
    // "Stellar Flux" aka power/matter a starlifter station can raise from the star (Wellspring)
    Flux,
    // Local speed cap (closer you are to a star the higher your speed cap is).
    // All sources must be budgeted so that the CBoost merged together doesn't clamp the max c
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

#[derive(Clone, Debug, Default, Reflect)]
struct Knot {
    r: f32,
    v: Milli,
}

// Behavior past the last knot
#[derive(Clone, Debug, Default, Reflect)]
enum Tail {
    // Clamped to last value
    #[default]
    Clamp,
    // Clamped to zero
    Zero,
    // Inverse square law (i = 1 / d²)
    InvSquare,
}

#[derive(Clone, Debug, Default, Reflect)]
struct RadialTable {
    // Strictly increasing radii
    points: Vec<Knot>,
    tail: Tail,
}

impl RadialTable {
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

//#[serde(from = "HashMap<Field, RadialTable>")]
#[derive(Reflect)]
struct FieldTables([RadialTable; Field::COUNT]);

impl From<HashMap<Field, RadialTable>> for FieldTables {
    fn from(m: HashMap<Field, RadialTable>) -> Self {
        FieldTables(std::array::from_fn(|i| {
            m.get(&Field::ALL[i]).cloned().unwrap_or_default()
        }))
    }
}

// Loaded set of value to represent an spectral class star
// - G class star
// - Dark star? (Despawn+CBoost only for eg)
#[derive(Asset, Reflect)]
struct FieldProfile {
    body_radius: f32,
    flux: Milli,
    tables: FieldTables,
}

// TODO: Do we use transform or Position here, since
// we can move Position to f64...
#[derive(Component)]
#[require(Transform)]
struct FieldSource(Handle<FieldProfile>);

// TODO: map-wide consts
#[derive(Resource)]
struct MapConsts {}

#[derive(Component, Default)]
pub struct Environ {
    raw: [Milli; Field::COUNT],
    pub nearest_source_radius: f32,
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

fn compose(source: &[(Vec2, &FieldProfile)], consts: &MapConsts, p: Vec2) -> Environ {
    Environ::default()
}

fn sample_environ(
    source: Query<(&Transform, &FieldSource)>,
    profiles: Res<Assets<FieldProfile>>,
    consts: Res<MapConsts>,
    mut ships: Query<(&Position, &mut Environ)>,
) {
    // Collect a snapshot of the source + their profiles
    let snapshot: Vec<(Vec2, &FieldProfile)> = source
        .iter()
        .filter_map(|(tran, s)| profiles.get(&s.0).map(|p| (tran.translation.truncate(), p)))
        .collect();

    // Compute the environ for everything that needs it
    ships.par_iter_mut().for_each(|(pos, mut env)| {
        *env = compose(&snapshot, &consts, pos.0);
    });
}

fn star_setup(mut commands: Commands) {}
