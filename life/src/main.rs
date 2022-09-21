use bytemuck::{cast_slice, Pod, Zeroable};
use concept::regex::{get_center, get_default_init_callback};
use concept::scroll::ScrollState;
use concept::spring::SpringMat4;
use guppies::glam::{Mat4, Vec2};
use guppies::winit::event::Event;
use regex::Regex;
use salvage::svg_set::SvgSet;
use salvage::usvg::NodeExt;
use std::f32::consts::PI;

const RANDOM_VARIANCE: u64 = 12;
const RANDOM_BASE: u64 = 18;
const ROULETTE_MAX: u64 = 6;

#[derive(Default)]
struct LifeGame {
    dollars: [i32; 4],
    position: [usize; 4],
    current_player: usize,
    pub position_to_dollar: Vec<i32>,
    position_to_coordinates: Vec<Vec2>,
}

impl LifeGame {
    fn spin_roulette() -> u64 {
        RANDOM_BASE + (fastrand::u64(..) % RANDOM_VARIANCE)
    }
    fn proceed(&mut self, steps: u64) -> Vec2 {
        let proceed = steps % ROULETTE_MAX + 1;
        self.position[self.current_player] = (self.position[self.current_player]
            + proceed as usize)
            .min(self.position_to_coordinates.len() - 1);
        self.position_to_coordinates[self.position[self.current_player]]
    }
    fn finish_turn(&mut self) {
        let dollar_delta = self.position_to_dollar[self.position[self.current_player]];
        self.dollars[self.current_player] += dollar_delta;
        for n in 1..4 {
            self.current_player = (self.current_player + n) % 4;
            if self.position[self.current_player] < self.position_to_dollar.len() - 1 {
                break;
            }
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Default)]
struct Texture {
    identity_transform: Mat4,
    player_avatar_transforms: [Mat4; 4],
    tip_transform: Mat4,
}

pub fn main() {
    env_logger::init();
    let mut position_to_dollar: Vec<i32> = vec![];
    let mut position_to_coordinates: Vec<Vec2> = vec![];
    let mut tip_center = Mat4::IDENTITY;
    let mut default_callback = get_default_init_callback();
    let coord = Regex::new(r"#coord(?:$| |#)").unwrap();
    let stops = Regex::new(r"^(\d+)\.((?:\+|-)\d+):").unwrap();
    let mut svg_set = SvgSet::new(include_str!("../../svg/life.svg"), |node, pass_down| {
        let id = node.id();
        for captures in stops.captures_iter(&id) {
            let stop: usize = captures[1].parse().unwrap();
            let dollar: i32 = captures[2].parse().unwrap();
            let coordinate = get_center(&node);
            if stop >= position_to_dollar.len() {
                position_to_dollar.resize(stop, dollar);
                position_to_coordinates.resize(stop, coordinate);
            }
            position_to_dollar.insert(stop, dollar);
            position_to_coordinates.insert(stop, coordinate);
        }
        if coord.is_match(&id) {
            let center = Mat4::from_translation((get_center(&node), 0.).into());
            if id.starts_with("Tip") {
                tip_center = center;
            }
        };
        default_callback(node.clone(), pass_down)
    });
    let mut life_game = LifeGame {
        position_to_coordinates,
        position_to_dollar,
        ..Default::default()
    };
    let mut scroll_state = ScrollState::new_from_svg_set(&svg_set);
    let mut texture = Texture::default();
    svg_set.update_text("instruction #dynamicText", "Please click");
    let mut tip_animation = SpringMat4::default();
    let mut player_animations = texture
        .player_avatar_transforms
        .map(|_| SpringMat4::default());
    let start_center = Mat4::from_translation((life_game.position_to_coordinates[0], 0.).into());
    guppies::render_loop(move |event, gpu_redraw| {
        let clicked = scroll_state.event_handler(event);
        if clicked {
            if tip_animation.is_animating
                || player_animations.iter().any(|spring| spring.is_animating)
            {
                return;
            }
            let one_sixths_spins = LifeGame::spin_roulette();
            let target = life_game.proceed(one_sixths_spins);
            let current_player = life_game.current_player;
            let instruction_text = format!("Player: {}", life_game.current_player + 1);
            svg_set.update_text("instruction #dynamicText", &instruction_text);
            life_game.finish_turn();
            let money = life_game.dollars[current_player];
            let after_tip_animation = move |player_animations: &mut [SpringMat4<SvgSet>; 4]| {
                player_animations[current_player].set_target(
                    Mat4::IDENTITY + Mat4::from_translation((target, 0.).into()) - start_center,
                    move |svg_set| {
                        svg_set.update_text(
                            &format!("{}. Player #dynamicText", current_player + 1),
                            &format!("${}", money),
                        )
                    },
                )
            };
            tip_animation.set_target(
                tip_center
                    * Mat4::from_rotation_z(PI / 3. * one_sixths_spins as f32)
                    * tip_center.inverse(),
                after_tip_animation,
            );
        }
        if let Event::RedrawRequested(_) = event {
            tip_animation.update(&mut texture.tip_transform, &mut player_animations);
            player_animations
                .iter_mut()
                .enumerate()
                .for_each(|(i, animation)| {
                    animation.update(&mut texture.player_avatar_transforms[i], &mut svg_set);
                });
        }
        let geometry = svg_set.get_combined_geometries();
        gpu_redraw.update_triangles(geometry.triangles, 0);
        gpu_redraw.update_texture(
            [
                cast_slice(&[scroll_state.transform]),
                cast_slice(&[texture.clone()]),
            ]
            .concat(),
        );
    });
}
