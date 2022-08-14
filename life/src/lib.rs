use concept::spring::{MutCount, SpringMat4, StaticCallback};
use guppies::glam::{DVec2, Mat4, Vec2, Vec3};
use guppies::primitives::DrawPrimitives;
use guppies::winit::dpi::PhysicalSize;
use guppies::winit::event::{ElementState, MouseScrollDelta, WindowEvent};
use guppies::{get_scale, ViewModel};
use log::info;
use mobile_entry_point::mobile_entry_point;
use regex::{Regex, RegexSet};
use salvage::callback::{IndicesPriority, InitCallback, Initialization};
use salvage::geometry::SvgSet;
use salvage::usvg::{Node, NodeExt, NodeKind};
use std::f32::consts::PI;
use std::iter;
use std::sync::{Arc, Mutex};
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
    animation_register: Arc<Mutex<Vec<SpringMat4>>>,
    player_avatar_transforms: MutCount<[SpringMat4; 4]>,
    tip_center: Mat4,
    start_center: Mat4,
    global_transform: MutCount<Mat4>,
    tip_transform: MutCount<SpringMat4>,
    instruction_text: MutCount<String>,
    life_game: LifeGame,
    mouse_position: Vec2,
    mouse_down: Option<Vec2>,
    svg_set: SvgSet<'a>,
}

impl ViewModel for LifeGameView<'_> {
    fn reset_mut_count(&mut self) {
        self.player_avatar_transforms.reset_mut_count();
        self.tip_transform.reset_mut_count();
        self.instruction_text.reset_mut_count();
    }
    fn on_redraw(&mut self) -> (Option<Vec<u8>>, Option<DrawPrimitives>) {
        {
            let mut r = self.animation_register.lock().unwrap().clone();
            let mut vec: Vec<SpringMat4> = r
                .iter_mut()
                .filter_map(|spring| {
                    let is_animating = !spring.update();
                    if is_animating {
                        Some(spring.clone())
                    } else {
                        None
                    }
                })
                .collect();
            // r2 is necessary because spring.update above will add new elements not in r
            let mut r2 = self.animation_register.lock().unwrap();
            vec.extend_from_slice(&r2[r.len()..]);
            *r2 = vec;
        }
        let _is_mutated = [
            self.player_avatar_transforms.mut_count,
            self.tip_transform.mut_count,
        ]
        .iter()
        .any(|x| x > &0);

        let mat_4: Vec<Mat4> = iter::empty::<Mat4>()
            .chain([self.global_transform.unwrapped])
            .chain([Mat4::IDENTITY])
            .chain(
                self.player_avatar_transforms
                    .iter()
                    .map(|m| m.get_inner().current),
            )
            .chain([self.tip_transform.get_inner().current])
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
            Some((
                self.svg_set.geometry_set.get_vertices(),
                self.svg_set.geometry_set.get_indices(),
            )),
        )
    }
    fn on_event(&mut self, event: WindowEvent) {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                let new_position = Vec2::new(position.x as f32, position.y as f32);
                if self.mouse_down.is_some() {
                    let motion = new_position - self.mouse_position;
                    self.global_transform.unwrapped *=
                        Mat4::from_translation(Vec3::from((motion.x, motion.y, 0_f32)))
                }
                self.mouse_position = new_position
            }
            WindowEvent::Touch(touch) => {
                self.tip_clicked();
            }
            WindowEvent::MouseInput {
                state: ElementState::Released,
                ..
            } => {
                self.mouse_down = None;
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                ..
            } => {
                self.mouse_down = Some(self.mouse_position);
                self.tip_clicked();
            }
            WindowEvent::MouseWheel {
                delta: MouseScrollDelta::PixelDelta(p),
                ..
            } => {
                if p.y != 0. {
                    self.global_transform.unwrapped = Mat4::from_scale(
                        [1. + (1. / (p.y as f32)), 1. + (1. / (p.y as f32)), 1_f32].into(),
                    ) * self.global_transform.unwrapped;
                }
            }
            _ => (),
        }
    }
}

impl LifeGameView<'_> {
    fn tip_clicked(&mut self) {
        if self.tip_transform.get_inner().is_animating
            || self
                .player_avatar_transforms
                .iter()
                .any(|spring| spring.get_inner().is_animating)
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

        let mut arc = self.player_avatar_transforms[life_game.current_player].clone();
        let arc2 = self.animation_register.clone();
        let current = life_game.current_player;
        self.instruction_text = MutCount::from(format!("Player: {current}"));
        life_game.finish_turn();
        self.tip_transform.spring_to(
            self.tip_center
                * Mat4::from_rotation_z(PI / 3. * one_sixths_spins as f32)
                * self.tip_center.inverse(),
            self.animation_register.clone(),
            Some(StaticCallback::new(move |_| {
                arc.spring_to(avatar_mat4, arc2.clone(), None);
            })),
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

#[mobile_entry_point]
fn main() {
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
    let callback_fn = |node: &Node| -> Initialization {
        let node_ref = node.borrow();
        let id = NodeKind::id(&node_ref);
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
        if !default_matches.matched(dynamic_text_regex_pattern.index) {
            return Initialization {
                indices_priority: IndicesPriority::Fixed,
            };
        }
        Initialization::default()
    };
    let callback = InitCallback::new(callback_fn);
    let svg_set = SvgSet::new(include_str!("../../svg/life.svg"), callback);
    let svg_scale = svg_set.bbox.size;

    let scale: Mat4 = get_scale(PhysicalSize::<u32>::new(1600, 1200), svg_scale);
    let translate = Mat4::from_translation([-1., 1.0, 0.0].into());
    let life_view = LifeGameView {
        global_transform: (translate * scale).into(),
        tip_center,
        instruction_text: MutCount::from("Please click".to_string()),
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
