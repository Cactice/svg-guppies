use glam::Mat4;
use natura::{AngularFrequency, DampingRatio, DeltaTime, Spring};
use std::default::Default;
use std::ops::{Deref, DerefMut};
use std::{iter::zip, sync::RwLock};

pub struct SpringMat4 {
    spring: Spring,
    target: Mat4,
    pub current: RwLock<Mat4>,
    velocity: Mat4,
    pub is_animating: bool,
}
impl Default for SpringMat4 {
    fn default() -> Self {
        Self {
            spring: Spring::new(
                DeltaTime(natura::fps(60)),
                AngularFrequency(6.0),
                DampingRatio(0.5),
            ),
            is_animating: false,
            current: Default::default(),
            target: Default::default(),
            velocity: Default::default(),
        }
    }
}

impl SpringMat4 {
    pub fn spring_to(&mut self, target: Mat4) {
        self.target = target;
        self.is_animating = true;
    }
    pub fn update(&mut self) {
        let animating_complete = {
            let mut current = self.current.write().unwrap();
            let mut current_position_vec = vec![];
            let mut vel_vec = vec![];
            zip(
                zip(current.to_cols_array(), self.velocity.to_cols_array()),
                self.target.to_cols_array(),
            )
            .for_each(|((current_position, vel), target)| {
                let (new_current_position, new_vel) =
                    self.spring
                        .update(current_position as f64, vel as f64, target as f64);
                current_position_vec.push(new_current_position as f32);
                vel_vec.push(new_vel as f32);
            });

            *current = Mat4::from_cols_array(&current_position_vec.try_into().unwrap());
            self.velocity = Mat4::from_cols_array(&vel_vec.try_into().unwrap());

            current.abs_diff_eq(self.target, 0.1) && self.velocity.abs_diff_eq(Mat4::ZERO, 0.01)
        };
        if animating_complete {
            self.is_animating = false;
        }
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
