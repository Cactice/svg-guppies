use crate::geometry::Geometry;
use guppies::callback::Callback;
use usvg::Node;

#[derive(PartialEq, PartialOrd, Eq, Ord, Clone, Copy, Debug, Default)]
pub enum IndicesPriority {
    Variable,
    #[default]
    Fixed,
}

#[derive(PartialEq, PartialOrd, Eq, Ord, Clone, Copy, Debug, Default)]
pub struct PassDown {
    pub indices_priority: IndicesPriority,
    pub transform_id: u32,
}

pub type InitCallback<'a> = Callback<'a, (Node, PassDown), (Option<Geometry>, PassDown)>;
