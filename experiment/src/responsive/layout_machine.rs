use super::clickable::Clickable;
use super::clickable::ClickableBbox;
use super::constraint::Constraint;
use super::layout::bbox_to_mat4;
use super::layout::size_to_mat4;
use super::layout::Layout;
use crate::scroll::ScrollState;
use crate::svg_init::PassDown;
use crate::svg_init::CLICKABLE_REGEX;
use crate::svg_init::LAYOUT_REGEX;
use core::fmt::Debug;
use guppies::glam::Mat4;
use guppies::glam::Vec3;
use guppies::glam::Vec4;
use guppies::winit::dpi::PhysicalSize;
use guppies::winit::event::ElementState;
use guppies::winit::event::Event;
use guppies::winit::event::WindowEvent;
use regex::Regex;
use salvage::usvg::Node;
use salvage::usvg::NodeExt;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConstraintMap(pub HashMap<String, Constraint>);

#[derive(Debug, Clone, Default)]
pub struct LayoutMachine {
    pub id_to_layout: HashMap<String, Layout>,
    pub layouts: Vec<String>,
    pub clickables: Vec<Clickable>,
    pub svg_mat4: Mat4,
    pub display_mat4: Mat4,
    pub scroll_state: ScrollState,
    pub transforms: Vec<Mat4>,
    pub id_to_transform_index: HashMap<String, usize>,
    pub constraint_map: ConstraintMap,
}

impl LayoutMachine {
    pub fn event_handler(&mut self, event: &Event<()>) {
        self.scroll_state.event_handler(event);
        if let guppies::winit::event::Event::WindowEvent { event, .. } = event {
            match event {
                WindowEvent::Resized(p) => {
                    self.resize(p);
                    let mut transforms = vec![Mat4::IDENTITY, Mat4::IDENTITY];
                    transforms.append(&mut self.get_transforms());
                    self.transforms = transforms;
                }

                WindowEvent::MouseInput {
                    state: ElementState::Pressed,
                    ..
                } => {
                    self.click_detection(&self.scroll_state);
                }
                _ => {}
            }
        }
    }
    pub fn resize(&mut self, p: &PhysicalSize<u32>) {
        self.display_mat4 = Mat4::from_scale([0.5, 0.5, 1.].into()) * size_to_mat4(*p);
    }
    pub fn resize_to_box(&mut self, mat4: Mat4) {
        self.display_mat4 = Mat4::from_scale([0.5, 0.5, 1.].into()) * mat4;
    }
    pub fn get_transform_for(&self, element_name: String) -> Option<Mat4> {
        self.layouts
            .iter()
            .filter(|e| *e == &element_name)
            .map(|e| self.calculate_layout(e))
            .next()
    }
    pub fn get_transforms(&self) -> Vec<Mat4> {
        self.layouts
            .iter()
            .map(|id| self.calculate_layout(id))
            .collect()
    }

    fn calculate_layout(&self, id: &String) -> Mat4 {
        let mut next_parent_name = Some(id);
        let mut parent_layouts = [].to_vec();
        while let Some(current_parent) = next_parent_name {
            let next_parent = self
                .id_to_layout
                .get(current_parent)
                .expect(&format!("Key Should exist: {current_parent}"));
            next_parent_name = next_parent.parent.as_ref();
            parent_layouts.push(next_parent)
        }
        Mat4::from_scale([2., -2., 1.].into())
            * parent_layouts
                .iter()
                .rev()
                .fold(
                    (Mat4::IDENTITY, self.get_display_bbox()),
                    |(_parent_result, parent_bbox), layout| {
                        let layout_result = layout.to_mat4(self.display_mat4, parent_bbox);
                        (
                            layout_result,
                            self.display_mat4 * layout_result * layout.bbox,
                        )
                    },
                )
                .0
    }
    fn get_display_bbox(&self) -> Mat4 {
        let (scale, rot, _trans) = self.display_mat4.to_scale_rotation_translation();
        Mat4::from_scale_rotation_translation(
            scale,
            rot,
            Vec3 {
                x: -scale.x / 2.,
                y: -scale.y / 2.,
                z: 0.,
            },
        )
    }

    pub fn click_detection(&self, scroll_state: &ScrollState) -> Vec<String> {
        let click = Vec4::from((scroll_state.mouse_position, 1., 1.));
        let clicked_ids = self
            .clickables
            .iter()
            .filter_map(|clickable| {
                if clickable.bbox.click_detection(click, self.display_mat4) {
                    Some(clickable.id.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<String>>();
        clicked_ids
    }
    pub fn add_node(&mut self, node: &Node, pass_down: &mut PassDown) {
        let clickable_regex = Regex::new(CLICKABLE_REGEX).unwrap();
        let layout_regex = Regex::new(LAYOUT_REGEX).unwrap();
        let id = &node.id().to_string();
        if layout_regex.is_match(id) {
            let mut layout = Layout::new(&node, &self.constraint_map);
            layout.parent = pass_down.parent.clone();
            let this_id = (!node.id().is_empty()).then(|| node.id().to_string());
            match this_id {
                Some(this_id) => {
                    self.layouts.push(this_id.clone());
                    self.id_to_layout.insert(this_id.clone(), layout.clone());
                    pass_down.parent = Some(this_id.clone());
                }
                None => {}
            };
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
