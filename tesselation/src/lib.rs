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
use usvg::{fontdb::Source, Node, NodeKind, Path};

#[derive(PartialEq, PartialOrd, Eq, Ord, Clone, Copy)]
pub enum Priority {
    Static,
    DynamicVertex,
    DynamicIndex, // if index is dynamic, vertex is always dynamic
}

pub struct Callback<'a> {
    func: Box<dyn FnMut(&Node) -> Priority + 'a>,
}

impl<'a> Callback<'a> {
    pub fn new(c: impl FnMut(&Node) -> Priority + 'a) -> Self {
        Self { func: Box::new(c) }
    }
    fn process_events(&mut self, node: &Node) -> Priority {
        (self.func)(node)
    }
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

#[derive(Clone, Default, Debug)]
struct GeometryVariable {
    id: Option<String>,
    vertices: Vertices,
    indices: Indices,
    index_base: usize,
    transform_index: usize,
}
#[derive(Clone, Default, Debug)]
struct GeometryVariables(Vec<GeometryVariable>);
impl GeometryVariables {
    fn get_vertices_len(&mut self) -> usize {
        self.0
            .iter()
            .fold(0, |acc, curr| acc + curr.get_vertices_len())
    }
    fn get_vertices(&mut self) -> Vertices {
        self.0.iter().flat_map(|v| v.get_v()).collect()
    }
    fn get_indices_with_offset(&mut self, offset: u32) -> Vec<u32> {
        self.0
            .iter()
            .flat_map(|v| v.get_i().iter().map(|i| i + offset).collect::<Vec<_>>())
            .collect()
    }
    fn new() -> Self {
        Self(vec![])
    }
}

impl GeometryVariable {
    fn get_vertices_len(&self) -> usize {
        self.vertices.len()
    }
    fn get_v(&self) -> Vertices {
        self.vertices.clone()
    }
    fn get_i(&self) -> Indices {
        self.indices
            .iter()
            .map(|index| index + self.index_base as u32)
            .collect()
    }
}

impl GeometryVariable {
    fn new(v: VertexBuffers<Vertex, Index>, index_base: usize, id: Option<String>) -> Self {
        Self {
            id,
            vertices: v.vertices,
            indices: v.indices,
            index_base,
            ..Default::default()
        }
    }
}
fn recursive(node: Node, priority: Priority, callback: &mut Callback) {
    if let usvg::NodeKind::Path(ref p) = *node.borrow() {
        for child in node.children() {
            let priority = priority.max(callback.process_events(&node));
            recursive(child, priority, callback);
        }
    }
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

    let mut statics: GeometryVariables = GeometryVariables::new();
    let mut dynamic_vertices: GeometryVariables = GeometryVariables::new();
    let mut dynamic_indices: GeometryVariables = GeometryVariables::new();
    let mut statics_vertices_len: usize = 0;
    let mut dynamic_vertices_vertices_len: usize = 0;
    let mut dynamic_indices_vertices_len: usize = 0;

    for node in rtree.root().descendants() {
        if let usvg::NodeKind::Path(ref p) = *node.borrow() {
            let priority = callback.process_events(&node);
            let mut vertex_buffer = VertexBuffers::<Vertex, Index>::new();
            let (priority_container, vertices_len) = match priority {
                Priority::Static => (&mut statics, &mut statics_vertices_len),
                Priority::DynamicVertex => {
                    (&mut dynamic_vertices, &mut dynamic_vertices_vertices_len)
                }
                Priority::DynamicIndex => (&mut dynamic_indices, &mut dynamic_indices_vertices_len),
            };

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
                iterate_stroke(stroke, p, &mut vertex_buffer, color);
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

                iterate_fill(p, &color, &mut vertex_buffer);
            }
            let geometry = GeometryVariable::new(
                vertex_buffer,
                *vertices_len,
                if &p.id == "" {
                    None
                } else {
                    Some((*p.id).to_string())
                },
            );
            *vertices_len += geometry.get_vertices_len();
            priority_container.0.push(geometry);
        }
    }

    let vertices = [
        statics.get_vertices(),
        dynamic_vertices.get_vertices(),
        dynamic_indices.get_vertices(),
    ]
    .concat();
    let indices = [
        statics.get_indices_with_offset(0),
        dynamic_vertices.get_indices_with_offset(statics_vertices_len as u32),
        dynamic_indices.get_indices_with_offset(
            statics_vertices_len as u32 + dynamic_vertices_vertices_len as u32,
        ),
    ]
    .concat();
    ((vertices, indices), rect)
}
