use glam::{DMat4, DVec2};
use natura::Spring;
use regex::{Regex, RegexSet};
use std::sync::mpsc::{channel, Sender};
use std::{
    f64::consts::PI,
    hash::{BuildHasher, Hasher},
    iter::zip,
};
use windowing::tesselation::callback::{IndicesPriority, InitCallback, Initialization};
use windowing::tesselation::usvg::{Node, NodeExt, NodeKind};
use windowing::IntoWindowable;

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
    player_avatar_matrices: [SpringMat4; 4],
    tip_matrix: SpringMat4,
    player_texts: [String; 4],
    instruction_text: String,
}

impl IntoWindowable for LifeGameView {
    fn into_bytes(&self) -> Option<Vec<u8>> {
        let mat_4: Vec<DMat4> = self
            .player_avatar_matrices
            .iter()
            .map(|m| m.current)
            .chain([self.tip_matrix.current])
            .collect();
        Some(bytemuck::cast_vec(mat_4))
    }
    fn into_texts(&self) -> Option<Vec<(String, String)>> {
        let texts: Vec<(String, String)> = self
            .player_texts
            .iter()
            .enumerate()
            .map(|(i, m)| (format!("{}. Player #dynamicText", i), m.to_owned()))
            .chain([(
                "instruction #dynamicText".to_string(),
                self.instruction_text.clone(),
            )])
            .collect();
        Some(texts)
    }
}

impl LifeGameView {
    async fn roulette_clicked(&mut self, life_game: &mut LifeGame) {
        if self.tip_matrix.complete_animation.is_some()
            || self
                .player_avatar_matrices
                .iter()
                .any(|spring| spring.complete_animation.is_some())
        {
            return;
        }

        let one_sixths_spins = LifeGame::spin_roulette();
        self.tip_matrix
            .spring_to(DMat4::from_rotation_z(one_sixths_spins as f64 * PI / 3.))
            .await;

        life_game.proceed(one_sixths_spins);
        self.player_avatar_matrices[life_game.current_player]
            .spring_to(DMat4::from_translation(
                (
                    life_game.position_to_coordinates[life_game.position[life_game.current_player]],
                    0.0,
                )
                    .into(),
            ))
            .await;
        life_game.finish_turn()
    }
}

#[derive(Default)]
struct SpringMat4 {
    spring: Spring,
    target: DMat4,
    current: DMat4,
    velocity: DMat4,
    complete_animation: Option<Sender<()>>,
}

impl SpringMat4 {
    async fn spring_to(&mut self, target: DMat4) {
        self.target = target;
        let (sender, receiver) = channel::<()>();
        self.complete_animation = Some(sender);
        let is_err = receiver.recv().is_err();
        self.complete_animation = None;
        if is_err { /* TODO: How to handle this...?*/ }
    }
    fn update(&mut self) -> bool {
        zip(
            zip(self.current.to_cols_array(), self.velocity.to_cols_array()),
            self.target.to_cols_array(),
        )
        .for_each(|((mut current_position, mut vel), target)| {
            (current_position, vel) = self.spring.update(current_position, vel, target);
        });
        let animating_complete = self.current.abs_diff_eq(self.target, 0.1)
            && self.velocity.abs_diff_eq(DMat4::ZERO, 0.01);
        if let Some(animating_completed) = self.complete_animation.clone() {
            animating_completed.send(()).unwrap();
        }
        self.complete_animation = None;
        animating_complete
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
    let mut position_to_cordinates: Vec<DVec2> = vec![];
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
                position_to_cordinates.resize(stop, coordinate);
            }
            position_to_dollar.insert(stop, dollar);
            position_to_cordinates.insert(stop, coordinate);
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
    let life_view = LifeGameView::default();
    windowing::main::<LifeGameView>(callback, life_view);
}
