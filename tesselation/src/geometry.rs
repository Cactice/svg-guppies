use crate::{
    callback::{Callback, IndicesPriority},
    fill::iterate_fill,
    stroke::iterate_stroke,
};
use glam::{DVec2, Vec2, Vec4};
use lyon::lyon_tessellation::{FillVertex, StrokeVertex, VertexBuffers};

use roxmltree::{Document, NodeId};
use std::{collections::HashMap, ops::Range, sync::Arc};
use usvg::{fontdb::Source, NodeKind, Options, Path, Tree};
use xmlwriter::XmlWriter;
pub type Index = u32;
pub type Vertices = Vec<Vertex>;
pub type Indices = Vec<Index>;
pub type DrawPrimitives = (Vertices, Indices);
pub type Size = Vec2;
pub type Position = Vec2;
pub type Rect = (Position, Size);

pub const FALLBACK_COLOR: Vec4 = Vec4::ONE;

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
    fn update_geometry(&mut self, id: &String, vertices: Vertices, indices: Indices) {
        let variable_geometry_index = self
            .variable_geometries_id_range
            .get(id)
            .expect("Invalid id for variable_geometries_id_range")
            .start;
        let geometry = self
            .variable_geometries
            .0
            .get_mut(variable_geometry_index)
            .expect("variable_geometry_index is out of bounds");

        let index_base_offset = vertices.len() as i32 - geometry.vertices.len() as i32;
        geometry.vertices = vertices;
        geometry.indices = indices;
        self.variable_geometries.0[variable_geometry_index + 1..]
            .iter_mut()
            .for_each(|geometry: &mut Geometry| {
                geometry.index_base =
                    (geometry.index_base as i32 + index_base_offset as i32) as usize;
            });
    }
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
#[derive(Clone, Debug, Default)]
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
            transform_index: 0,
        }
    }
}

fn recursive_svg(
    node: usvg::Node,
    parent_priority: IndicesPriority,
    callback: &mut Callback,
    geometry_set: &mut GeometrySet,
    mut ids: Vec<String>,
) {
    let priority = parent_priority.max(callback.process_events(&node));
    let node_ref = &node.borrow();
    let id = NodeKind::id(node_ref);
    if !id.is_empty() {
        ids.push(id.to_string());
    }

    if let usvg::NodeKind::Path(ref p) = *node.borrow() {
        let geometry = Geometry::new(p, geometry_set.get_vertices_len(priority), ids.to_vec());
        geometry_set.push_with_priority(geometry, priority);
    }
    for child in node.children() {
        recursive_svg(child, priority, callback, geometry_set, ids.clone());
    }
}

fn find_text_node_path(node: roxmltree::Node, path: &mut Vec<roxmltree::NodeId>) -> bool {
    if node.is_text() {
        return true;
    }
    for child in node.children() {
        if find_text_node_path(child, path) {
            if child.is_element() {
                path.push(child.id());
            }
            return true;
        }
    }
    false
}

#[derive(Debug)]
pub struct SvgSet<'a> {
    pub geometry_set: GeometrySet,
    pub document: roxmltree::Document<'a>,
    pub id_map: HashMap<String, NodeId>,
    pub bbox: Rect,
    usvg_options: Options,
}
impl<'a> SvgSet<'a> {
    fn copy_element_recursively(&self, node: &roxmltree::Node, writer: &mut XmlWriter) {
        if node.is_element() {
            self.copy_element(node, writer);
        }
        for child in node.children() {
            self.copy_element_recursively(&child, writer)
        }
        if node.is_element() {
            writer.end_element()
        }
    }
    fn copy_element(&self, node: &roxmltree::Node, writer: &mut XmlWriter) {
        writer.start_element(node.tag_name().name());
        for a in node.attributes() {
            let name = if a.namespace().is_some() {
                format!("xml:{}", a.name())
            } else {
                a.name().to_string()
            };
            if a.name() != "filter" {
                writer.write_attribute(&name, a.value());
            }
        }
    }

    pub fn get_node_with_id(&self, id: &String) -> Result<roxmltree::Node, &str> {
        let node_id = self.id_map.get(id).ok_or("Not in node_id")?;
        let node = self.document.get_node(*node_id).ok_or("Not in document")?;
        Ok(node)
    }
    pub fn new(xml: &'a str, mut callback: Callback) -> Self {
        let font = include_bytes!("../fallback_font/Roboto-Medium.ttf");
        let mut opt = Options::default();
        opt.fontdb
            .load_font_source(Source::Binary(Arc::new(font.as_ref())));
        opt.font_family = "Roboto Medium".to_string();
        opt.keep_named_groups = true;
        let mut geometry_set = GeometrySet::default();
        let document = Document::parse(xml).unwrap();
        let tree = Tree::from_xmltree(&document, &opt.to_ref()).unwrap();
        let id_map =
            document
                .descendants()
                .fold(HashMap::<String, NodeId>::new(), |mut acc, curr| {
                    if let Some(attribute_id) = curr.attribute("id") {
                        acc.insert(attribute_id.to_string(), curr.id());
                    }
                    acc
                });
        recursive_svg(
            tree.root(),
            IndicesPriority::Fixed,
            &mut callback,
            &mut geometry_set,
            vec![],
        );
        let view_box = tree.svg_node().view_box;
        let bbox: Rect = (
            Vec2::new(view_box.rect.x() as f32, view_box.rect.y() as f32),
            Vec2::new(view_box.rect.width() as f32, view_box.rect.height() as f32),
        );
        Self {
            geometry_set,
            document,
            id_map,
            bbox,
            usvg_options: opt,
        }
    }
    fn get_base_writer(&self) -> XmlWriter {
        let mut writer = XmlWriter::new(xmlwriter::Options {
            use_single_quote: true,
            ..Default::default()
        });
        writer.write_declaration();
        writer.set_preserve_whitespaces(true);
        writer
    }
    pub fn update_text(&mut self, id: &String, new_text: &String) {
        let node = self.get_node_with_id(id).unwrap();
        let mut writer = self.get_base_writer();
        let mut parent_ids: Vec<roxmltree::NodeId> = vec![];
        find_text_node_path(node, &mut parent_ids);

        parent_ids.push(node.id());
        let mut current_node = node;
        while let Some(parent) = current_node.parent() {
            if !parent.is_element() {
                // FIXME: why doesn't simple continue not work..?
                if parent.parent().is_none() {
                    break;
                }
                continue;
            }
            parent_ids.push(parent.id());
            current_node = parent;
        }
        while let Some(parent_id) = parent_ids.pop() {
            let parent = self.document.get_node(parent_id).unwrap();
            self.copy_element(&parent, &mut writer);
            if parent.has_tag_name("svg") {
                writer.write_attribute("xmlns", "http://www.w3.org/2000/svg");
            }
        }
        writer.write_text(new_text);

        let xml = writer.end_document();
        println!("{}", &xml);
        let tree = Tree::from_str(&xml, &self.usvg_options.to_ref()).unwrap();
        let mut geometry_set = GeometrySet::default();
        recursive_svg(
            tree.root(),
            IndicesPriority::Variable,
            &mut Callback::new(|_| IndicesPriority::Variable),
            &mut geometry_set,
            vec![],
        );
        let Geometry {
            indices, vertices, ..
        } = geometry_set
            .variable_geometries
            .0
            .first()
            .expect("No geometry in geometry_set");

        self.geometry_set
            .update_geometry(id, vertices.to_vec(), indices.to_vec());
    }
}
