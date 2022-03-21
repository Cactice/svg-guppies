use crate::concept::{Area, Point, Points, Presenter, Rect};
use enumflags2::bitflags;

#[bitflags]
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq)]
enum TodoE {
    goal,
    done,
}

#[derive(Default)]
struct Todo {
    goal: String,
    done: bool,
}
enum SvgIDs {
    rect(Rect),
}

fn goalChange(todo: &Todo, selection: (SvgIDs, Area)) -> Points {
    vec![Point::default()]
}

fn onCheckBoxClick() -> (Todo, TodoE) {
    let todo = Todo::default();
    (todo, TodoE::done)
}

fn func() {
    const presenter: Presenter<Todo, TodoE, SvgIDs> = Presenter {
        layouts: &[(TodoE::goal, goalChange)],
        callbacks: &[onCheckBoxClick],
    };
}
