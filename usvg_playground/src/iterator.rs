use usvg::{self, Color, NodeExt, PathData, PathSegment, XmlOptions};

type Index = u16;
pub fn iterate(path: PathData, vb: usvg::ViewBox) -> (Vec<Vertex>, Vec<Index>) {
    let mut vertices: Vec<Vertex> = vec![];
    let mut indices: Vec<Index> = vec![];
    path.iter().for_each(|path| match path {
        PathSegment::MoveTo { x, y } => {
            vertices.push(Vertex::from_xy(*x as f32, *y as f32));
            indices.push(vertices.len() as u16 - 1)
        }
        PathSegment::LineTo { x, y } => {
            vertices.push(Vertex::from_xy(*x as f32, *y as f32));
            indices.push(vertices.len() as u16 - 1)
        }
        PathSegment::CurveTo {
            x1,
            y1,
            x2,
            y2,
            x,
            y,
        } => {
            vertices.push(Vertex::from_xy(*x as f32, *y as f32));
            indices.push(vertices.len() as u16 - 1)
        }
        PathSegment::ClosePath => todo!(),
    });
    return (todo!(), todo!());
}
#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}
impl Vertex {
    fn from_xy(x: f32, y: f32) -> Self {
        Self {
            position: [x, y, 0.0],
            ..Default::default()
        }
    }
}
