use std::ops::{Deref, DerefMut};

use enumflags2::bitflags;

use crate::concept::{Layout, Point, Presenter};

#[bitflags]
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum TodoE {
    Goal,
    Done,
}

#[derive(Default)]
struct Todo {
    goal: String,
    done: bool,
}
impl Todo {
    fn done(&mut self) {
        self.done = true
    }
    fn forget(&mut self) {
        self.done = true;
        self.goal = "Forget it!".to_string();
    }
}
enum SvgID {
    Check,
}

#[derive(Default)]
struct TodoP0 {
    count0: u8,
    ptr: TodoP1,
}
impl Deref for TodoP0 {
    type Target = TodoP1;
    fn deref(&self) -> &Self::Target {
        &self.ptr
    }
}
impl DerefMut for TodoP0 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.count0 += 1;
        &mut self.ptr
    }
}
#[derive(Default)]
struct TodoP1 {
    done: bool,
    count1: u8,
    ptr: TodoP2,
}
#[derive(Default)]
struct TodoP2 {
    goal: String,
}

impl Deref for TodoP1 {
    type Target = TodoP2;
    fn deref(&self) -> &Self::Target {
        &self.ptr
    }
}
impl DerefMut for TodoP1 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.count1 += 1;
        &mut self.ptr
    }
}

pub fn app() {
    let mut p0 = TodoP0::default();
    p0.done = true;
    p0.goal = "he".to_string();
    let on_check_box_click = &|| -> TodoE { TodoE::Done };
    let goal_change: Layout<TodoE, SvgID> = (TodoE::Goal, SvgID::Check, &|point, _area| -> Point {
        point
    });
    let _presenter: Presenter<TodoE, SvgID> = Presenter {
        layouts: &[goal_change],
        callbacks: &[on_check_box_click],
        precompute: &[],
    };
}
