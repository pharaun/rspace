use bevy_prototype_lyon::geometry::ShapeBuilderBase;
use bevy_prototype_lyon::prelude::Fill;
use bevy_prototype_lyon::prelude::Shape;
use bevy_prototype_lyon::prelude::ShapeBuilder;
use bevy_prototype_lyon::prelude::ShapePath;
use bevy_prototype_lyon::prelude::Stroke;

use bevy::prelude::Vec2;

// There are several classes of ship:
// 1. cruiser - large
// 2. frigate - medium
// 3. fighter - small
// 4. missiles/mines - tiny
//
// but we can probs represent this idea with something that is like
// ship size, then you load out a customized list of component on it
// to produce a whole ship, so ie a tiny-class loaded with a warhead
// and radar would be a missile for example.
//
// This file would mostly serve a way to provide a render for the various
// class of ship, and then we can feed it into the base mod to yield a 'ship'
pub enum ShipClass {
    Large,
    Medium,
    Small,
    Tiny,
}

pub fn get_ship(class: ShipClass, fill: Fill, stroke: Stroke) -> Shape {
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

    ShapeBuilder::with(&ship_path).fill(fill).stroke(stroke).build()
}

pub fn get_radar(stroke: Stroke) -> Shape {
    let radar_path = ShapePath::new()
        .move_to(Vec2::new(5.0, 0.0))
        .arc(Vec2::new(0.0, 0.0), Vec2::new(5.0, 4.5), f32::to_radians(-180.0), f32::to_radians(0.0))
        .move_to(Vec2::new(0.0, 2.0))
        .line_to(Vec2::new(0.0, -4.5));

    ShapeBuilder::with(&radar_path).stroke(stroke).build()
}
