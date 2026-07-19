use bevy_prototype_lyon::geometry::ShapeBuilderBase as _;
use bevy_prototype_lyon::prelude::Fill;
use bevy_prototype_lyon::prelude::Shape;
use bevy_prototype_lyon::prelude::ShapeBuilder;
use bevy_prototype_lyon::prelude::ShapePath;
use bevy_prototype_lyon::prelude::Stroke;

use bevy::prelude::Srgba;
use bevy::prelude::Vec2;

use crate::ship::ShipClass;

pub(super) fn get_ship(class: ShipClass, fill: Srgba, stroke: Srgba) -> Shape {
    let ship_path = match class {
        ShipClass::Large => todo!(),
        ShipClass::Medium => ShapePath::new()
            .move_to(Vec2::new(0.0, 200.0))
            .line_to(Vec2::new(100.0, -200.0))
            .line_to(Vec2::new(0.0, -100.0))
            .line_to(Vec2::new(-100.0, -200.0))
            .close(),
        ShipClass::Small => todo!(),
        ShipClass::Tiny => todo!(),
    };

    ShapeBuilder::with(&ship_path)
        .fill(Fill::color(fill))
        .stroke(Stroke::new(stroke, 20.0))
        .build()
}

pub(super) fn get_radar(stroke: Srgba) -> Shape {
    let radar_path = ShapePath::new()
        .move_to(Vec2::new(50.0, 0.0))
        .arc(
            Vec2::new(0.0, 0.0),
            Vec2::new(50.0, 45.0),
            f32::to_radians(-180.0),
            f32::to_radians(0.0),
        )
        .move_to(Vec2::new(0.0, 20.0))
        .line_to(Vec2::new(0.0, -45.0));

    ShapeBuilder::with(&radar_path)
        .stroke(Stroke::new(stroke, 15.0))
        .build()
}
