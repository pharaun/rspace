use avian2d::prelude::*;
use bevy::prelude::*;
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy_common_assets::ron::RonAssetPlugin;

use serde::Deserialize;
use std::collections::HashMap;

use crate::FixedGameSystem;

pub struct StarPlugin;
impl Plugin for StarPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(RonAssetPlugin::<FieldProfile>::new(&["ron"]))
            .init_resource::<MapConsts>()
            .init_asset::<FieldProfile>()
            .add_systems(
                FixedUpdate,
                (sample_environ,).chain().in_set(FixedGameSystem::Sampler),
            );
        // TODO: temp
        app.world_mut().register_component::<FieldSource>();
    }
}

#[derive(Component)]
struct StarMarker;

pub fn star_setup(
    mut profiles: ResMut<Assets<FieldProfile>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    // TODO: Workaround for now
    // This works - cuz we register the component (above) and then we forcefully load the assets
    // So this works to prove the hook logic works, but the hook logic probs should be done in a
    // better way
    let raw = std::fs::read_to_string("assets/profiles/g_class.ron").expect("g_class.ron missing");
    let profile: FieldProfile = ron::from_str(&raw).expect("g_class.ron malformed");

    commands.spawn((
        StarMarker,
        Transform::from_xyz(0.0, -1000.0, 0.0),
        //FieldSource(asset_server.load("profiles/g_class.ron")),
        FieldSource(profiles.add(profile)),
        // This breaks the render separation
        Sprite::from_color(Color::srgb(0.0, 1.0, 1.0), Vec2::splat(200.0)),
    ));
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

#[derive(Clone, Debug, Default, Reflect, Deserialize)]
struct Knot {
    r: f32,
    v: Milli,
}

// Behavior past the last knot
#[derive(Clone, Debug, Default, Reflect, Deserialize)]
enum Tail {
    // Clamped to last value
    #[default]
    Clamp,
    // Clamped to zero
    Zero,
    // Inverse square law (i = 1 / d²)
    InvSquare,
}

#[derive(Clone, Debug, Default, Reflect, Deserialize)]
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

#[derive(Reflect, Deserialize, Debug)]
#[serde(from = "HashMap<Field, RadialTable>")]
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
#[derive(Asset, Reflect, Deserialize, Debug)]
pub  struct FieldProfile {
    body_radius: f32,
    flux: Milli,
    tables: FieldTables,
}

// TODO: Do we use transform or Position here, since
// we can move Position to f64...
#[derive(Component)]
#[require(Transform)]
struct FieldSource(Handle<FieldProfile>);

#[derive(Resource, Default)]
struct MapConsts {
    // Baseline value ie X amount of flux no matter where you are for eg
    baseline: [Milli; Field::COUNT],
    // Absolute map wide map c_speed cap
    c_max: Milli,
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
        let Some(profiles) = world.get_resource::<Assets<FieldProfile>>() else {
            return;
        };
        let Some(consts) = world.get_resource::<MapConsts>() else {
            return;
        };
        let mut query = world.try_query::<(&Transform, &FieldSource)>().unwrap();
        let snapshot: Vec<(Vec2, &FieldProfile)> = query.iter(&world)
            .filter_map(|(tran, s)| profiles.get(&s.0).map(|p| (tran.translation.truncate(), p)))
            .collect();

        compose(&snapshot, &consts, pos)
    };

    // At this point we should have everything needed to compose a updated environ
    let Some(mut environ) = world.get_mut::<Environ>(context.entity) else {
        return;
    };
    *environ = new_environ;
}

fn compose(sources: &[(Vec2, &FieldProfile)], consts: &MapConsts, p: Vec2) -> Environ {
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
