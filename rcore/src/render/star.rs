use bevy::prelude::*;

use crate::FieldProfile;
use crate::FieldSource;

pub(super) struct StarPlugin;
impl Plugin for StarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, sync_star_visuals);
    }
}

// We re-sync the star visuals if:
//  - Changed<FieldSource> - the star got respawned
//  - AssetChanged<FieldSource> - the prf was reloaded (update the visual to match new prf)
#[expect(clippy::type_complexity)]
fn sync_star_visuals(
    stars: Query<(Entity, &FieldSource), Or<(Changed<FieldSource>, AssetChanged<FieldSource>)>>,
    profiles: Res<Assets<FieldProfile>>,
    mut commands: Commands,
) {
    for (entity, field_source) in &stars {
        let Some(prof) = profiles.get(&field_source.0) else {
            continue;
        };

        // Insert will replace the Sprite already on the star with a new one if it already had one
        commands.entity(entity).insert(Sprite::from_color(
            prof.color,
            Vec2::splat(prof.body_radius * 2.0),
        ));
    }
}
