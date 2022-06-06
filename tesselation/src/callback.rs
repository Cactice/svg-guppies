use usvg::Node;

#[derive(PartialEq, PartialOrd, Eq, Ord, Clone, Copy, Debug)]
pub enum IndicesPriority {
    Fixed,
    Variable,
}

pub type OnClickCallback<'a> = Callback<'a, (), ()>;

pub struct Initialization<'a> {
    pub indicesPriority: IndicesPriority,
    pub onClickCallBack: OnClickCallback<'a>,
}
impl Default for Initialization<'_> {
    fn default() -> Self {
        return Self {
            indicesPriority: IndicesPriority::Variable,
            onClickCallBack: OnClickCallback::new(|_| {}),
        };
    }
}

pub type InitCallback<'a> = Callback<'a, Initialization<'a>, Node>;

pub struct Callback<'a, T, S> {
    func: Box<dyn FnMut(&S) -> T + 'a>,
}

impl<'a, T: Default, S> Callback<'a, T, S> {
    pub fn new(c: impl FnMut(&S) -> T + 'a) -> Self {
        Self { func: Box::new(c) }
    }
    pub fn process_events(&mut self, arg: &S) -> T {
        (self.func)(arg)
    }
}
