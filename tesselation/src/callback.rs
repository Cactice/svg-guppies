use usvg::Node;

#[derive(PartialEq, PartialOrd, Eq, Ord, Clone, Copy, Debug)]
pub enum IndicesPriority {
    Fixed,
    Variable,
}
pub struct Callback<'a> {
    func: Box<dyn FnMut(&Node) -> IndicesPriority + 'a>,
}

impl<'a> Callback<'a> {
    pub fn new(c: impl FnMut(&Node) -> IndicesPriority + 'a) -> Self {
        Self { func: Box::new(c) }
    }
    pub fn process_events(&mut self, node: &Node) -> IndicesPriority {
        (self.func)(node)
    }
}
