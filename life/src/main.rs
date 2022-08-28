use concept::regex::{default_directives, get_center, RegexPatterns};
use concept::scroll::{event_handler_for_scroll, ScrollState};
use concept::spring::{GetSelf, SpringMat4};
use guppies::glam::{DVec2, Mat4};
use guppies::primitives::{TextureBytes, Triangles};
use guppies::winit::dpi::PhysicalSize;
use guppies::winit::event::WindowEvent;
use guppies::{get_scale, ViewModel};
use regex::Regex;
use salvage::callback::{IndicesPriority, InitCallback, PassDown};
use salvage::svg_set::SvgSet;
use salvage::usvg::{self, NodeExt, NodeKind};
use std::f32::consts::PI;
use std::iter;
use std::rc::Rc;

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
    animation_vec: Vec<GetSelf<Self>>,
    scroll_state: ScrollState,
    player_avatar_transforms: [SpringMat4<Self>; 4],
    tip_center: Mat4,
    start_center: Mat4,
    tip_transform: SpringMat4<Self>,
    instruction_text: String,
    life_game: LifeGame,
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

        let mat_4: Vec<Mat4> = iter::empty::<Mat4>()
            .chain([self.scroll_state.transform])
            .chain([Mat4::IDENTITY])
            .chain(self.player_avatar_transforms.iter().map(|m| m.current))
            .chain([self.tip_transform.current])
            .collect();
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
            Some(self.svg_set.get_combined_geometries().triangles),
        )
    }
    fn on_event(&mut self, event: WindowEvent) {
        if event_handler_for_scroll(event, &mut self.scroll_state) {
            self.tip_clicked()
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
        let cb1 = Rc::new(move |ctx: &mut LifeGameView| {
            let current = ctx.life_game.current_player;
            SpringMat4::<LifeGameView>::spring_to(
                ctx,
                Rc::new(move |ctx| &mut ctx.player_avatar_transforms[current]),
                Rc::new(|ctx, get_self| ctx.animation_vec.push(get_self)),
                avatar_mat4,
                Rc::new(|ctx| {
                    ctx.life_game.finish_turn();
                }),
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
    let mut position_to_coordinates: Vec<DVec2> = vec![];
    let mut tip_center = Mat4::IDENTITY;
    let mut start_center = Mat4::IDENTITY;
    let coord = Regex::new(r"#coord(?:$| |#)").unwrap();
    let stops = Regex::new(r"^(\d+)\.((?:\+|-)\d+):").unwrap();
    let mut default_callback = default_directives();
    let callback = InitCallback::new(|(node, pass_down)| {
        let id = node.id();
        for captures in stops.captures_iter(&id) {
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
        if coord.is_match(&id) {
            let center = get_center(node);
            if id.starts_with("Tip") {
                tip_center = center;
            } else if id.starts_with("0.") {
                start_center = center;
            }
        };
        default_callback.process_events(&(node.clone(), *pass_down))
    });
    let svg_set = SvgSet::new(include_str!("../../svg/life.svg"), callback);
    let svg_scale = svg_set.bbox.size;

    // Below scale should get overridden by guppies' redraw event forced on init
    let scale: Mat4 = get_scale(PhysicalSize::<u32>::new(100, 100), svg_scale);
    let translate = Mat4::from_translation([-1., 1.0, 0.0].into());
    let life_view = LifeGameView {
        life_game: LifeGame {
            position_to_coordinates,
            position_to_dollar,
            ..Default::default()
        },
        scroll_state: ScrollState {
            transform: translate * scale,
            display_image_size: svg_set.bbox.size,
            ..Default::default()
        },
        tip_center,
        start_center,
        instruction_text: "Please click".to_string(),
        svg_set,
        ..Default::default()
    };
    guppies::main::<LifeGameView>(life_view);
}
