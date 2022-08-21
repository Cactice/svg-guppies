use crate::{
    callback::{InitCallback, PassDown},
    geometry::Geometry,
};
use guppies::{
    glam::Vec2,
    primitives::{Rect, Triangles},
};
use roxmltree::{Document, NodeId};
use std::{collections::HashMap, sync::Arc};
use usvg::{fontdb::Source, Options, Tree};
use xmlwriter::XmlWriter;
fn recursive_svg(
    node: usvg::Node,
    pass_down: PassDown,
    geometries: &mut Vec<Geometry>,
    callback: &mut InitCallback,
) {
    let (geometry, pass_down) = callback.process_events(&(node.clone(), pass_down));
    if let Some(geometry) = geometry {
        geometries.push(geometry);
    }

    for child in node.children() {
        recursive_svg(child, pass_down, geometries, callback);
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
    pub geometries: Vec<Geometry>,
    pub document: roxmltree::Document<'a>,
    pub id_map: HashMap<String, NodeId>,
    pub bbox: Rect,
    usvg_options: Options,
}
impl<'a> Default for SvgSet<'a> {
    fn default() -> Self {
        Self {
            geometries: vec![],
            document: Document::parse("<e/>").unwrap(),
            id_map: Default::default(),
            bbox: Default::default(),
            usvg_options: Default::default(),
        }
    }
}
impl<'a> SvgSet<'a> {
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
    pub fn get_combined_geometries(&self) -> Geometry {
        self.geometries.iter().fold(
            Geometry::default(),
            |mut acc: Geometry, geometry: &Geometry| {
                acc.extend(&geometry);
                acc
            },
        )
    }
    pub fn get_node_with_id(&self, id: &String) -> Result<roxmltree::Node, &str> {
        let node_id = self.id_map.get(id).ok_or("Not in node_id")?;
        let node = self.document.get_node(*node_id).ok_or("Not in document")?;
        Ok(node)
    }
    pub fn new(xml: &'a str, mut callback: InitCallback) -> Self {
        let font = include_bytes!("../fallback_font/Roboto-Medium.ttf");
        let mut opt = Options::default();
        opt.fontdb
            .load_font_source(Source::Binary(Arc::new(font.as_ref())));
        opt.font_family = "Roboto Medium".to_string();
        opt.keep_named_groups = true;
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
        let mut geometries: Vec<Geometry> = vec![];
        recursive_svg(
            tree.root(),
            PassDown::default(),
            &mut geometries,
            &mut callback,
        );
        let view_box = tree.svg_node().view_box;
        let bbox: Rect = Rect::new(
            Vec2::new(view_box.rect.x() as f32, view_box.rect.y() as f32),
            Vec2::new(view_box.rect.width() as f32, view_box.rect.height() as f32),
        );
        Self {
            geometries,
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
    pub fn update_text(&mut self, id: &String, _new_text: &String) {
        let node = self.get_node_with_id(id).unwrap();
        let _writer = self.get_base_writer();
        let mut parent_ids: Vec<roxmltree::NodeId> = vec![];
        find_text_node_path(node, &mut parent_ids);
    }
}
