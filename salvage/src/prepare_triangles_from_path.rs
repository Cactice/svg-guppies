use crate::{fill::iterate_fill, stroke::iterate_stroke};
use guppies::{
    glam::Vec4,
    primitives::{Index, Triangles, Vertex},
};
use lyon::lyon_tessellation::VertexBuffers;
use usvg::Path;
pub const FALLBACK_COLOR: Vec4 = Vec4::ONE;

pub fn prepare_triangles_from_path(p: &Path, transform_id: u32) -> Triangles {
    let mut vertex_buffer = VertexBuffers::<Vertex, Index>::new();
    if let Some(stroke) = &p.stroke {
        let color = match stroke.paint {
            usvg::Paint::Color(c) => Vec4::new(
                c.red as f32 / u8::MAX as f32,
                c.green as f32 / u8::MAX as f32,
                c.blue as f32 / u8::MAX as f32,
                stroke.opacity.value() as f32,
            ),
            _ => FALLBACK_COLOR,
        };
        iterate_stroke(stroke, p, &mut vertex_buffer, color, transform_id);
    }
    if let Some(fill) = &p.fill {
        let color = match fill.paint {
            usvg::Paint::Color(c) => Vec4::new(
                c.red as f32 / u8::MAX as f32,
                c.green as f32 / u8::MAX as f32,
                c.blue as f32 / u8::MAX as f32,
                fill.opacity.value() as f32,
            ),
            _ => FALLBACK_COLOR,
        };

        iterate_fill(p, &color, &mut vertex_buffer, transform_id);
    };
    Triangles {
        vertices: vertex_buffer.vertices,
        indices: vertex_buffer.indices,
    }
}
