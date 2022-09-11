use guppies::{
    get_scale,
    glam::{Mat4, Vec2, Vec3},
    winit::{
        dpi::PhysicalSize,
        event::{ElementState, MouseScrollDelta, TouchPhase, WindowEvent},
    },
};
use salvage::svg_set::SvgSet;
const UNMOVED_RADIUS: f32 = 40.;

#[derive(Default, Debug, Clone)]
pub struct ScrollState {
    pub fingers: Vec<(u64, Vec2)>,
    pub transform: Mat4,
    pub mouse_position: Vec2,
    pub mouse_down: Option<Vec2>,
    pub display_image_size: Vec2,
}
impl ScrollState {
    pub fn new_from_svg_set(svg_set: &SvgSet) -> Self {
        // Below scale should get overridden by guppies' redraw event forced on init
        let svg_scale = svg_set.bbox.size;
        let scale: Mat4 = get_scale(PhysicalSize::<u32>::new(100, 100), svg_scale);
        let translate = Mat4::from_translation([-1., 1.0, 0.0].into());
        Self {
            transform: translate * scale,
            display_image_size: svg_set.bbox.size,
            ..Default::default()
        }
    }
}

pub fn event_handler_for_scroll(event: WindowEvent, scroll_state: &mut ScrollState) -> bool {
    match event {
        WindowEvent::Resized(p) => {
            let (_scale, rot, trans) = scroll_state.transform.to_scale_rotation_translation();
            let scale = get_scale(p, scroll_state.display_image_size)
                .to_scale_rotation_translation()
                .0;
            scroll_state.transform = Mat4::from_scale_rotation_translation(scale, rot, trans);
        }
        WindowEvent::CursorMoved { position, .. } => {
            let new_position = Vec2::new(position.x as f32, position.y as f32);
            if scroll_state.mouse_down.is_some() {
                let motion = new_position - scroll_state.mouse_position;
                scroll_state.transform *=
                    Mat4::from_translation(Vec3::from((motion.x, motion.y, 0_f32)))
            }
            scroll_state.mouse_position = new_position
        }
        WindowEvent::Touch(touch) => match touch.phase {
            TouchPhase::Started => {
                let new_position = Vec2::new(touch.location.x as f32, touch.location.y as f32);
                let fingers_len = scroll_state.fingers.len();
                if fingers_len == 0 {
                    scroll_state.mouse_down = Some(new_position);
                }
                if fingers_len < 2 {
                    scroll_state.fingers.push((touch.id, new_position));
                }
            }
            TouchPhase::Moved => {
                let other_finger: Option<(u64, Vec2)> = scroll_state
                    .fingers
                    .iter()
                    .find(|finger| finger.0 != touch.id)
                    .cloned();
                let this_finger: Option<&mut (u64, Vec2)> = scroll_state
                    .fingers
                    .iter_mut()
                    .find(|finger| finger.0 == touch.id);
                let new_position = Vec2::new(touch.location.x as f32, touch.location.y as f32);
                if let Some(this_finger) = this_finger {
                    let old_position = this_finger.1;
                    if let Some(other_finger) = other_finger {
                        // zoom
                        let other_position = other_finger.1;
                        let original_distance = old_position.distance(other_position);
                        let new_distance = new_position.distance(other_position);
                        let distance_delta = (new_distance - original_distance) * 20.; //TODO: remove this magical number
                        if distance_delta != 0. {
                            scroll_state.transform = Mat4::from_scale(
                                [
                                    1. + (1. / (distance_delta as f32)),
                                    1. + (1. / (distance_delta as f32)),
                                    1_f32,
                                ]
                                .into(),
                            ) * scroll_state.transform;
                        }
                    } else {
                        // pan
                        let motion = new_position - old_position;
                        scroll_state.transform *=
                            Mat4::from_translation(Vec3::from((motion.x, motion.y, 0_f32)))
                    }
                    this_finger.1 = new_position;
                }
            }
            TouchPhase::Ended => {
                let new_position = Vec2::new(touch.location.x as f32, touch.location.y as f32);
                if scroll_state.fingers.len() == 1 {
                    if let Some(mouse_down) = scroll_state.mouse_down {
                        let is_click = UNMOVED_RADIUS > new_position.distance(mouse_down);
                        scroll_state.mouse_down = None;
                        if is_click {
                            scroll_state.mouse_down = None;
                            return true;
                        }
                    }
                }
                scroll_state.fingers = scroll_state
                    .fingers
                    .iter()
                    .filter(|finger| finger.0 != touch.id)
                    .cloned()
                    .collect();
            }
            TouchPhase::Cancelled => scroll_state.fingers = [].to_vec(),
        },
        WindowEvent::MouseInput {
            state: ElementState::Released,
            ..
        } => {
            if let Some(mouse_down) = scroll_state.mouse_down {
                let is_click = UNMOVED_RADIUS > scroll_state.mouse_position.distance(mouse_down);
                scroll_state.mouse_down = None;
                if is_click {
                    return true;
                }
            }
        }
        WindowEvent::MouseInput {
            state: ElementState::Pressed,
            ..
        } => {
            scroll_state.mouse_down = Some(scroll_state.mouse_position);
        }
        WindowEvent::MouseWheel {
            delta: MouseScrollDelta::PixelDelta(p),
            ..
        } => {
            if p.y != 0. {
                scroll_state.transform = Mat4::from_scale(
                    [1. + (1. / (p.y as f32)), 1. + (1. / (p.y as f32)), 1_f32].into(),
                ) * scroll_state.transform;
            }
        }
        _ => (),
    }
    false
}
