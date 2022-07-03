use concept::spring::{MutCount, SpringMat4};
use glam::{DVec2, Mat4, Vec2, Vec3};
use regex::{Regex, RegexSet};
use std::iter;
use std::ops::{Deref, DerefMut};
use std::sync::mpsc::{channel, Sender};
use std::{
    f32::consts::PI,
    hash::{BuildHasher, Hasher},
    iter::zip,
};
use windowing::tesselation::callback::{IndicesPriority, InitCallback, Initialization};
use windowing::tesselation::geometry::SvgSet;
use windowing::tesselation::usvg::{Node, NodeExt, NodeKind};
use windowing::winit::dpi::PhysicalSize;
use windowing::winit::event::{ElementState, MouseScrollDelta, WindowEvent};
use windowing::{get_scale, ViewModel};

#[derive(Default)]
struct LifeGame {
    dollars: [i32; 4],
    position: [usize; 4],
    current_player: usize,
    pub position_to_dollar: Vec<i32>,
    position_to_coordinates: Vec<DVec2>,
}

#[derive(Default)]
struct LifeGameView {
    player_avatar_transforms: MutCount<[SpringMat4; 4]>,
    global_transform: MutCount<Mat4>,
    tip_transform: MutCount<SpringMat4>,
    player_texts: MutCount<[String; 4]>,
    instruction_text: MutCount<String>,
    life_game: LifeGame,
    mouse_position: Vec2,
    mouse_down: Option<Vec2>,
}

impl ViewModel for LifeGameView {
    fn reset_mut_count(&mut self) {
        self.player_avatar_transforms.reset_mut_count();
        self.tip_transform.reset_mut_count();
        self.player_texts.reset_mut_count();
        self.instruction_text.reset_mut_count();
    }
    fn into_bytes(&self) -> Option<Vec<u8>> {
        let is_mutated = [
            self.player_avatar_transforms.mut_count,
            self.tip_transform.mut_count,
        ]
        .iter()
        .any(|x| x > &0);
        if is_mutated {
            return None;
        }

        let mat_4: Vec<Mat4> = iter::empty::<Mat4>()
            .chain([self.global_transform.unwrapped])
            .chain(self.player_avatar_transforms.iter().map(|m| m.current))
            .chain([self.tip_transform.current])
            .collect();
        Some(bytemuck::cast_slice(mat_4.as_slice()).to_vec())
    }
    fn into_texts(&self) -> Option<Vec<(String, String)>> {
        let is_mutated = [self.player_texts.mut_count, self.instruction_text.mut_count]
            .iter()
            .any(|x| x > &0);
        if is_mutated {
            return None;
        }

        let texts = iter::empty::<(String, String)>()
            .chain(
                self.player_texts
                    .iter()
                    .enumerate()
                    .map(|(i, m)| (format!("{}. Player #dynamicText", i), m.to_owned())),
            )
            .chain([(
                "instruction #dynamicText".to_string(),
                self.instruction_text.clone(),
            )])
            .collect();
        Some(texts)
    }
    fn on_event(&mut self, svg_set: &SvgSet, event: WindowEvent) {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                let new_position = Vec2::new(position.x as f32, position.y as f32);
                if self.mouse_down.is_some() {
                    let motion = new_position - self.mouse_position;
                    self.global_transform.unwrapped *=
                        Mat4::from_translation(Vec3::from((motion.x, motion.y, 0. as f32)))
                }
                self.mouse_position = new_position
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
            }
            WindowEvent::MouseWheel {
                delta: MouseScrollDelta::PixelDelta(p),
                ..
            } => {
                if p.y != 0. {
                    self.global_transform.unwrapped = Mat4::from_scale(
                        [
                            1. + (1. / (p.y as f32)),
                            1. + (1. / (p.y as f32)),
                            1. as f32,
                        ]
                        .into(),
                    ) * self.global_transform.unwrapped;
                }
            }
            _ => (),
        }
    }
}

impl LifeGameView {
    async fn tip_clicked(&mut self) {
        let life_game = &mut self.life_game;
        if self.tip_transform.complete_animation.is_some()
            || self
                .player_avatar_transforms
                .iter()
                .any(|spring| spring.complete_animation.is_some())
        {
            return;
        }

        let one_sixths_spins = LifeGame::spin_roulette();
        self.tip_transform
            .spring_to(Mat4::from_rotation_z(one_sixths_spins as f32 * PI / 3.))
            .await;

        life_game.proceed(one_sixths_spins);
        self.player_avatar_transforms[life_game.current_player]
            .spring_to(Mat4::from_translation(
                (
                    life_game.position_to_coordinates[life_game.position[life_game.current_player]]
                        .as_vec2(),
                    0.0 as f32,
                )
                    .into(),
            ))
            .await;
        life_game.finish_turn()
    }
}

pub(crate) fn rand_u64() -> u64 {
    std::collections::hash_map::RandomState::new()
        .build_hasher()
        .finish()
        % u64::MAX
        / u64::MAX
}

const RANDOM_VARIANCE: u64 = 12;
const RANDOM_BASE: u64 = 18;
const ROULETTE_MAX: u64 = 6;

impl LifeGame {
    fn spin_roulette() -> u64 {
        RANDOM_BASE + (rand_u64() % RANDOM_VARIANCE)
    }
    fn proceed(&mut self, steps: u64) {
        let proceed = steps % ROULETTE_MAX;
        self.position[self.current_player] =
            (self.position[self.current_player] + proceed as usize).min(self.position.len() - 1);
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
                break;
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

fn main() {
    let mut position_to_dollar: Vec<i32> = vec![];
    let mut position_to_coordinates: Vec<DVec2> = vec![];
    let mut regex_patterns = RegexPatterns::default();
    let _clickable_regex_pattern = regex_patterns.add(r"#clickable(?:$| |#)");
    let _dynamic_regex_pattern = regex_patterns.add(r"#dynamic(?:$| |#)");
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
        if !default_matches.matched(dynamic_text_regex_pattern.index) {
            return Initialization {
                indices_priority: IndicesPriority::Fixed,
                ..Default::default()
            };
        }
        Initialization::default()
    };
    let callback = InitCallback::new(callback_fn);
    let svg_set = SvgSet::new(include_str!("../../svg/life_text.svg"), callback);
    let svg_scale = svg_set.bbox.size;

    let scale: Mat4 = get_scale(PhysicalSize::<u32>::new(1600, 1200), svg_scale);
    let translate = Mat4::from_translation([-1., 1.0, 0.0].into());
    let life_view = LifeGameView {
        global_transform: (translate * scale).into(),
        ..Default::default()
    };
    windowing::main::<LifeGameView>(svg_set, life_view);
}
