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
use std::collections::HashMap;

pub type ConstraintMap = HashMap<String, Constraint>;

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
                    let clicked = self.click_detection();
                    dbg!(&clicked);
                }
                _ => {}
            }
        }
    }
    pub fn resize(&mut self, p: &PhysicalSize<u32>) {
        self.display_mat4 = Mat4::from_scale([0.5, 0.5, 1.].into()) * size_to_mat4(*p);
    }
    pub fn get_bbox_for(&self, element_name: String) -> Option<Mat4> {
        self.id_to_layout
            .get(&element_name)
            .map(|e| self.calculate_layout(&element_name) * e.bbox)
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

    pub fn click_detection(&self) -> Vec<String> {
        let click = Vec4::from((self.scroll_state.mouse_position, 1., 1.));
        let clicked_ids = self
            .clickables
            .iter()
            .filter_map(|clickable| {
                if clickable.bbox.click_detection(click, &self) {
                    Some(clickable.id.clone())
                } else {
                    None
                }
            })
            .collect::<Vec<String>>();
        clicked_ids
    }
    pub fn add_node(&mut self, node: &Node, pass_down: &mut PassDown, id_suffix: Option<&str>) {
        if !pass_down.is_include {
            return;
        }
        let clickable_regex = Regex::new(CLICKABLE_REGEX).unwrap();
        let layout_regex = Regex::new(LAYOUT_REGEX).unwrap();
        let id = node.id().to_string();
        let id_with_suffix =
            id.clone() + &id_suffix.map_or("".to_string(), |suffix| " ".to_owned() + suffix);
        match layout_regex.is_match(&id_with_suffix) {
            true => {
                let constraint = self
                    .constraint_map
                    .get(&id)
                    .expect(&(id + "not in constraints.json"))
                    .clone();
                let mut layout = Layout::new(&node, constraint);

                layout.parent = pass_down.parent.clone();
                let some_id_with_suffix = (!id_with_suffix.is_empty()).then(|| &id_with_suffix);
                if let Some(id_with_suffix) = some_id_with_suffix {
                    self.layouts.push(id_with_suffix.clone());
                    self.id_to_layout
                        .insert(id_with_suffix.clone(), layout.clone());
                    pass_down.parent = Some(id_with_suffix.clone());
                };
                if clickable_regex.is_match(&id_with_suffix) {
                    let clickable = Clickable {
                        bbox: ClickableBbox::Layout(id_with_suffix.to_string()),
                        id: id_with_suffix.to_string(),
                    };
                    self.clickables.push(clickable)
                }
            }
            false => {
                if clickable_regex.is_match(&id_with_suffix) {
                    let bbox_mat4 = bbox_to_mat4(node.calculate_bbox().unwrap());
                    let clickable = Clickable {
                        bbox: ClickableBbox::Bbox(bbox_mat4),
                        id: id_with_suffix,
                    };
                    self.clickables.push(clickable)
                }
            }
        };
    }
}
