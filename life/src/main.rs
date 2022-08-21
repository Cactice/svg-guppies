use concept::spring::{GetSelf, SpringMat4};
use guppies::glam::{DVec2, Mat4, Vec2, Vec3};
use guppies::primitives::{TextureBytes, Triangles};
use guppies::winit::dpi::PhysicalSize;
use guppies::winit::event::{ElementState, MouseScrollDelta, TouchPhase, WindowEvent};
use guppies::{get_scale, ViewModel};
use regex::{Regex, RegexSet};
use salvage::callback::{IndicesPriority, InitCallback, PassDown};
use salvage::geometry::Geometry;
use salvage::svg_set::SvgSet;
use salvage::usvg::{self, NodeExt, NodeKind};
use std::f32::consts::PI;
use std::iter;
use std::rc::Rc;

const UNMOVED_RADIUS: f32 = 40.;

#[derive(Default)]
struct LifeGame {
    dollars: [i32; 4],
    position: [usize; 4],
    current_player: usize,
    pub position_to_dollar: Vec<i32>,
    position_to_coordinates: Vec<DVec2>,
}

#[derive(Default)]
struct LifeGameView<'a> {
    fingers: Vec<(u64, Vec2)>,
    animation_vec: Vec<GetSelf<Self>>,
    player_avatar_transforms: [SpringMat4<Self>; 4],
    tip_center: Mat4,
    start_center: Mat4,
    global_transform: Mat4,
    tip_transform: SpringMat4<Self>,
    instruction_text: String,
    life_game: LifeGame,
    mouse_position: Vec2,
    mouse_down: Option<Vec2>,
    svg_set: SvgSet<'a>,
}

impl ViewModel for LifeGameView<'_> {
    fn on_redraw(&mut self) -> (Option<TextureBytes>, Option<Triangles>) {
        {
            self.animation_vec.clone().iter_mut().for_each(|a| {
                SpringMat4::<Self>::update(self, a);
            });
            self.animation_vec = self
                .animation_vec
                .clone()
                .into_iter()
                .filter(|a| a(self).is_animating)
                .collect();
        }
        // let _is_mutated = [self.player_avatar_transforms.mut_count]
        //     .iter()
        //     .any(|x| x > &0);

        let mat_4: Vec<Mat4> = iter::empty::<Mat4>()
            .chain([self.global_transform])
            .chain([Mat4::IDENTITY])
            .chain(self.player_avatar_transforms.iter().map(|m| m.current))
            .chain([self.tip_transform.current])
            .collect();
        // let _is_mutated = [self.instruction_text.mut_count].iter().any(|x| x > &0);
        iter::empty::<(String, String)>()
            .chain(
                self.life_game
                    .dollars
                    .iter()
                    .enumerate()
                    .map(|(i, m)| (format!("{}. Player #dynamicText", i + 1), format!("${}", m)))
                    .collect::<Vec<(String, String)>>(),
            )
            .chain([(
                "instruction #dynamicText".to_string(),
                self.instruction_text.clone(),
            )])
            .for_each(|(id, new_text)| {
                self.svg_set.update_text(&id, &new_text);
            });
        (
            Some(bytemuck::cast_slice(mat_4.as_slice()).to_vec()),
            Some(self.svg_set.get_triangles_with_offset()),
        )
    }
    fn on_event(&mut self, event: WindowEvent) {
        match event {
            WindowEvent::Resized(p) => {
                let (_scale, rot, trans) = self.global_transform.to_scale_rotation_translation();
                let scale = get_scale(p, self.svg_set.bbox.size);
                self.global_transform = Mat4::from_scale_rotation_translation(
                    scale.to_scale_rotation_translation().0,
                    rot,
                    trans,
                );
            }
            WindowEvent::CursorMoved { position, .. } => {
                let new_position = Vec2::new(position.x as f32, position.y as f32);
                if self.mouse_down.is_some() {
                    let motion = new_position - self.mouse_position;
                    self.global_transform *=
                        Mat4::from_translation(Vec3::from((motion.x, motion.y, 0_f32)))
                }
                self.mouse_position = new_position
            }
            WindowEvent::Touch(touch) => match touch.phase {
                TouchPhase::Started => {
                    let new_position = Vec2::new(touch.location.x as f32, touch.location.y as f32);
                    let fingers_len = self.fingers.len();
                    if fingers_len == 0 {
                        self.mouse_down = Some(new_position);
                    }
                    if fingers_len < 2 {
                        self.fingers.push((touch.id, new_position));
                    }
                }
                TouchPhase::Moved => {
                    let other_finger: Option<(u64, Vec2)> = self
                        .fingers
                        .iter()
                        .find(|finger| finger.0 != touch.id)
                        .cloned();
                    let this_finger: Option<&mut (u64, Vec2)> =
                        self.fingers.iter_mut().find(|finger| finger.0 == touch.id);
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
                                self.global_transform = Mat4::from_scale(
                                    [
                                        1. + (1. / (distance_delta as f32)),
                                        1. + (1. / (distance_delta as f32)),
                                        1_f32,
                                    ]
                                    .into(),
                                ) * self.global_transform;
                            }
                        } else {
                            // pan
                            let motion = new_position - old_position;
                            self.global_transform *=
                                Mat4::from_translation(Vec3::from((motion.x, motion.y, 0_f32)))
                        }
                        this_finger.1 = new_position;
                    }
                }
                TouchPhase::Ended => {
                    let new_position = Vec2::new(touch.location.x as f32, touch.location.y as f32);
                    if self.fingers.len() == 1 {
                        if let Some(mouse_down) = self.mouse_down {
                            if UNMOVED_RADIUS > new_position.distance(mouse_down) {
                                self.tip_clicked()
                            }
                            self.mouse_down = None;
                        }
                    }
                    self.fingers = self
                        .fingers
                        .iter()
                        .filter(|finger| finger.0 != touch.id)
                        .cloned()
                        .collect();
                }
                TouchPhase::Cancelled => self.fingers = [].to_vec(),
            },
            WindowEvent::MouseInput {
                state: ElementState::Released,
                ..
            } => {
                if let Some(mouse_down) = self.mouse_down {
                    if UNMOVED_RADIUS > self.mouse_position.distance(mouse_down) {
                        self.tip_clicked()
                    }
                }
                self.mouse_down = None;
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                ..
            } => {
                self.mouse_down = Some(self.mouse_position);
            }
            WindowEvent::MouseWheel {
                delta: MouseScrollDelta::PixelDelta(p),
                ..
            } => {
                if p.y != 0. {
                    self.global_transform = Mat4::from_scale(
                        [1. + (1. / (p.y as f32)), 1. + (1. / (p.y as f32)), 1_f32].into(),
                    ) * self.global_transform;
                }
            }
            _ => (),
        }
    }
}

