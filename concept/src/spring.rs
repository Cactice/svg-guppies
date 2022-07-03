use glam::Mat4;
use natura::{AngularFrequency, DampingRatio, DeltaTime, Spring};
use std::{
    iter::zip,
    ops::{Deref, DerefMut},
    sync::mpsc::{channel, Sender},
};

pub struct SpringMat4 {
    spring: Spring,
    target: Mat4,
    pub current: Mat4,
    velocity: Mat4,
    pub complete_animation: Option<Sender<()>>,
}
impl Default for SpringMat4 {
    fn default() -> Self {
        Self {
            spring: Spring::new(
                DeltaTime(natura::fps(60)),
                AngularFrequency(6.0),
                DampingRatio(0.5),
            ),
            complete_animation: None,
            current: Default::default(),
            target: Default::default(),
            velocity: Default::default(),
        }
    }
}

impl SpringMat4 {
    pub async fn spring_to(&mut self, target: Mat4) {
        self.target = target;
        let (sender, receiver) = channel::<()>();
        self.complete_animation = Some(sender);
        let is_err = receiver.recv().is_err();
        self.complete_animation = None;
        if is_err { /* TODO: How to handle this...?*/ }
    }
    fn update(&mut self) -> bool {
        zip(
            zip(self.current.to_cols_array(), self.velocity.to_cols_array()),
            self.target.to_cols_array(),
        )
        .for_each(|((mut current_position, mut vel), target)| {
            let (new_current_position, new_vel) =
                self.spring
                    .update(current_position as f64, vel as f64, target as f64);
            current_position = new_current_position as f32;
            vel = new_vel as f32;
        });
        let animating_complete = self.current.abs_diff_eq(self.target, 0.1)
            && self.velocity.abs_diff_eq(Mat4::IDENTITY, 0.01);
        if let Some(animating_completed) = self.complete_animation.clone() {
            animating_completed.send(()).unwrap();
        }
        self.complete_animation = None;
        animating_complete
    }
}

#[derive(Default)]
pub struct MutCount<T> {
    pub unwrapped: T, //TODO: remove pub
    pub mut_count: u8,
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
