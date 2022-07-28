use guppies::callback::Callback;
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
        Self {
            indices_priority: IndicesPriority::Variable,
        }
    }
}

pub type InitCallback<'a> = Callback<'a, Node, Initialization>;
pub type OnClickCallback<'a> = Callback<'a, Node, Initialization>;
