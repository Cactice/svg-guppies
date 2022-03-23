use std::{
    marker::PhantomData,
    ptr::NonNull,
    sync::{atomic, Arc},
};

use crate::concept::{Layout, Point, Presenter};
use enumflags2::bitflags;

#[bitflags]
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum TodoE {
    Goal,
    Done,
}

#[derive(Default)]
struct TodoS {
    todo: Todo,
}
trait TodoT {
    fn toTodo(&self) -> Todo;
    fn done(&self);
    fn forget(&self);
}
// impl TodoT<T> for TodoS<T> {
//     fn toTodo(&self) -> Todo {
//         Todo {
//             goal: self.todo.goal,
//             done: self.todo.done,
//         }
//     }
//     fn done(self: &TodoS) {
//         Todo::done(&mut self.todo);
//     }
//     fn forget(self: &TodoS) {
//         Todo::forget(&mut self.todo);
//     }
// }

struct TodoV<const T: usize> {
    goal: String,
    done: bool,
}

impl TodoS {
    fn done(self: &mut Self) {
        Todo::done(&mut self.todo);
    }
    fn forget(self: &mut Self) {
        Todo::forget(&mut self.todo);
    }
}

#[derive(Default)]
struct Todo {
    goal: String,
    done: bool,
}
impl Todo {
    fn done(self: &mut Self) {
        self.done = true
    }
    fn forget(self: &mut Self) {
        self.done = true;
        self.goal = "Forget it!".to_string();
    }
}
enum SvgID {
    Check,
}

#[repr(C)]
struct ArcInner<T: ?Sized> {
    strong: atomic::AtomicUsize,

    // the value usize::MAX acts as a sentinel for temporarily "locking" the
    // ability to upgrade weak pointers or downgrade strong ones; this is used
    // to avoid races in `make_mut` and `get_mut`.
    weak: atomic::AtomicUsize,

    data: T,
}
pub struct MyArc<T: ?Sized> {
    ptr: NonNull<ArcInner<T>>,
    phantom: PhantomData<ArcInner<T>>,
}
impl<T: ?Sized> MyArc<T> {
    unsafe fn from_inner(ptr: NonNull<ArcInner<T>>) -> Self {
        Self {
            ptr,
            phantom: PhantomData,
        }
    }
}
impl<T> MyArc<T> {
    #[inline]
    pub fn new(data: T) -> MyArc<T> {
        // Start the weak pointer count as 1 which is the weak pointer that's
        // held by all the strong pointers (kinda), see std/rc.rs for more info
        let x: Box<_> = Box::new(ArcInner {
            strong: atomic::AtomicUsize::new(1),
            weak: atomic::AtomicUsize::new(1),
            data,
        });
        unsafe { Self::from_inner(Box::leak(x).into()) }
    }
}
pub fn app() {
    let todo = Arc::new(Todo::default());
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
        memos: &[],
    };
}
