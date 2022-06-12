use usvg::Node;

#[derive(PartialEq, PartialOrd, Eq, Ord, Clone, Copy, Debug)]
pub enum IndicesPriority {
    Fixed,
    Variable,
}

pub type OnClickCallback<'a> = Callback<'a, (), ()>;

pub struct Initialization<'a> {
    pub indices_priority: IndicesPriority,
    pub on_click_callback: OnClickCallback<'a>,
}
impl Default for Initialization<'_> {
    fn default() -> Self {
        return Self {
            indices_priority: IndicesPriority::Variable,
            on_click_callback: OnClickCallback::new(|_| {}),
        };
    }
}

pub type InitCallback<'a> = Callback<'a, Node, Initialization<'a>>;

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
