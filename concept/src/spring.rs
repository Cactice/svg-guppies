use guppies::glam::Mat4;
use natura::{AngularFrequency, DampingRatio, DeltaTime, Spring};
use std::iter::zip;
use std::marker::PhantomData;
use std::rc::Rc;

#[derive(Clone)]
pub struct SpringMat4<T, G: Fn(&mut T)> {
    _marker: PhantomData<T>,
    spring: Spring,
    target: Mat4,
    velocity: Mat4,
    pub is_animating: bool,
    on_complete: Rc<G>,
}

// impl<T, G: Fn(&mut T)> Default for SpringMat4<T, G> {
// fn default() -> Self {}
// }

impl<T, G: Fn(&mut T)> SpringMat4<T, G> {
    pub fn new(on_complete: G, target: Mat4) -> Self {
        Self {
            spring: Spring::new(
                DeltaTime(natura::fps(60)),
                AngularFrequency(20.0),
                DampingRatio(0.7),
            ),
            is_animating: true,
            target,
            velocity: Default::default(),
            on_complete: Rc::new(on_complete),
            _marker: PhantomData,
        }
    }

    pub fn update(&mut self, current: &mut Mat4, arg: &mut T) {
        if !self.is_animating {
            return;
        }
        let me = self;
        let mut current_position_vec = vec![];
        let mut vel_vec = vec![];

        let animating_complete = {
            zip(
                zip(current.to_cols_array(), me.velocity.to_cols_array()),
                me.target.to_cols_array(),
            )
            .for_each(|((current_position, vel), target)| {
                let (new_current_position, new_vel) =
                    me.spring
                        .update(current_position as f64, vel as f64, target as f64);
                current_position_vec.push(new_current_position as f32);
                vel_vec.push(new_vel as f32);
            });
            *current = Mat4::from_cols_array(&current_position_vec.try_into().unwrap());
            me.velocity = Mat4::from_cols_array(&vel_vec.try_into().unwrap());

            current.abs_diff_eq(me.target, 1.0) && me.velocity.abs_diff_eq(Mat4::ZERO, 100.0)
        };
        if animating_complete {
            me.is_animating = false;
            (me.on_complete)(arg)
        }
    }
}
