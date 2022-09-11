use guppies::glam::Mat4;
use natura::{AngularFrequency, DampingRatio, DeltaTime, Spring};
use std::iter::zip;
use std::rc::Rc;

#[derive(Clone)]
pub struct SpringMat4<T> {
    spring: Spring,
    target: Mat4,
    velocity: Mat4,
    pub is_animating: bool,
    on_complete: Rc<dyn Fn(&mut T) -> ()>,
}

impl<T> Default for SpringMat4<T> {
    fn default() -> Self {
        Self {
            spring: Spring::new(
                DeltaTime(natura::fps(60)),
                AngularFrequency(20.0),
                DampingRatio(0.7),
            ),
            is_animating: false,
            target: Default::default(),
            velocity: Default::default(),
            on_complete: Rc::new(|_| {}),
        }
    }
}

impl<T> SpringMat4<T> {
    pub fn new(target: Mat4, on_complete: Rc<dyn Fn(&mut T) -> ()>) -> Self {
        let mut me = Self::default();
        me.is_animating = true;
        me.target = target;
        me.on_complete = on_complete;
        me
    }

    pub fn update(&mut self, ctx: &mut T, current: Mat4) -> (Mat4, bool) {
        let me = self;
        let mut current_position_vec = vec![];
        let mut vel_vec = vec![];
        let new_current;

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
            new_current = Mat4::from_cols_array(&current_position_vec.try_into().unwrap());
            me.velocity = Mat4::from_cols_array(&vel_vec.try_into().unwrap());

            current.abs_diff_eq(me.target, 1.0) && me.velocity.abs_diff_eq(Mat4::ZERO, 100.0)
        };
        if animating_complete {
            me.is_animating = false;
            (me.on_complete)(ctx)
        }
        (new_current, animating_complete)
    }
}
