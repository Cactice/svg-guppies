use crate::geometry::Geometry;
use guppies::{glam::Vec2, primitives::Rect};
use roxmltree::{Document, NodeId};
use std::{collections::HashMap, sync::Arc};
use usvg::{fontdb::Source, Node, Options, Tree};
use xmlwriter::XmlWriter;

fn recursive_svg<P: Clone, C: FnMut(Node, P) -> (Option<Geometry>, P)>(
    node: usvg::Node,
    pass_down: P,
    geometries: &mut Vec<Geometry>,
    callback: &mut C,
) {
    let (geometry, pass_down) = callback(node.clone(), pass_down);
    if let Some(geometry) = geometry {
        geometries.push(geometry);
    }
    for child in node.children() {
        recursive_svg(child, pass_down.clone(), geometries, callback);
    }
}

fn find_text_node_path(node: roxmltree::Node, path: &mut Vec<roxmltree::NodeId>) -> bool {
    if node.is_text() {
        return true;
    }
    for child in node.children() {
        if find_text_node_path(child, path) {
            if node.is_element() {
                path.push(node.id());
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
    pub root: usvg::Node,
    pub id_to_svg: HashMap<String, NodeId>,
    pub id_to_geometry_index: HashMap<String, usize>,
    pub bbox: Rect,
    usvg_options: Options,
}

impl<'a> Default for SvgSet<'a> {
    fn default() -> Self {
        Self {
            geometries: vec![],
            document: Document::parse("<e/>").unwrap(),
            root: Tree::from_str("<e/>", &Options::default().to_ref())
                .unwrap()
                .root(),
            id_to_svg: Default::default(),
            id_to_geometry_index: Default::default(),
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
        let node_id = self.id_to_svg.get(id).ok_or("Not in node_id")?;
        let node = self.document.get_node(*node_id).ok_or("Not in document")?;
        Ok(node)
    }
    pub fn new<P: Clone, C: FnMut(Node, P) -> (Option<Geometry>, P)>(
        xml: &'a str,
        initial_pass_down: P,
        mut callback: C,
    ) -> Self {
        let font = include_bytes!("../fallback_font/Roboto-Medium.ttf");
        let mut opt = Options::default();
        opt.fontdb
            .load_font_source(Source::Binary(Arc::new(font.as_ref())));
        opt.font_family = "Roboto Medium".to_string();
        opt.keep_named_groups = true;
        let document = Document::parse(xml).unwrap();
        let tree = Tree::from_xmltree(&document, &opt.to_ref()).unwrap();
        let id_to_svg =
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
            initial_pass_down,
            &mut geometries,
            &mut callback,
        );
        geometries.sort_by_key(|a| a.priority);
        let id_to_geometry_index: HashMap<String, usize> =
            geometries
                .iter()
                .enumerate()
                .fold(HashMap::new(), |mut acc, (i, geometry)| {
                    acc.insert(geometry.id.to_owned(), i);
                    acc
                });
        let view_box = tree.svg_node().view_box;
        let bbox: Rect = Rect::new(
            Vec2::new(view_box.rect.x() as f32, view_box.rect.y() as f32),
            Vec2::new(view_box.rect.width() as f32, view_box.rect.height() as f32),
        );
        Self {
            geometries,
            document,
            root: tree.root(),
            id_to_svg,
            id_to_geometry_index,
            bbox,
            usvg_options: opt,
        }
    }
    pub fn update_text(&mut self, id: &str, new_text: &str) {
        let node = self.get_node_with_id(&id.to_string()).unwrap();
        let mut writer = XmlWriter::new(xmlwriter::Options {
            use_single_quote: true,
            ..Default::default()
        });
        writer.set_preserve_whitespaces(true);
        let mut parent_ids: Vec<roxmltree::NodeId> = vec![];

        // TODO: this only works with one line of text
        find_text_node_path(node, &mut parent_ids);

        while let Some(parent_id) = parent_ids.pop() {
            let parent = self.document.get_node(parent_id).unwrap();
            self.copy_element(&parent, &mut writer);
        }
        writer.write_text(new_text);
        let xml = format!(
            "<?xml version='1.0' encoding='UTF-8' standalone='no'?><svg xmlns='http://www.w3.org/2000/svg'>{}</svg>",
            &writer.end_document()
        );
        let tree = Tree::from_str(&xml, &self.usvg_options.to_ref()).unwrap();
        let geometry_to_update =
            &mut self.geometries[*self.id_to_geometry_index.get(id).unwrap() as usize];
        *geometry_to_update = tree.into();
    }
}
