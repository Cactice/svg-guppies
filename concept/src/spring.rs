use guppies::callback::{Callback, MutCallback};
use guppies::glam::Mat4;
use natura::{AngularFrequency, DampingRatio, DeltaTime, Spring};
use std::cell::{Cell, RefCell};
use std::default::Default;
use std::iter::zip;
use std::ops::{Deref, DerefMut};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
pub type GetSelf<T> = Arc<dyn Fn(&mut T) -> &mut SpringMat4<T>>;
pub struct AnimationRegister<T> {
    pub sender: Sender<SpringMat4<T>>,
    pub receiver: Receiver<SpringMat4<T>>,
}
impl<T> Default for AnimationRegister<T> {
    fn default() -> Self {
        let (sender, receiver) = channel();
        Self { sender, receiver }
    }
}
pub struct SpringMat4<T> {
    spring: Spring,
    target: Mat4,
    pub current: Mat4,
    velocity: Mat4,
    pub is_animating: bool,
    on_complete: Arc<dyn Fn(&mut T) -> ()>,
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
            current: Default::default(),
            target: Default::default(),
            velocity: Default::default(),
            on_complete: Arc::new(|_| {}),
        }
    }
}

impl<T> SpringMat4<T> {
    pub fn spring_to(
        ctx: &mut T,
        mut get_self: GetSelf<T>,
        register: Arc<dyn Fn(&mut T, GetSelf<T>) -> ()>,
        target: Mat4,
        on_complete: Arc<dyn Fn(&mut T) -> ()>,
    ) {
        {
            let mut me = get_self(ctx);
            me.is_animating = true;
            me.target = target;
            me.on_complete = on_complete;
        }
        Self::update(ctx, &mut get_self);
        register(ctx, get_self);
    }

    pub fn update(ctx: &mut T, get_self: &mut GetSelf<T>) -> bool {
        let mut current_position_vec = vec![];
        let mut vel_vec = vec![];

        let animating_complete = {
            let mut me = get_self(ctx);
            zip(
                zip(me.current.to_cols_array(), me.velocity.to_cols_array()),
                me.target.to_cols_array(),
            )
            .for_each(|((current_position, vel), target)| {
                let (new_current_position, new_vel) =
                    me.spring
                        .update(current_position as f64, vel as f64, target as f64);
                current_position_vec.push(new_current_position as f32);
                vel_vec.push(new_vel as f32);
            });
            me.current = Mat4::from_cols_array(&current_position_vec.try_into().unwrap());
            me.velocity = Mat4::from_cols_array(&vel_vec.try_into().unwrap());

            me.current.abs_diff_eq(me.target, 1.0) && me.velocity.abs_diff_eq(Mat4::ZERO, 100.0)
        };
        if animating_complete {
            let call = {
                let me = get_self(ctx);
                me.is_animating = false;
                me.on_complete.clone()
            };
            call(ctx);
        }
        animating_complete
    }
}

pub struct MutCount<T> {
    pub unwrapped: T, //TODO: remove pub
    pub mut_count: u8,
}
impl<T: std::default::Default> Default for MutCount<T> {
    fn default() -> Self {
        Self {
            unwrapped: Default::default(),
            mut_count: 1,
        }
    }
}

impl<T> MutCount<T> {
    pub fn reset_mut_count(&mut self) {
        self.mut_count = 0
    }
}
impl<T> From<T> for MutCount<T> {
    fn from(unwrapped: T) -> Self {
        Self {
            unwrapped,
            mut_count: 0,
        }
    }
}
impl<T> Deref for MutCount<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.unwrapped
    }
}
impl<T> DerefMut for MutCount<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.mut_count += 1;
        &mut self.unwrapped
    }
}