impl LifeGameView<'_> {
    fn tip_clicked(&mut self) {
        if self.tip_transform.is_animating
            || self
                .player_avatar_transforms
                .iter()
                .any(|spring| spring.is_animating)
        {
            return;
        }

        let one_sixths_spins = LifeGame::spin_roulette();
        let life_game = &mut self.life_game;
        let avatar_mat4 = {
            life_game.proceed(one_sixths_spins);
            let target = life_game.position_to_coordinates
                [life_game.position[life_game.current_player]]
                .as_vec2();
            Mat4::IDENTITY + Mat4::from_translation((target, 0.).into()) - self.start_center
        };

        let current = life_game.current_player;
        self.instruction_text = format!("Player: {}", current + 1);
        life_game.finish_turn();
        let cb1 = Rc::new(move |ctx: &mut LifeGameView| {
            let current = ctx.life_game.current_player;
            SpringMat4::<LifeGameView>::spring_to(
                ctx,
                Rc::new(move |ctx| &mut ctx.player_avatar_transforms[current]),
                Rc::new(|ctx, get_self| ctx.animation_vec.push(get_self)),
                avatar_mat4,
                Rc::new(|_| {}),
            )
        });

        SpringMat4::<LifeGameView>::spring_to(
            self,
            Rc::new(|ctx| &mut ctx.tip_transform),
            Rc::new(|ctx, get_self| ctx.animation_vec.push(get_self)),
            self.tip_center
                * Mat4::from_rotation_z(PI / 3. * one_sixths_spins as f32)
                * self.tip_center.inverse(),
            cb1,
        );
    }
}

const RANDOM_VARIANCE: u64 = 12;
const RANDOM_BASE: u64 = 18;
const ROULETTE_MAX: u64 = 6;

impl LifeGame {
    fn spin_roulette() -> u64 {
        RANDOM_BASE + (fastrand::u64(..) % RANDOM_VARIANCE)
    }
    fn proceed(&mut self, steps: u64) {
        let proceed = steps % ROULETTE_MAX + 1;
        self.position[self.current_player] = (self.position[self.current_player]
            + proceed as usize)
            .min(self.position_to_coordinates.len() - 1);
    }
    fn finish_turn(&mut self) {
        let dollar_delta = self
            .position_to_dollar
            .get(self.current_player)
            .expect("current_player is invalid");
        self.dollars[self.current_player] += dollar_delta;
        for n in 1..4 {
            if n == 4 {
                todo!("game finished")
            } else {
                self.current_player += n;
                self.current_player %= 4;
                let length_of_positions = self.position_to_dollar.len() - 1;
                if self.position[self.current_player] < length_of_positions {
                    break;
                }
            }
        }
    }
}

