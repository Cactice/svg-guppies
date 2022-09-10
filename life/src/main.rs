use bytemuck::{Pod, Zeroable};
use concept::regex::{get_center, get_default_init_callback};
use concept::scroll::ScrollState;
use concept::spring::SpringMat4;
use guppies::glam::{Mat4, Vec2};
use regex::Regex;
use salvage::callback::InitCallback;
use salvage::svg_set::SvgSet;
use salvage::usvg::NodeExt;
use std::f32::consts::PI;
use std::rc::Rc;

#[derive(Default)]
struct LifeGame {
    dollars: [i32; 4],
    position: [usize; 4],
    current_player: usize,
    pub position_to_dollar: Vec<i32>,
    position_to_coordinates: Vec<Vec2>,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Default)]
struct Texture {
    tip_center: Mat4,
    player_avatar_transforms: [Mat4; 4],
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
            .get(self.position[self.current_player])
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

pub fn main() {
    env_logger::init();
    let mut position_to_dollar: Vec<i32> = vec![];
    let mut position_to_coordinates: Vec<Vec2> = vec![];
    let mut tip_center = Mat4::IDENTITY;
    let mut start_center = Mat4::IDENTITY;
    let coord = Regex::new(r"#coord(?:$| |#)").unwrap();
    let stops = Regex::new(r"^(\d+)\.((?:\+|-)\d+):").unwrap();
    let mut default_callback = get_default_init_callback();
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
    let svg_set = SvgSet::new(include_str!("../../svg/life.svg"), callback);

    let life_game = LifeGame {
        position_to_coordinates,
        position_to_dollar,
        ..Default::default()
    };
    let scroll_state = ScrollState::new_from_svg_set(&svg_set);
    let instruction_text = "Please click".to_string();
    guppies::main(move |event, gpu_redraw| {
        if event.is_none() {
            let geometry = svg_set.get_combined_geometries();
            gpu_redraw.update_triangles(geometry.triangles, 0);
            // gpu_redraw.updateTexture(, 0);
        } else if let Some(event) = event {
            if tip_transform.is_animating
                || life_view
                    .player_avatar_transforms
                    .iter()
                    .any(|spring| spring.is_animating)
            {
                return;
            }

            let one_sixths_spins = LifeGame::spin_roulette();
            let avatar_mat4 = {
                life_game.proceed(one_sixths_spins);
                let target =
                    life_game.position_to_coordinates[life_game.position[life_game.current_player]];
                Mat4::IDENTITY + Mat4::from_translation((target, 0.).into()) - start_center
            };

            let current = life_game.current_player;
            instruction_text = format!("Player: {}", current + 1);
            let cb1 = Rc::new(move |ctx: &mut LifeGameView| {
                let current = ctx.life_game.current_player;
                SpringMat4::<LifeGameView>::spring_to(
                    ctx,
                    Rc::new(move |ctx| &mut ctx.player_avatar_transforms[current]),
                    Rc::new(|ctx, get_life_view| ctx.animation_vec.push(get_life_view)),
                    avatar_mat4,
                    Rc::new(|ctx| {
                        ctx.life_game.finish_turn();
                    }),
                )
            });

            SpringMat4::<LifeGameView>::spring_to(
                &mut life_view,
                Rc::new(|ctx| &mut ctx.tip_transform),
                Rc::new(|ctx, get_self| ctx.animation_vec.push(get_self)),
                life_view.tip_center
                    * Mat4::from_rotation_z(PI / 3. * one_sixths_spins as f32)
                    * life_view.tip_center.inverse(),
                cb1,
            );
        }
    });
}
