use glam::{DVec2, Vec2, Vec4};

pub type Index = u32;
pub type Vertices = Vec<Vertex>;
pub type Indices = Vec<Index>;
pub type DrawPrimitives = (Vertices, Indices);
pub type Size = Vec2;
pub type Position = Vec2;

#[derive(Copy, Clone, Debug, Default)]
pub struct Rect {
    pub position: Position,
    pub size: Size,
}
impl Rect {
    pub fn new(position: Vec2, size: Vec2) -> Self {
        Self { position, size }
    }
    pub fn contains_point(self, position: &Vec2) -> bool {
        if self.position.x < position.x
            && self.position.y < position.y
            && position.x < self.position.x + self.size.x
            && position.y < self.position.y + self.size.y
        {
            return true;
        }
        false
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub transform_id: u32,
    pub color: [f32; 4],
}
impl From<&DVec2> for Vertex {
    fn from(v: &DVec2) -> Self {
        Self {
            position: [(v.x) as f32, (v.y) as f32, 0.0],
            ..Default::default()
        }
    }
}

impl From<(&DVec2, &Vec4)> for Vertex {
    fn from((v, c): (&DVec2, &Vec4)) -> Self {
        Self {
            position: [(v.x) as f32, (v.y) as f32, 0.0],
            color: [c.x, c.y, c.z, c.w],
            ..Default::default()
        }
    }
}
impl From<(&DVec2, &Vec4, u32)> for Vertex {
    fn from((v, c, transform_id): (&DVec2, &Vec4, u32)) -> Self {
        Self {
            position: [(v.x) as f32, (v.y) as f32, 0.0],
            color: [c.x, c.y, c.z, c.w],
            transform_id,
        }
    }
}
