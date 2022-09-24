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
