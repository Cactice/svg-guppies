use crate::convert_path::convert_path;
use guppies::glam::Vec4;
use guppies::primitives::{Index, Vertex};
use lyon::lyon_tessellation::{StrokeOptions, StrokeTessellator, StrokeVertex};
use lyon::tessellation::geometry_builder::*;
use lyon::tessellation::{self};
use usvg::{self, Path};

pub fn convert_stroke(s: &usvg::Stroke) -> StrokeOptions {
    let linecap = match s.linecap {
        usvg::LineCap::Butt => tessellation::LineCap::Butt,
        usvg::LineCap::Square => tessellation::LineCap::Square,
        usvg::LineCap::Round => tessellation::LineCap::Round,
    };
    let linejoin = match s.linejoin {
        usvg::LineJoin::Miter => tessellation::LineJoin::Miter,
        usvg::LineJoin::Bevel => tessellation::LineJoin::Bevel,
        usvg::LineJoin::Round => tessellation::LineJoin::Round,
    };

    StrokeOptions::tolerance(0.01)
        .with_line_width(s.width.value() as f32)
        .with_line_cap(linecap)
        .with_line_join(linejoin)
}

pub fn iterate_stroke(
    s: &usvg::Stroke,
    path: &Path,
    geometry: &mut VertexBuffers<Vertex, Index>,
    color: Vec4,
    id: u32,
) {
    let mut stroke_tess = StrokeTessellator::new();
    let stroke_opts = convert_stroke(s);
    let _ = stroke_tess.tessellate(
        convert_path(path),
        &stroke_opts,
        &mut BuffersBuilder::new(geometry, |v: StrokeVertex| {
            let position = v.position();
            let (x, y) = path.transform.apply(position.x as f64, position.y as f64);
            Vertex {
                position: [x as f32, y as f32, 0.],
                color: color.to_array(),
                transform_id: id,
            }
        }),
    );
}
