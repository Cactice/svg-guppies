use async_std::task::sleep;
use glam::{DMat4, DVec2, Mat4};
use natura::Spring;
use std::{
    f64::consts::PI,
    hash::{BuildHasher, Hasher},
    iter::zip,
    time::Duration,
};
use sxd_document::parser;
use sxd_xpath::evaluate_xpath;

struct LifeGame {
    dollars: [i32; 4],
    position: [usize; 4],
    turn: u32,
    position_to_dollar: Vec<i32>,
}

struct LifeGameView {
    players_avatar_matrix: [SpringMat4; 4],
    tip_matrix: SpringMat4,
    players_text: [String; 4],
    position_to_coordinates: Vec<DVec2>,
}

impl LifeGameView {
    async fn roulette_clicked(&mut self, life_game: &mut LifeGame) {
        let spins = LifeGame::spin_roulette();
        self.tip_matrix.target = DMat4::from_rotation_z(spins as f64 * PI);
        while !self.tip_matrix.update() {
            sleep(Duration::from_millis(16)).await;
        }
        life_game.proceed(spins);
        let current_player = (life_game.turn % 4) as usize;
        self.players_avatar_matrix[current_player].target = DMat4::from_translation(
            (
                self.position_to_coordinates[life_game.position[current_player]],
                0.0,
            )
                .into(),
        );
        while !self.players_avatar_matrix[current_player].update() {
            sleep(Duration::from_millis(16)).await;
        }
        life_game.finish_turn()
    }
}

struct SpringMat4 {
    pub spring: Spring,
    pub target: DMat4,
    pub current: DMat4,
    pub velocity: DMat4,
}
impl SpringMat4 {
    fn update(&mut self) -> bool {
        zip(
            zip(self.current.to_cols_array(), self.velocity.to_cols_array()),
            self.target.to_cols_array(),
        )
        .for_each(|((mut current_position, mut vel), target)| {
            (current_position, vel) = self.spring.update(current_position, vel, target);
        });
        self.current.abs_diff_eq(self.target, 0.1) && self.velocity.abs_diff_eq(DMat4::ZERO, 0.01)
    }
}

pub(crate) fn rand_u64() -> u64 {
    std::collections::hash_map::RandomState::new()
        .build_hasher()
        .finish()
        % u64::MAX
        / u64::MAX
}
const RANDOM_VARIANCE: u64 = 6;
const RANDOM_BASE: u64 = 10;
const ROULETTE_MAX: u64 = 6;

impl LifeGame {
    fn spin_roulette() -> u64 {
        RANDOM_BASE + (rand_u64() % RANDOM_VARIANCE)
    }
    fn proceed(&mut self, spins: u64) {
        let proceed = spins % ROULETTE_MAX;
        let current_player = (self.turn % 4) as usize;
        self.position[current_player] += proceed as usize;
    }
    fn finish_turn(&mut self) {
        let current_player = (self.turn % 4) as usize;
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
