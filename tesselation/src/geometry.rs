use crate::{fill::iterate_fill, stroke::iterate_stroke};
use glam::{DVec2, Vec2, Vec4};
use lyon::lyon_tessellation::{FillVertex, StrokeVertex, VertexBuffers};

use std::{collections::HashMap, ops::Range};
use usvg::{Node, Path};
pub type Index = u32;
pub type Vertices = Vec<Vertex>;
pub type Indices = Vec<Index>;
pub type DrawPrimitives = (Vertices, Indices);
pub type Size = Vec2;
pub type Position = Vec2;
pub type Rect = (Position, Size);

pub const FALLBACK_COLOR: Vec4 = Vec4::ONE;
#[derive(PartialEq, PartialOrd, Eq, Ord, Clone, Copy, Debug)]
pub enum IndicesPriority {
    Fixed,
    Variable,
}

pub struct Callback<'a> {
    func: Box<dyn FnMut(&Node) -> IndicesPriority + 'a>,
}

impl<'a> Callback<'a> {
    pub fn new(c: impl FnMut(&Node) -> IndicesPriority + 'a) -> Self {
        Self { func: Box::new(c) }
    }
    pub fn process_events(&mut self, node: &Node) -> IndicesPriority {
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

#[derive(Clone, Default, Debug)]
pub struct GeometrySet {
    fixed_geometries: Geometries,
    variable_geometries: Geometries,
    fixed_geometries_vertices_len: usize,
    variable_geometries_vertices_len: usize,
    variable_geometries_id_range: HashMap<String, Range<usize>>,
}
impl GeometrySet {
    pub fn get_indices(&self) -> Indices {
        [
            self.fixed_geometries.get_indices_with_offset(0),
            self.variable_geometries
                .get_indices_with_offset(self.fixed_geometries_vertices_len as u32),
        ]
        .concat()
    }
    pub fn get_vertices(&self) -> Vertices {
        [
            self.fixed_geometries.get_vertices(),
            self.variable_geometries.get_vertices(),
        ]
        .concat()
    }
    pub fn get_vertices_len(&self, priority: IndicesPriority) -> usize {
        match priority {
            IndicesPriority::Fixed => self.fixed_geometries_vertices_len,
            IndicesPriority::Variable => self.variable_geometries_vertices_len,
        }
    }
    pub fn push_with_priority(&mut self, geometry: Geometry, priority: IndicesPriority) {
        if priority == IndicesPriority::Variable {
            geometry.ids.iter().for_each(|id| {
                let start = if let Some(range) = self.variable_geometries_id_range.get(id) {
                    range.start
                } else {
                    self.variable_geometries.0.len()
                };
                let end = self.variable_geometries.0.len() + 1;
                let new_range = start..end;
                self.variable_geometries_id_range
                    .insert(id.to_string(), new_range);
            });
        }
        let (geometries, vertices_len) = match priority {
            IndicesPriority::Fixed => (
                &mut self.fixed_geometries,
                &mut self.fixed_geometries_vertices_len,
            ),
            IndicesPriority::Variable => (
                &mut self.variable_geometries,
                &mut self.variable_geometries_vertices_len,
            ),
        };
        *vertices_len += geometry.get_vertices_len();
        geometries.0.push(geometry);
    }
}
#[derive(Clone, Default, Debug)]
pub struct Geometries(Vec<Geometry>);
impl Geometries {
    pub fn get_vertices(&self) -> Vertices {
        self.0.iter().flat_map(|v| v.get_v()).collect()
    }
    pub fn get_indices_with_offset(&self, offset: u32) -> Indices {
        self.0
            .iter()
            .flat_map(|v| v.get_i().iter().map(|i| i + offset).collect::<Indices>())
            .collect()
    }
}
#[derive(Clone, Default, Debug)]
pub struct Geometry {
    ids: Vec<String>,
    vertices: Vertices,
    indices: Indices,
    index_base: usize,
    transform_index: usize,
}
impl Geometry {
    pub fn get_vertices_len(&self) -> usize {
        self.vertices.len()
    }
    pub fn get_v(&self) -> Vertices {
        self.vertices.clone()
    }
    pub fn get_i(&self) -> Indices {
        self.indices
            .iter()
            .map(|index| index + self.index_base as u32)
            .collect()
    }
    pub fn prepare_vertex_buffer(p: &Path) -> VertexBuffers<Vertex, Index> {
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
        };
        vertex_buffer
    }
    pub fn new(p: &Path, index_base: usize, ids: Vec<String>) -> Self {
        let v = Self::prepare_vertex_buffer(p);
        Self {
            ids,
            vertices: v.vertices,
            indices: v.indices,
            index_base,
            ..Default::default()
        }
    }
}
