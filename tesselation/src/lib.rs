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
use usvg::{fontdb::Source, Node};

#[derive(PartialEq, PartialOrd, Eq, Ord, Clone, Copy)]
pub enum IndicesComplexity {
    Static,
    Dynamic,
}

pub struct Callback<'a> {
    func: Box<dyn FnMut(&Node) -> IndicesComplexity + 'a>,
}

impl<'a> Callback<'a> {
    pub fn new(c: impl FnMut(&Node) -> IndicesComplexity + 'a) -> Self {
        Self { func: Box::new(c) }
    }
    fn process_events(&mut self, node: &Node) -> IndicesComplexity {
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
struct GeometryVariablesSet {
    static_geometries: GeometryVariables,
    dynamic_geometries: GeometryVariables,
    static_geometries_vertices_len: usize,
    dynamic_geometries_vertices_len: usize,
}

impl GeometryVariablesSet {
    fn get_indices(&self) -> Indices {
        [
            self.static_geometries.get_indices_with_offset(0),
            self.dynamic_geometries
                .get_indices_with_offset(self.static_geometries_vertices_len as u32),
        ]
        .concat()
    }
    fn get_vertices(&self) -> Vertices {
        [
            self.static_geometries.get_vertices(),
            self.dynamic_geometries.get_vertices(),
        ]
        .concat()
    }
    fn get_vertices_len(&self, priority: IndicesComplexity) -> usize {
        match priority {
            IndicesComplexity::Static => self.static_geometries_vertices_len,
            IndicesComplexity::Dynamic => self.dynamic_geometries_vertices_len,
        }
    }
    fn push_with_priority(&mut self, geometry: GeometryVariable, priority: IndicesComplexity) {
        let (geometries, vertices_len) = match priority {
            IndicesComplexity::Static => (
                &mut self.static_geometries,
                &mut self.static_geometries_vertices_len,
            ),
            IndicesComplexity::Dynamic => (
                &mut self.dynamic_geometries,
                &mut self.dynamic_geometries_vertices_len,
            ),
        };
        *vertices_len += geometry.get_vertices_len();
        geometries.0.push(geometry);
    }
}

#[derive(Clone, Default, Debug)]
struct GeometryVariables(Vec<GeometryVariable>);
impl GeometryVariables {
    fn get_vertices_len(&self, priority: IndicesComplexity) -> usize {
        self.0
            .iter()
            .fold(0, |acc, curr| acc + curr.get_vertices_len())
    }
    fn get_vertices(&self) -> Vertices {
        self.0.iter().flat_map(|v| v.get_v()).collect()
    }
    fn get_indices_with_offset(&self, offset: u32) -> Indices {
        self.0
            .iter()
            .flat_map(|v| v.get_i().iter().map(|i| i + offset).collect::<Indices>())
            .collect()
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

fn recursive(node: Node, priority: IndicesComplexity, callback: &mut Callback) {
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

    let mut geometry_set = GeometryVariablesSet::default();

    for node in rtree.root().descendants() {
        if let usvg::NodeKind::Path(ref p) = *node.borrow() {
            let priority = callback.process_events(&node);
            let mut vertex_buffer = VertexBuffers::<Vertex, Index>::new();

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
                geometry_set.get_vertices_len(priority),
                if &p.id == "" {
                    None
                } else {
                    Some((*p.id).to_string())
                },
            );
            geometry_set.push_with_priority(geometry, priority)
        }
    }

    (
        (geometry_set.get_vertices(), geometry_set.get_indices()),
        rect,
    )
}
