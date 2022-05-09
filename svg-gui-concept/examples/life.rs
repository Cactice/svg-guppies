use glam::{DMat4};
use natura::Spring;
use std::{
    hash::{BuildHasher, Hasher},
    iter::zip,
};
use sxd_document::{parser};
use sxd_xpath::{evaluate_xpath};

struct LifeGame {
    dollars: [i32; 4],
    position: [i32; 4],
    turn: u32,
    position_to_dollar: Vec<i32>,
}

struct LifeGameView {
    player1_avatar_matrix: SpringMat4,
    player2_avatar_matrix: SpringMat4,
    player3_avatar_matrix: SpringMat4,
    player4_avatar_matrix: SpringMat4,
    tip_matrix: SpringMat4,
    player1_text: String,
    player2_text: String,
    player3_text: String,
    player4_text: String,
}
struct SpringMat4 {
    spring: Spring,
    target: DMat4,
    current: DMat4,
    velocity: DMat4,
}
impl SpringMat4 {
    fn update(mut self) {
        zip(
            zip(self.current.to_cols_array(), self.velocity.to_cols_array()),
            self.target.to_cols_array(),
        )
        .for_each(|((mut current_position, mut vel), target)| {
            (current_position, vel) = self.spring.update(current_position, vel, target);
        });
    }
}

pub(crate) fn rand_u64() -> u64 {
    std::collections::hash_map::RandomState::new()
        .build_hasher()
        .finish()
        % u64::MAX
        / u64::MAX
}
const RANDOM_VARIANCE: u64 = 18;
const RANDOM_BASE: u64 = 30;
const ROULETTE_MAX: u64 = 6;

impl LifeGame {
    async fn spin_roulette(mut self) {
        let spins = RANDOM_BASE + (rand_u64() % RANDOM_VARIANCE);
        let proceed = spins % ROULETTE_MAX;
        let current_player = (self.turn % 3) as usize;
        self.position[current_player] = proceed as i32;
        let dollar_delta = self
            .position_to_dollar
            .get(current_player)
            .expect("current_player is invalid");
        self.dollars[current_player] += dollar_delta;
        self.turn += 1;
    }
}

fn main() {
    let package = parser::parse(include_str!("../../svg/life.svg")).expect("failed to parse XML");
    let document = package.as_document();
    evaluate_xpath(&document, "//g[@id='Route']/path[matches(@id,'\\d\\.')]")
        .expect("Xpath parsing error");
}
