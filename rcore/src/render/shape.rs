use bevy::asset::LoadContext;
use bevy::asset::RenderAssetUsages;
use bevy::color::ColorToComponents as _;
use bevy::mesh::Indices;
use bevy::mesh::PrimitiveTopology;
use bevy::prelude::*;

use lyon_tessellation::{
    BuffersBuilder, FillOptions, FillTessellator, FillVertex,
    FillVertexConstructor, StrokeOptions, StrokeTessellator,
    StrokeVertex, StrokeVertexConstructor, TessellationError,
    VertexBuffers,
};
use lyon_tessellation::geom::{Angle, Box2D};
use lyon_tessellation::math::{Point, point, vector};
use lyon_tessellation::path::{Path, Polygon, Winding};

use serde::Deserialize;
use std::collections::BTreeMap;
use thiserror::Error;

// The actual ship meshes for the render pipeline
#[derive(Debug, Default)]
pub struct ShipMeshes {
    pub hull: Option<Handle<Mesh>>,
    pub mounts: BTreeMap<String, Handle<Mesh>>,
}

// The data out of the ShipClass asset -> tessellated at load time and distributed to ships that
// uses the same base ship class.
#[derive(Deserialize, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct ShipVisual {
    #[serde(default)]
    pub hull: Vec<Layer>,
    // Each mount will have a unique name so that we can map attributes to each mount (maybe could
    // call it mount type? but for now name)
    #[serde(default)]
    pub mounts: BTreeMap<String, Vec<Layer>>,
}

impl ShipVisual {
    // Tessellate a ship visual down to an concrete ShipMeshes (hull + mounts (Ie radar))
    pub fn build(&self, ctx: &mut LoadContext<'_>) -> Result<ShipMeshes, ShapeError> {
        let hull = tessellate("hull", &self.hull)?.map(|mesh| ctx.add_labeled_asset("hull", mesh));
        let mut mounts = BTreeMap::new();
        for (name, layers) in &self.mounts {
            if let Some(mesh) = tessellate(name, layers)? {
                mounts.insert(name.clone(), ctx.add_labeled_asset(format!("mount/{name}"), mesh));
            }
        }
        Ok(ShipMeshes { hull, mounts })
    }
}

// Layers to permit differing type of stroke/detail on a ship, for eg we can have one layer
// for the outline, 1 layer for the details, and 1 layer for like the faction/group/fleet it is
// associated with.
#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Layer {
    pub shape: Shape,
    #[serde(default)]
    pub fill: Option<Paint>,
    #[serde(default)]
    pub stroke: Option<StrokeStyle>,
}

// This is mostly a wrapper/somewhat a copy from bevy prototype lyon idea of having a list of
// "shape" that you then run through the tessellator to get out a mesh.
#[derive(Deserialize, Debug)]
pub enum Shape {
    Path(Vec<PathCmd>),
    // Always closed
    Polygon(Vec<Vec2>),
    Circle { center: Vec2, radius: f32 },
    Rect { min: Vec2, max: Vec2 },
}

impl Shape {
    fn to_path(&self) -> Path {
        match self {
            Self::Path(cmds) => {
                let mut path = Path::builder().with_svg();

                for cmd in cmds {
                    match *cmd {
                        PathCmd::MoveTo(to) => { path.move_to(to.to_point()); },
                        PathCmd::LineTo(to) => { path.line_to(to.to_point()); },
                        PathCmd::QuadTo { ctrl, to } => { path.quadratic_bezier_to(
                            ctrl.to_point(),
                            to.to_point()
                        ); },
                        PathCmd::CubicTo { ctrl1, ctrl2, to } => { path.cubic_bezier_to(
                            ctrl1.to_point(),
                            ctrl2.to_point(),
                            to.to_point()
                        ); },
                        PathCmd::Arc { center, radii, sweep, rotation } => path.arc(
                            center.to_point(),
                            vector(radii.x, radii.y),
                            Angle::degrees(sweep),
                            Angle::degrees(rotation),
                        ),
                        PathCmd::Close => path.close(),
                    }
                }
                path.build()
            },
            Self::Polygon(points) => {
                let points: Vec<Point> = points.iter().copied().map(|p| p.to_point()).collect();
                let mut path = Path::builder();
                path.add_polygon(Polygon { points: &points, closed: true });
                path.build()
            },
            Self::Circle { center, radius } => {
                let mut path = Path::builder();
                path.add_circle(center.to_point(), *radius, Winding::Positive);
                path.build()
            },
            Self::Rect { min, max } => {
                let mut path = Path::builder();
                path.add_rectangle(&Box2D::new(min.to_point(), max.to_point()), Winding::Positive);
                path.build()
            },
        }
    }
}

