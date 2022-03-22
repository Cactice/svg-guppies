use crate::concept::{Layout, Point, Presenter};
use enumflags2::bitflags;

#[bitflags]
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq)]
enum TodoE {
    Goal,
    Done,
}

#[derive(Default)]
struct Todo {
    goal: String,
    done: bool,
}
enum SvgID {
    Check,
}

pub fn app() {
    let mut todo = Todo::default();
    let on_check_box_click = &|| -> TodoE {
        todo.done = true;
        TodoE::Done
    };
    let goal_change: Layout<TodoE, SvgID> = (TodoE::Goal, SvgID::Check, &|point, _area| -> Point {
        point
    });
    let _presenter: Presenter<TodoE, SvgID> = Presenter {
        layouts: &[goal_change],
        callbacks: &[on_check_box_click],
        memo: &[],
    };
}
