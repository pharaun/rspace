use bevy::prelude::*;

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

pub struct TimeControlPlugin;
impl Plugin for TimeControlPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_message::<TimeMsg>()
            .insert_resource(Ticks(0))
            // Must happen outside of FixedUpdate since it manages FixedUpdate timings
            .add_systems(PreUpdate, apply_time_command)
            .add_systems(
                FixedFirst,
                |mut ticks: ResMut<Ticks>| {
                    ticks.0 = ticks.0.wrapping_add(1);
                }
            );
    }
}

fn apply_time_command(
    mut message: MessageReader<TimeMsg>,
    fixed: Res<Time<Fixed>>,
    mut time: ResMut<Time<Virtual>>,
    mut backlog: Local<u32>,
) {
    let mut next_speed: Option<i8> = None;
    let mut pause: Option<bool> = None;

    // Read all message to enforce a last-write-wins semantics
    for &msg in message.read() {
        match msg {
            TimeMsg::Speed(s) => next_speed = Some(s),
            TimeMsg::Pause(b) => pause = Some(b),
            TimeMsg::Step(n)  => *backlog = backlog.saturating_add(n),
        }
    }

    if let Some(s) = next_speed {
        time.set_relative_speed(
            2f32.powi(s.clamp(-4, 4).into())
        );
    }
    if let Some(p) = pause {
        if p {
            time.pause();
        } else {
            time.unpause();
        }
    }
    if *backlog > 0 {
        // We force time to be paused to single step here
        time.pause();

        // Start with 1024 step in a frame
        let run = (*backlog).min(1024);
        time.advance_by(fixed.timestep() * run);
        *backlog -= run;
    }
}
