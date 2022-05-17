mod convert_path;
mod fill;
mod stroke;

use fill::iterate_fill;
pub use glam;
use glam::{DMat4, DVec2, Vec2, Vec4};
use lyon::lyon_tessellation::{FillVertex, StrokeVertex, VertexBuffers};
use std::sync::Arc;
use stroke::iterate_stroke;
pub use usvg;
use usvg::{fontdb::Source, Path};

pub struct Callback<'a> {
    func: Box<dyn FnMut(&Path) -> bool + 'a>,
}

impl<'a> Callback<'a> {
    pub fn new(c: impl FnMut(&Path) -> bool + 'a) -> Self {
        Self { func: Box::new(c) }
    }
    fn process_events(&mut self, path: &Path) {
        (self.func)(path);
    }
}

pub type Vertices = Vec<Vertex>;
pub type Indices = Vec<Index>;
pub type DrawPrimitives = (Vertices, Indices);
pub type Size = Vec2;
pub type Position = Vec2;
pub type Rect = (Position, Size);

pub const FALLBACK_COLOR: Vec4 = Vec4::ONE;

struct TransformVariable {
    transform: DMat4,
    transform_index: u16,
}

struct GeometryVariable {
    vertices: Vertices,
    indices: Indices,
    index_base: u16,
}

pub fn init(mut callback: Callback) -> (DrawPrimitives, Rect) {
    // Parse and tessellate the geometry

    let mut opt = usvg::Options::default();
    let contents = include_bytes!("../fallback_font/Roboto-Medium.ttf");
    opt.fontdb
        .load_font_source(Source::Binary(Arc::new(contents.as_ref())));
    opt.font_family = "Roboto Medium".to_string();
    let rtree = usvg::Tree::from_data(include_bytes!("../../svg/life.svg"), &opt.to_ref()).unwrap();

    let view_box = rtree.svg_node().view_box;
    let rect: Rect = (
        Vec2::new(view_box.rect.x() as f32, view_box.rect.y() as f32),
        Vec2::new(view_box.rect.width() as f32, view_box.rect.height() as f32),
    );

    let mut geometry = VertexBuffers::<Vertex, Index>::new();
    for node in rtree.root().descendants() {
        if let usvg::NodeKind::Path(ref p) = *node.borrow() {
            callback.process_events(&p);

            if let Some(ref stroke) = p.stroke {
                let color = match stroke.paint {
                    usvg::Paint::Color(c) => Vec4::new(
                        c.red as f32 / u8::MAX as f32,
                        c.green as f32 / u8::MAX as f32,
                        c.blue as f32 / u8::MAX as f32,
                        stroke.opacity.value() as f32,
                    ),
                    _ => FALLBACK_COLOR,
                };
                iterate_stroke(stroke, p, &mut geometry, color);
            }
            if let Some(ref fill) = p.fill {
                let color = match fill.paint {
                    usvg::Paint::Color(c) => Vec4::new(
                        c.red as f32 / u8::MAX as f32,
                        c.green as f32 / u8::MAX as f32,
                        c.blue as f32 / u8::MAX as f32,
                        fill.opacity.value() as f32,
                    ),
                    _ => FALLBACK_COLOR,
                };

                iterate_fill(p, &color, &mut geometry);
            }
        }
    }
    ((geometry.vertices, geometry.indices), rect)
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub transform_matrix_index: u32,
    pub color: [f32; 4],
}
impl From<&DVec2> for Vertex {
    fn from(v: &DVec2) -> Self {
        Self {
            position: [(v.x) as f32, (v.y) as f32, 0.0],
            ..Default::default()
        }
    }
}
impl From<(&FillVertex<'_>, &Vec4)> for Vertex {
    fn from((v, c): (&FillVertex, &Vec4)) -> Self {
        Self {
            position: [v.position().x, v.position().y, 0.],
            color: c.to_array(),
            ..Default::default()
        }
    }
}
impl From<(&StrokeVertex<'_, '_>, &Vec4)> for Vertex {
    fn from((v, c): (&StrokeVertex, &Vec4)) -> Self {
        Self {
            position: [v.position().x, v.position().y, 0.],
            color: c.to_array(),
            ..Default::default()
        }
    }
}

impl From<(&DVec2, &Vec4)> for Vertex {
    fn from((v, c): (&DVec2, &Vec4)) -> Self {
        Self {
            position: [(v.x) as f32, (v.y) as f32, 0.0],
            color: [c.x, c.y, c.z, c.w],
            ..Default::default()
        }
    }
}

pub type Index = u32;
