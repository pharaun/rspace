use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

mod arena;
use crate::arena::ArenaPlugins;

mod script;
use crate::script::ScriptPlugins;

mod ship;
use crate::ship::ShipPlugins;

fn main() {
    App::new()
        .insert_resource(Msaa::default())
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(10.0))
        //.add_plugins(RapierDebugRenderPlugin::default())

        .add_plugins(ArenaPlugins)
        .add_plugins(ScriptPlugins)
        .add_plugins(ShipPlugins)

        .run();
}
