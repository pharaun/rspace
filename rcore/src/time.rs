use crate::GamePhase;
use bevy::prelude::*;
use bevy::state::state::StateTransitionSystems;

// Simulation speed manager:
// - Pause the simulation
// - ~16x slow down to 15x speedup
// - Single tick stepping.
//
// Send a message to this subsystem to change the timing parameters.
#[derive(Message, Clone, Copy, Debug)]
pub enum TimeMsg {
    Speed(i8),
    Pause(bool),
    Step(u32),
}

// Center game-tick counter
#[derive(Resource)]
pub struct Ticks(u64);

impl Ticks {
    pub fn elapsed(&self, start: u64) -> u64 {
        self.0.wrapping_sub(start)
    }

    pub fn is_ready(&self, start: u64, cooldown: u64) -> bool {
        self.elapsed(start) >= cooldown
    }
}

// Stepping backlog for mega-steps
#[derive(Resource)]
pub struct BacklogStep(u32);

pub struct TimeControlPlugin;
impl Plugin for TimeControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<TimeMsg>()
            .insert_resource(Ticks(0))
            .insert_resource(BacklogStep(0))
            // Must happen outside of FixedUpdate since it manages FixedUpdate timings
            .add_systems(
                PreUpdate,
                apply_time_command.run_if(in_state(GamePhase::Run)),
            )
            .add_systems(FixedFirst, apply_tick.run_if(in_state(GamePhase::Run)))
            // StateTransition - so pause happens before FixedUpdate but after AssetGate flips state.
            .add_systems(
                StateTransition,
                manage_sim_pause.in_set(StateTransitionSystems::TransitionSchedules),
            );
    }
}

fn apply_tick(mut ticks: ResMut<Ticks>) {
    // NOTE: Due to this being in FixedFirst it will only advance the ticks when
    // Time<Virtual> is not paused.
    ticks.0 = ticks.0.wrapping_add(1);
}

fn manage_sim_pause(
    mut transitions: MessageReader<StateTransitionEvent<GamePhase>>,
    mut time: ResMut<Time<Virtual>>,
    mut backlog: ResMut<BacklogStep>,
    mut player_paused: Local<bool>,
) {
    for transition in transitions.read() {
        match (&transition.exited, &transition.entered) {
            // So we don't override player_paused
            (Some(GamePhase::Load), Some(GamePhase::Load)) => (),
            // Remember if the sim was paused before or not
            (_, Some(GamePhase::Load)) => {
                *player_paused = time.is_paused();
                time.pause();
                backlog.0 = 0;
            }
            (Some(GamePhase::Load), Some(GamePhase::Run)) if !*player_paused => {
                time.unpause();
            }
            _ => (),
        }
    }
}

fn apply_time_command(
    mut message: MessageReader<TimeMsg>,
    fixed: Res<Time<Fixed>>,
    mut time: ResMut<Time<Virtual>>,
    mut backlog: ResMut<BacklogStep>,
) {
    let mut next_speed: Option<i8> = None;
    let mut pause: Option<bool> = None;

    // Read all message to enforce a last-write-wins semantics
    for &msg in message.read() {
        match msg {
            TimeMsg::Speed(s) => next_speed = Some(s),
            TimeMsg::Pause(b) => pause = Some(b),
            TimeMsg::Step(n) => backlog.0 = backlog.0.saturating_add(n),
        }
    }

    if let Some(s) = next_speed {
        time.set_relative_speed(2f32.powi(s.clamp(-4, 4).into()));
    }
    if let Some(p) = pause {
        if p {
            time.pause();
        } else {
            time.unpause();
        }
    }
    if backlog.0 > 0 {
        // We force time to be paused to single step here
        time.pause();

        // Start with 1024 step in a frame
        let run = (backlog.0).min(1024);
        time.advance_by(fixed.timestep() * run);
        backlog.0 -= run;
    }
}
