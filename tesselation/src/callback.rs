use usvg::Node;

#[derive(PartialEq, PartialOrd, Eq, Ord, Clone, Copy, Debug)]
pub enum IndicesPriority {
    Fixed,
    Variable,
}

pub struct Initialization {
    pub indices_priority: IndicesPriority,
}
impl Default for Initialization {
    fn default() -> Self {
        return Self {
            indices_priority: IndicesPriority::Variable,
        };
    }
}

pub type InitCallback<'a> = Callback<'a, Node, Initialization>;
pub type OnClickCallback<'a> = Callback<'a, Node, Initialization>;

pub struct Callback<'a, A, T> {
    func: Box<dyn FnMut(&A) -> T + 'a>,
}

impl<'a, A, T> Callback<'a, A, T> {
    pub fn new(c: impl FnMut(&A) -> T + 'a) -> Self {
        Self { func: Box::new(c) }
    }
    pub fn process_events(&mut self, arg: &A) -> T {
        (self.func)(arg)
    }
}
