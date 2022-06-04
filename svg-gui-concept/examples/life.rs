use glam::{DMat4, DVec2, Mat4};
use natura::Spring;
use regex::{Regex, RegexSet};
use std::borrow::Borrow;
use std::sync::mpsc::{channel, Sender};
use std::{
    f64::consts::PI,
    hash::{BuildHasher, Hasher},
    iter::zip,
};
use windowing::tesselation::usvg::NodeKind;
use windowing::tesselation::usvg::{Node, Path};
use windowing::tesselation::{Callback, Priority};

#[derive(Default)]
struct LifeGame {
    dollars: [i32; 4],
    position: [usize; 4],
    current_player: usize,
    pub position_to_dollar: Vec<i32>,
}

struct LifeGameView {
    player_avatar_matrices: [SpringMat4; 4],
    tip_matrix: SpringMat4,
    players_text: [String; 4],
    position_to_coordinates: Vec<DVec2>,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct LifeGameViewBytes {
    players_avatar_matrices: Mat4,
    tip_matrix: Mat4,
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
                    self.position_to_coordinates[life_game.position[life_game.current_player]],
                    0.0,
                )
                    .into(),
            ))
            .await;
        life_game.finish_turn()
    }
}

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
                self.position[self.current_player] = self.current_player + n;
                break;
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct RegexPattern<'a> {
    regex_patern: &'a str,
    index: usize,
}
fn main() {
    let mut position_to_dollar: Vec<i32> = vec![];
    let mut i = 0;
    let clickable_regex_pattern = RegexPattern {
        regex_patern: r"#clickable(?:$| |#)",
        index: i,
    };
    i += 1;
    let dynamic_regex_pattern = RegexPattern {
        regex_patern: r"#dynamic(?:$| |#)",
        index: i,
    };
    i += 1;
    let dynamic_text_regex_pattern = RegexPattern {
        regex_patern: r"#dynamicText(?:$| |#)",
        index: i,
    };
    let defaults = RegexSet::new(
        [
            clickable_regex_pattern,
            dynamic_regex_pattern,
            dynamic_text_regex_pattern,
        ]
        .map(|r| r.regex_patern),
    )
    .unwrap();
    let stops = Regex::new(r"^(\d+)\.((?:\+|-)\d+):").unwrap();
    let callback_fn = |node: &Node| -> Priority {
        let node_kind = &node.borrow();
        let id = NodeKind::id(node_kind);
        let default_matches = defaults.matches(&id);
        if default_matches.matched(dynamic_regex_pattern.index) {
            return Priority::DynamicIndex;
        }
        if default_matches.matched(dynamic_text_regex_pattern.index) {
            return Priority::DynamicVertex;
        }

        for captures in stops.captures_iter(&id) {
            let stop: usize = captures[1].parse().unwrap();
            let value: i32 = captures[2].parse().unwrap();
            if stop >= position_to_dollar.len() {
                position_to_dollar.resize(stop, value);
            }
            position_to_dollar.insert(stop, value);
        }
        Priority::Static
    };
    let callback: Callback = Callback::new(callback_fn);
    windowing::main(callback);
}