// Extension trait to add an conversion for vec2->point so the to_path code feels better
trait Vec2Ext {
    fn to_point(&self) -> Point;
}

impl Vec2Ext for Vec2 {
    #[inline]
    fn to_point(&self) -> Point {
        point(self.x, self.y)
    }
}

// Like said above with Shape, this maps to lyon's svg path builder
#[derive(Deserialize, Debug)]
pub enum PathCmd {
    MoveTo(Vec2),
    LineTo(Vec2),
    QuadTo {
        ctrl: Vec2,
        to: Vec2,
    },
    CubicTo {
        ctrl1: Vec2,
        ctrl2: Vec2,
        to: Vec2,
    },
    // TODO: Angles are in degrees (do we want something else here?)
    Arc {
        center: Vec2,
        radii: Vec2,
        sweep: f32,
        #[serde(default)]
        rotation: f32,
    },
    Close,
}

// TODO: Might want to swap it into something like:
// - Role (ie fleet allegance)
// - Variant (ie hull/trim/accent/etc) - ie different variant of a ship class
#[derive(Deserialize, Debug, Clone, Copy)]
pub enum Paint {
    Rgba(f32, f32, f32, f32),
}

impl Paint {
    fn to_vertex_color(self) -> [f32; 4] {
        match self {
            Self::Rgba(r, g, b, a) => LinearRgba::from(Srgba::new(r, g, b, a)).to_f32_array(),
        }
    }
}

// TODO: can we use Vec4 here instead of a fixed array?
#[derive(Clone, Copy)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 4],
}

struct PaintVertex([f32; 4]);

impl FillVertexConstructor<Vertex> for PaintVertex {
    fn new_vertex(&mut self, vertex: FillVertex) -> Vertex {
        Vertex {
            position: vertex.position().to_array(),
            color: self.0,
        }
    }
}

impl StrokeVertexConstructor<Vertex> for PaintVertex {
    fn new_vertex(&mut self, vertex: StrokeVertex) -> Vertex {
        Vertex {
            position: vertex.position().to_array(),
            color: self.0,
        }
    }
}

#[derive(Deserialize, Debug, Clone, Copy)]
#[serde(deny_unknown_fields)]
pub struct StrokeStyle {
    pub paint: Paint,
    pub width: f32,
}

#[derive(Debug, Error)]
#[error("failed to tessellate {part} layer {layer}: {source}")]
pub struct ShapeError {
    part: String,
    layer: usize,
    #[source]
    source: TessellationError,
}

// A ship can have more than 1 layer for different stroke/color/paths, this will
// collect it all into a single mesh.
fn tessellate(part: &str, layers: &[Layer]) -> Result<Option<Mesh>, ShapeError> {
    let mut buffers: VertexBuffers<Vertex, u32> = VertexBuffers::new();

    let mut fill_tess = FillTessellator::new();
    let mut stroke_tess = StrokeTessellator::new();

    for (index, layer) in layers.iter().enumerate() {
        let path = layer.shape.to_path();

        if let Some(paint) = layer.fill {
            fill_tess.tessellate_path(
                &path,
                &FillOptions::default(),
                &mut BuffersBuilder::new(&mut buffers, PaintVertex(paint.to_vertex_color())),
            ).map_err(|e| ShapeError { part: part.to_owned(), layer: index, source: e })?;
        }
        if let Some(stroke) = layer.stroke {
            stroke_tess.tessellate_path(
                &path,
                &StrokeOptions::default().with_line_width(stroke.width),
                &mut BuffersBuilder::new(&mut buffers, PaintVertex(stroke.paint.to_vertex_color())),
            ).map_err(|e| ShapeError { part: part.to_owned(), layer: index, source: e })?;
        }
    }

    if buffers.indices.is_empty() {
        return Ok(None);
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_indices(Indices::U32(buffers.indices));
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        buffers
            .vertices
            .iter()
            .map(|v| [v.position[0], v.position[1], 0.0])
            .collect::<Vec<[f32; 3]>>(),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_COLOR,
        buffers.vertices.iter().map(|v| v.color).collect::<Vec<_>>(),
    );
    Ok(Some(mesh))
}