#[derive(Clone, Debug, Default)]
struct RegexPattern {
    regex_pattern: String,
    index: usize,
}
#[derive(Clone, Debug, Default)]
struct RegexPatterns(Vec<RegexPattern>);

impl RegexPatterns {
    fn add(&mut self, regex_pattern: &str) -> RegexPattern {
        let regex_pattern = RegexPattern {
            regex_pattern: regex_pattern.to_string(),
            index: self.0.len(),
        };
        self.0.push(regex_pattern.clone());
        regex_pattern
    }
}

pub fn main() {
    env_logger::init();
    let mut position_to_dollar: Vec<i32> = vec![];
    let mut position_to_coordinates: Vec<DVec2> = vec![];
    let mut regex_patterns = RegexPatterns::default();
    let mut tip_center = Mat4::IDENTITY;
    let mut start_center = Mat4::IDENTITY;
    let _clickable_regex_pattern = regex_patterns.add(r"#clickable(?:$| |#)");
    let _dynamic_regex_pattern = regex_patterns.add(r"#dynamic(?:$| |#)");
    let coord_regex_pattern = regex_patterns.add(r"#coord(?:$| |#)");
    let dynamic_text_regex_pattern = regex_patterns.add(r"#dynamicText(?:$| |#)");
    let defaults = RegexSet::new(regex_patterns.0.iter().map(|r| &r.regex_pattern)).unwrap();
    let stops = Regex::new(r"^(\d+)\.((?:\+|-)\d+):").unwrap();
    let mut transform_count = 1;
    let callback = InitCallback::new(|(node, pass_down)| {
        let PassDown {
            transform_id: parent_transform_id,
            indices_priority: parent_priority,
        } = pass_down;
        let node_ref = node.borrow();
        let id = NodeKind::id(&node_ref);
        let transform_id = if id.ends_with("#dynamic") {
            transform_count += 1;
            transform_count
        } else {
            *parent_transform_id
        };
        for captures in stops.captures_iter(id) {
            let stop: usize = captures[1].parse().unwrap();
            let dollar: i32 = captures[2].parse().unwrap();
            let bbox = node.calculate_bbox().unwrap();
            let coordinate =
                DVec2::new(bbox.x() + bbox.width() / 2., bbox.y() + bbox.height() / 2.);
            if stop >= position_to_dollar.len() {
                position_to_dollar.resize(stop, dollar);
                position_to_coordinates.resize(stop, coordinate);
            }
            position_to_dollar.insert(stop, dollar);
            position_to_coordinates.insert(stop, coordinate);
        }
        let default_matches = defaults.matches(id);
        if default_matches.matched(coord_regex_pattern.index) {
            let bbox = node.calculate_bbox().unwrap();
            let center = Mat4::from_translation(
                [
                    (bbox.x() + bbox.width() / 2.) as f32,
                    (bbox.y() + bbox.height() / 2.) as f32,
                    0.,
                ]
                .into(),
            );
            if id.starts_with("Tip") {
                tip_center = center
            } else {
                start_center = center
            }
        }
        let indices_priority = if !default_matches.matched(dynamic_text_regex_pattern.index) {
            IndicesPriority::Variable
        } else {
            IndicesPriority::Fixed
        };
        let geometry = {
            if let usvg::NodeKind::Path(ref p) = *node.borrow() {
                Some(Geometry::new(p, transform_id, indices_priority))
            } else {
                None
            }
        };
        let indices_priority = *parent_priority.max(&indices_priority);
        (
            geometry,
            PassDown {
                indices_priority,
                transform_id,
            },
        )
    });
    let svg_set = SvgSet::new(include_str!("../../svg/life.svg"), callback);
    let svg_scale = svg_set.bbox.size;

    // Below scale should get overridden by guppies' redraw event forced on init
    let scale: Mat4 = get_scale(PhysicalSize::<u32>::new(100, 100), svg_scale);
    let translate = Mat4::from_translation([-1., 1.0, 0.0].into());
    let life_view = LifeGameView {
        global_transform: translate * scale,
        tip_center,
        instruction_text: "Please click".to_string(),
        start_center,
        life_game: LifeGame {
            position_to_coordinates,
            position_to_dollar,
            ..Default::default()
        },
        svg_set,
        ..Default::default()
    };
    guppies::main::<LifeGameView>(life_view);
}
