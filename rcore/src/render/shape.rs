use bevy_prototype_lyon::geometry::ShapeBuilderBase as _;
use bevy_prototype_lyon::prelude::Fill;
use bevy_prototype_lyon::prelude::Shape;
use bevy_prototype_lyon::prelude::ShapeBuilder;
use bevy_prototype_lyon::prelude::ShapePath;
use bevy_prototype_lyon::prelude::Stroke;

use bevy::prelude::Srgba;
use bevy::prelude::Vec2;

use crate::ship::ShipClass;

#[expect(clippy::needless_pass_by_value)]
pub(super) fn get_ship(class: ShipClass, fill: Srgba, stroke: Srgba) -> Shape {
    let ship_path = match class {
        ShipClass::Large => todo!(),
        ShipClass::Medium => ShapePath::new()
            .move_to(Vec2::new(0.0, 20.0))
            .line_to(Vec2::new(10.0, -20.0))
            .line_to(Vec2::new(0.0, -10.0))
            .line_to(Vec2::new(-10.0, -20.0))
            .close(),
        ShipClass::Small => todo!(),
        ShipClass::Tiny => todo!(),
    };

    ShapeBuilder::with(&ship_path)
        .fill(Fill::color(fill))
        .stroke(Stroke::new(stroke, 2.0))
        .build()
}

pub(super) fn get_radar(stroke: Srgba) -> Shape {
    let radar_path = ShapePath::new()
        .move_to(Vec2::new(5.0, 0.0))
        .arc(
            Vec2::new(0.0, 0.0),
            Vec2::new(5.0, 4.5),
            f32::to_radians(-180.0),
            f32::to_radians(0.0),
        )
        .move_to(Vec2::new(0.0, 2.0))
        .line_to(Vec2::new(0.0, -4.5));

    ShapeBuilder::with(&radar_path)
        .stroke(Stroke::new(stroke, 1.5))
        .build()
}
