use crate::geometry::Geometry;
use guppies::{glam::Vec2, primitives::Rect};
use roxmltree::{Document, NodeId};
use std::{collections::HashMap, fmt::Debug, sync::Arc};
use usvg::{fontdb::Source, Node, Options, Tree};
use xmlwriter::XmlWriter;

fn recursive_svg<P: Clone + Debug, C: FnMut(Node, P) -> (Option<Geometry>, P)>(
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
    if node.is_element() {
        path.insert(0, node.id());
    }
    if let Some(text) = node.text() {
        let no_new_line_or_space = text.replace("\n", "").replace(" ", "");
        if no_new_line_or_space.len() != 0 {
            return true;
        }
    }
    for child in node.children() {
        if find_text_node_path(child, path) {
            return true;
        }
    }
    false
}
#[derive(Debug, Default, Clone)]
pub struct SvgSet {
    pub geometries: Vec<Geometry>,
    pub raw_xml: String,
    pub id_to_svg: HashMap<String, NodeId>,
    pub id_to_geometry_index: HashMap<String, usize>,
    pub current_text_map: HashMap<String, String>,
    pub bbox: Rect,
    usvg_options: Arc<Options>,
}

impl SvgSet {
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
        self.geometries
            .iter()
            .fold(Geometry::default(), |acc: Geometry, geometry: &Geometry| {
                acc.extend(&geometry)
            })
    }
    pub fn new<P: Clone + Debug, C: FnMut(Node, P) -> (Option<Geometry>, P)>(
        xml: String,
        initial_pass_down: P,
        mut callback: C,
    ) -> Self {
        let font = include_bytes!("../fallback_font/Roboto-Medium.ttf");
        let mut opt = Options::default();
        opt.fontdb
            .load_font_source(Source::Binary(Arc::new(font.as_ref())));
        opt.font_family = "Roboto Medium".to_string();
        opt.keep_named_groups = true;
        let document = Document::parse(&xml).unwrap();
        let opt = get_usvg_options();
        let tree = Tree::from_xmltree(&document, &opt.to_ref()).unwrap();
        let id_to_svg =
            document
                .descendants()
                .fold(HashMap::<String, NodeId>::new(), |mut acc, cur| {
                    if let Some(attribute_id) = cur.attribute("id") {
                        acc.insert(attribute_id.to_string(), cur.id());
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
            raw_xml: xml.to_string(),
            id_to_svg,
            id_to_geometry_index,
            bbox,
            usvg_options: Arc::new(opt),
            ..Default::default()
        }
    }
    pub fn update_text(&mut self, id: &str, new_text: &str) {
        match self
            .current_text_map
            .insert(id.to_string(), new_text.to_string())
        {
            Some(old_text) if old_text == new_text => {
                return;
            }
            _ => {}
        };

        let document = Document::parse(&self.raw_xml).unwrap();
        let node_id = self.id_to_svg.get(id).ok_or("Not in node_id").unwrap();
        let node = document
            .get_node(*node_id)
            .ok_or("Not in document")
            .unwrap();
        let mut writer = XmlWriter::new(xmlwriter::Options {
            use_single_quote: true,
            ..Default::default()
        });
        writer.set_preserve_whitespaces(true);
        let mut parent_ids: Vec<roxmltree::NodeId> = vec![];

        // TODO: this only works with one line of text
        find_text_node_path(node, &mut parent_ids);

        let mut current_node = node.clone();
        while let Some(parent) = current_node.parent() {
            parent_ids.push(parent.id());
            current_node = parent
        }
        parent_ids.pop();
        parent_ids.pop();

        while let Some(parent_id) = parent_ids.pop() {
            let parent = document.get_node(parent_id).unwrap();
            self.copy_element(&parent, &mut writer);
        }
        writer.write_text(new_text);
        let xml = format!(
            "<?xml version='1.0' encoding='UTF-8' standalone='no'?><svg xmlns='http://www.w3.org/2000/svg'>{}</svg>",
            &writer.end_document()
        );
        let tree = Tree::from_str(&xml, &self.usvg_options.to_ref()).unwrap();
        dbg!(&self.id_to_geometry_index, id);
        let geometry_to_update =
            &mut self.geometries[*self.id_to_geometry_index.get(id).unwrap() as usize];
        let transform_id = geometry_to_update
            .triangles
            .vertices
            .get(0)
            .map_or(1, |v| v.transform_id);
        dbg!(&transform_id);
        self.geometries[*self.id_to_geometry_index.get(id).unwrap() as usize] =
            Geometry::from_tree(tree, transform_id);
    }
}

pub fn get_usvg_options() -> Options {
    let font = include_bytes!("../fallback_font/Roboto-Medium.ttf");
    let mut opt = Options::default();
    opt.fontdb
        .load_font_source(Source::Binary(Arc::new(font.as_ref())));
    opt.font_family = "Roboto Medium".to_string();
    opt.keep_named_groups = true;
    opt
}
