use bevy::asset::UntypedAssetLoadFailedEvent;
use bevy::ecs::system::ScheduleSystem;
use bevy::prelude::*;

use std::any::TypeId;
use std::collections::HashMap;
use std::ops::Deref;
use std::ops::DerefMut;
use std::time::Duration;

use crate::GamePhase;

// Asset barrier. We want the sim to only ever see loaded + cooked assets all at once.  This will
// pause 1 or more frame whenever there's a hotload, but it will make sure that the load+cook
// completes all or none.
//
// Runs in 3 stage:
//  - Detect - notices that something changed (asset, resources, component, etc...)
//  - Decide - consult all of the gates, if any is not 'Ready' move GamePhase -> Load
//      and viceverse if all gate is 'Ready' move GamePhase -> Run
//  - Cook - does any processing that an reloaded asset/resource/etc needs.
//
// New gated assets:
//  - impl GateKey (gate name)
//  - app.add_gate(<GateKey>, detect, cook);
//
// Rule:
//  - gate only watch its own data, never its dependencies. (bevy will propgate for us)
//  - Only one cook system per gate.
//  - Cook must not trigger the Detect
pub trait GateKey: 'static + Send + Sync {
    const NAME: &'static str;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GateState {
    #[default]
    Pending,
    Ready,
    // Blocks till the failing asset is reloaded
    Failed,
}

struct GateSlot {
    name: &'static str,
    state: GateState,
}

#[derive(Resource, Default)]
pub struct AssetGate {
    slots: HashMap<TypeId, GateSlot>,
}

impl AssetGate {
    pub fn register<K: GateKey>(&mut self) {
        self.slots.entry(TypeId::of::<K>()).or_insert(GateSlot {
            name: K::NAME,
            state: GateState::Pending,
        });
    }

    // Prefer `AssetGateExt::set_gate` to avoid marking the resource as changed
    pub fn set<K: GateKey>(&mut self, state: GateState) {
        let slot = self.slots.entry(TypeId::of::<K>()).or_insert_with(|| {
            error!(
                "gate {} was never registered - use `app.add_gate(..)`",
                K::NAME
            );
            GateSlot {
                name: K::NAME,
                state: GateState::Pending,
            }
        });

        if slot.state != state {
            debug!("gate {} - {:?} -> {:?}", K::NAME, slot.state, state);
            slot.state = state;
        }
    }

    pub fn state<K: GateKey>(&self) -> GateState {
        self.slots
            .get(&TypeId::of::<K>())
            .map_or(GateState::Pending, |slot| slot.state)
    }

    pub fn is_pending<K: GateKey>(&self) -> bool {
        self.state::<K>() == GateState::Pending
    }

    pub fn all_ready(&self) -> bool {
        self.slots
            .values()
            .all(|slot| slot.state == GateState::Ready)
    }

    // Private api to uphold invariants
    fn registered_state<K: GateKey>(&self) -> Option<GateState> {
        // `None` if the gate was not registered (So we can warn about it)
        self.slots.get(&TypeId::of::<K>()).map(|slot| slot.state)
    }

    fn blocking(&self) -> Vec<(&'static str, GateState)> {
        // For logging what barrier is still unset to help debugging
        let mut blocking: Vec<_> = self
            .slots
            .values()
            .filter(|slot| slot.state != GateState::Ready)
            .map(|slot| (slot.name, slot.state))
            .collect();
        blocking.sort_unstable_by_key(|(name, _)| *name);
        blocking
    }
}

// Workaround bevy dirty detection.
//  - We want to permit systems to query/set a gate to a value every time
//  - We want the bevy dirty detection to go off iff the state actually changes.
pub trait AssetGateExt {
    fn set_gate<K: GateKey>(&mut self, state: GateState);
}

impl<T> AssetGateExt for T
where
    T: Deref<Target = AssetGate> + DerefMut,
{
    fn set_gate<K: GateKey>(&mut self, state: GateState) {
        // If the gate was not registered, fall through so it can be warned about
        if (**self).registered_state::<K>() == Some(state) {
            // TODO: can we nest the .set here?
            return;
        }
        // It was actually a new state, dirty the ref for bevy change detection to pick up on
        (**self).set::<K>(state);
    }
}

#[derive(SystemSet, Debug, Hash, Eq, PartialEq, Clone)]
pub enum GateSystems {
    Detect,
    Cook,
}

fn report_load_failures(mut failures: MessageReader<UntypedAssetLoadFailedEvent>) {
    for failure in failures.read() {
        error!("{} failed to load: {}", failure.path, failure.error);
    }
}

fn update_game_phase(
    gate: Res<AssetGate>,
    phase: Res<State<GamePhase>>,
    mut next: ResMut<NextState<GamePhase>>,
) {
    // Manages the GamePhase toggle between running/loading
    match (phase.get(), gate.all_ready()) {
        (GamePhase::Load, true) => {
            info!("all gates ready - Load -> Run");
            next.set(GamePhase::Run);
        }
        (GamePhase::Run, false) => {
            info!("gates {:?} went pending - Run -> Load", gate.blocking());
            next.set(GamePhase::Load);
        }
        _ => (),
    }
}

pub struct AssetGatePlugin;
impl Plugin for AssetGatePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AssetGate>()
            .add_systems(
                PreUpdate,
                (
                    report_load_failures.in_set(GateSystems::Detect),
                    update_game_phase.after(GateSystems::Detect),
                ),
            )
            .configure_sets(Update, GateSystems::Cook.run_if(in_state(GamePhase::Load)));
    }
}

pub trait GateAppExt {
    fn add_gate<K, D, C, DM, CM>(&mut self, key: K, detect: D, cook: C) -> &mut Self
    where
        K: GateKey,
        D: IntoScheduleConfigs<ScheduleSystem, DM>,
        C: IntoScheduleConfigs<ScheduleSystem, CM>;

    // Only register the key, the scheduling of the Detect + Cook is your problem
    fn register_gate_key<K: GateKey>(&mut self) -> &mut Self;
}

impl GateAppExt for App {
    fn add_gate<K, D, C, DM, CM>(&mut self, _key: K, detect: D, cook: C) -> &mut Self
    where
        K: GateKey,
        D: IntoScheduleConfigs<ScheduleSystem, DM>,
        C: IntoScheduleConfigs<ScheduleSystem, CM>,
    {
        self.register_gate_key::<K>()
            .add_systems(PreUpdate, detect.in_set(GateSystems::Detect))
            .add_systems(Update, cook.in_set(GateSystems::Cook))
    }

    fn register_gate_key<K: GateKey>(&mut self) -> &mut Self {
        if !self.is_plugin_added::<AssetGatePlugin>() {
            error!("Add AssetGatePlugin, otherwise this won't work!");
        }

        self.init_resource::<AssetGate>()
            .world_mut()
            .resource_mut::<AssetGate>()
            .register::<K>();
        self
    }
}
