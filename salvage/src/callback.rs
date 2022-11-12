#[derive(PartialEq, PartialOrd, Eq, Ord, Clone, Copy, Debug, Default)]
pub enum IndicesPriority {
    Variable,
    #[default]
    Fixed,
}

#[derive(PartialEq, PartialOrd, Eq, Ord, Clone, Copy, Debug)]
pub struct PassDown {
    pub indices_priority: IndicesPriority,
    pub transform_id: u32,
}

impl Default for PassDown {
    fn default() -> Self {
        Self {
            indices_priority: Default::default(),
            transform_id: 1,
        }
    }
}
