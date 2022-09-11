use guppies::glam::Mat4;
use natura::{AngularFrequency, DampingRatio, DeltaTime, Spring};
use std::cell::RefCell;
use std::default::Default;
use std::iter::zip;
use std::rc::Rc;

pub struct SpringMat4<'a> {
    spring: Spring,
    target: Mat4,
    pub current: &'a mut Mat4,
    velocity: Mat4,
    pub is_animating: bool,
    on_complete: Rc<RefCell<dyn FnMut() -> ()>>,
}

impl<'a> SpringMat4<'a> {
    pub fn new<'b: 'a>(current: &'b mut Mat4) -> Self {
        Self {
            spring: Spring::new(
                DeltaTime(natura::fps(60)),
                AngularFrequency(20.0),
                DampingRatio(0.7),
            ),
            is_animating: false,
            current,
            target: Default::default(),
            velocity: Default::default(),
            on_complete: Rc::new(RefCell::new(|| {})),
        }
    }
    pub fn spring_to(
        mut self,
        register: &mut Vec<Self>,
        target: Mat4,
        on_complete: Rc<RefCell<dyn FnMut() -> ()>>,
    ) {
        self.is_animating = true;
        self.target = target;
        self.target = target;
        self.on_complete = on_complete;
        self.update();
        register.push(self);
    }

    pub fn update(&mut self) -> bool {
        let mut current_position_vec = vec![];
        let mut vel_vec = vec![];

        let animating_complete = {
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
            *self.current = Mat4::from_cols_array(&current_position_vec.try_into().unwrap());
            self.velocity = Mat4::from_cols_array(&vel_vec.try_into().unwrap());

            self.current.abs_diff_eq(self.target, 1.0)
                && self.velocity.abs_diff_eq(Mat4::ZERO, 100.0)
        };
        if animating_complete {
            self.is_animating = false;
            let mut x = self.on_complete.try_borrow_mut().unwrap();
            x()
        }
        animating_complete
    }
}
