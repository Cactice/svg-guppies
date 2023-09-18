use super::clickable::Clickable;
use super::clickable::ClickableBbox;
use super::layout::bbox_to_mat4;
use super::layout::size_to_mat4;
use super::layout::Layout;
use crate::scroll::ScrollState;
use crate::svg_init::CLICKABLE_REGEX;
use crate::svg_init::LAYOUT_REGEX;
use guppies::glam::Mat4;
use guppies::glam::Vec4;
use guppies::winit::dpi::PhysicalSize;
use regex::Regex;
use salvage::usvg::Node;
use salvage::usvg::NodeExt;

#[derive(Debug, Clone, Default)]
pub struct LayoutMachine {
    pub layouts: Vec<Layout>,
    pub clickables: Vec<Clickable>,
    pub svg_mat4: Mat4,
    pub display_mat4: Mat4,
}

impl LayoutMachine {
    pub fn resize(&mut self, p: &PhysicalSize<u32>) {
        self.display_mat4 = size_to_mat4(*p);
    }
    pub fn get_transforms(&self) -> Vec<Mat4> {
        self.layouts
            .iter()
            .map(|layout| layout.to_mat4(self.display_mat4, self.svg_mat4))
            .collect()
    }
    pub fn click_detection(&self, scroll_state: &ScrollState) -> Vec<String> {
        let click = Vec4::from((scroll_state.mouse_position, 1., 1.));
        let clicked_ids = self
            .clickables
            .iter()
            .filter_map(|clickable| {
                if clickable
                    .bbox
                    .click_detection(click, self.display_mat4, self.svg_mat4)
                {
                    Some(clickable.id.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<String>>();
        clicked_ids
    }
    pub fn add_node(&mut self, node: Node) {
        let clickable_regex = Regex::new(CLICKABLE_REGEX).unwrap();
        let layout_regex = Regex::new(LAYOUT_REGEX).unwrap();
        let id = &node.id().to_string();
        if layout_regex.is_match(id) {
            let layout = Layout::new(&node);
            self.layouts.push(layout);
            if clickable_regex.is_match(&id) {
                let clickable = Clickable {
                    bbox: ClickableBbox::Layout(layout),
                    id: id.to_string(),
                };
                self.clickables.push(clickable)
            }
        } else {
            if clickable_regex.is_match(&id) {
                let bbox_mat4 = bbox_to_mat4(node.calculate_bbox().unwrap());
                let clickable = Clickable {
                    bbox: ClickableBbox::Bbox(bbox_mat4),
                    id: id.to_string(),
                };
                self.clickables.push(clickable)
            }
        }
    }
}
