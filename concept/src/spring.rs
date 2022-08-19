use guppies::callback::{Callback, MutCallback};
use guppies::glam::Mat4;
use natura::{AngularFrequency, DampingRatio, DeltaTime, Spring};
use std::default::Default;
use std::iter::zip;
use std::ops::{Deref, DerefMut};
use std::sync::mpsc::{channel, Receiver, Sender};

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
    on_complete: Box<dyn FnMut(&mut T) -> ()>,
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
            on_complete: Box::new(|_| {}),
        }
    }
}

pub trait springy<T> {
    fn spring_to(
        self,
        register: Sender<SpringMat4<T>>,
        ctx: &mut T,
        target: Mat4,
        on_complete: Box<dyn FnMut(&mut T) -> ()>,
    );
    fn update(&mut self, t: &mut T) -> bool;
}

impl<T> springy<T> for SpringMat4<T> {
    fn spring_to(
        mut self,
        register: Sender<SpringMat4<T>>,
        ctx: &mut T,
        target: Mat4,
        on_complete: Box<dyn FnMut(&mut T) -> ()>,
    ) {
        self.is_animating = true;
        self.target = target;
        self.update(ctx);
        self.on_complete = on_complete;
        register.send(self);
    }

    fn update(&mut self, ctx: &mut T) -> bool {
        let mut current_position_vec = vec![];
        let mut vel_vec = vec![];
        zip(
            zip(self.current.to_cols_array(), self.velocity.to_cols_array()),
            self.target.to_cols_array(),
        )
        .for_each(|((current_position, vel), target)| {
            let (new_current_position, new_vel) =
                self.spring
                    .update(current_position as f64, vel as f64, target as f64);
            current_position_vec.push(new_current_position as f32);
            vel_vec.push(new_vel as f32);
        });
        self.current = Mat4::from_cols_array(&current_position_vec.try_into().unwrap());
        self.velocity = Mat4::from_cols_array(&vel_vec.try_into().unwrap());

        let animating_complete = self.current.abs_diff_eq(self.target, 1.0)
            && self.velocity.abs_diff_eq(Mat4::ZERO, 100.0);
        if animating_complete {
            self.is_animating = false;
            (self.on_complete)(ctx);
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
