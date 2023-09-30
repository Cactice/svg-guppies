#[derive(PartialEq, PartialOrd, Eq, Ord, Clone, Copy, Debug)]
pub struct PassDown {
    pub transform_id: u32,
}

impl Default for PassDown {
    fn default() -> Self {
        Self { transform_id: 1 }
    }
}
