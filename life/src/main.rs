use bytemuck::{cast_slice, Pod, Zeroable};
use concept::regex::{get_center, get_default_init_callback};
use concept::scroll::ScrollState;
use concept::spring::SpringMat4;
use guppies::glam::{Mat4, Vec2};
use regex::Regex;
use salvage::callback::InitCallback;
use salvage::svg_set::SvgSet;
use salvage::usvg::NodeExt;
use std::cell::RefCell;
use std::f32::consts::PI;
use std::rc::Rc;

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
            if n == 4 {
                todo!("game finished")
            } else {
                self.current_player = (self.current_player + n) % 4;
                if self.position[self.current_player] < self.position_to_dollar.len() - 1 {
                    break;
                }
            }
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Default)]
struct Texture {
    identity_transform: Mat4,
    tip_transform: Mat4,
    player_avatar_transforms: [Mat4; 4],
}

#[derive(Default, Clone)]
struct AnimationRegisterer {
    texture: Texture,
    registerer: Vec<SpringMat4>,
}
impl AnimationRegisterer {
    fn spring_to<F: Fn(&mut Self) -> &mut Mat4, G: FnMut() + 'static>(
        &mut self,
        get_current: F,
        target: Mat4,
        on_complete: G,
    ) {
        let current = get_current(self).clone();
        let mut spring = SpringMat4::new(target, Rc::new(RefCell::new(on_complete)));
        *get_current(self) = spring.update(current).0;
        self.registerer.push(spring);
    }
    fn update(&mut self) {
        self.registerer = self
            .registerer
            .iter()
            .filter(|a| a.is_animating)
            .cloned()
            .collect();
    }
}

pub fn main() {
    env_logger::init();
    let mut position_to_dollar: Vec<i32> = vec![];
    let mut position_to_coordinates: Vec<Vec2> = vec![];
    let mut tip_center = Mat4::IDENTITY;
    let mut start_center = Mat4::IDENTITY;
    let mut default_callback = get_default_init_callback();
    let coord = Regex::new(r"#coord(?:$| |#)").unwrap();
    let stops = Regex::new(r"^(\d+)\.((?:\+|-)\d+):").unwrap();
    let callback = InitCallback::new(|(node, pass_down)| {
        let id = node.id();
        for captures in stops.captures_iter(&id) {
            let stop: usize = captures[1].parse().unwrap();
            let dollar: i32 = captures[2].parse().unwrap();
            let coordinate = get_center(node);
            if stop >= position_to_dollar.len() {
                position_to_dollar.resize(stop, dollar);
                position_to_coordinates.resize(stop, coordinate);
            }
            position_to_dollar.insert(stop, dollar);
            position_to_coordinates.insert(stop, coordinate);
        }
        if coord.is_match(&id) {
            let center = Mat4::from_translation((get_center(node), 0.).into());
            if id.starts_with("Tip") {
                tip_center = center;
            } else if id.starts_with("0.") {
                start_center = center;
            }
        };
        default_callback.process_events(&(node.clone(), *pass_down))
    });
    let mut svg_set = SvgSet::new(include_str!("../../svg/life.svg"), callback);

    let mut life_game = LifeGame {
        position_to_coordinates,
        position_to_dollar,
        ..Default::default()
    };
    let mut animation_registerer = AnimationRegisterer::default();
    let mut scroll_state = ScrollState::new_from_svg_set(&svg_set);
    svg_set.update_text("instruction #dynamicText", "Please click");
    guppies::main_loop(move |event, gpu_redraw| {
        let clicked = scroll_state.event_handler(event);
        if clicked {
            // if spring_tip.is_animating || spring_players.iter().any(|spring| spring.is_animating) {
            //     return;
            // }
            let one_sixths_spins = LifeGame::spin_roulette();
            let target = life_game.proceed(one_sixths_spins);
            let avatar_mat4 =
                Mat4::IDENTITY + Mat4::from_translation((target, 0.).into()) - start_center;

            // instruction_text = format!("Player: {}", life_game.current_player + 1);

            // let cb1 = || {
            //     animation_registerer.spring_to(
            //         |r| &mut r.texture.player_avatar_transforms[life_game.current_player],
            //         avatar_mat4,
            //         || {
            //             life_game.finish_turn();
            //         },
            //     )
            // };
            animation_registerer.spring_to(
                |r| &mut r.texture.tip_transform,
                tip_center
                    * Mat4::from_rotation_z(PI / 3. * one_sixths_spins as f32)
                    * tip_center.inverse(),
                || {},
            );
        }
        let geometry = svg_set.get_combined_geometries();
    });
}
