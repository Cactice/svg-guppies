use crate::convert_path::convert_path;
use guppies::{
    glam::Vec4,
    primitives::{Index, Vertex},
};
use lyon::lyon_tessellation::{
    BuffersBuilder, FillOptions, FillTessellator, FillVertex, VertexBuffers,
};
use usvg::{self, Path};

pub fn iterate_fill(
    path: &Path,
    color: &Vec4,
    geometry: &mut VertexBuffers<Vertex, Index>,
    id: u32,
) {
    let mut fill_tess = FillTessellator::new();
    fill_tess
        .tessellate(
            convert_path(path),
            &FillOptions::tolerance(0.01),
            &mut BuffersBuilder::new(geometry, |v: FillVertex| Vertex {
                position: [v.position().x, v.position().y, 0.],
                color: color.to_array(),
                transform_id: id,
            }),
        )
        .expect("Error during tesselation!");
}
