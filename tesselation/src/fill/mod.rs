mod convex_breakdown;
#[allow(dead_code)]
mod debug;
mod get_first_convex_index;
mod triangulate;

use crate::{convert_path::convert_path, Index, Vertex};
use glam::Vec4;
use lyon::lyon_tessellation::{
    BuffersBuilder, FillOptions, FillTessellator, FillVertex, VertexBuffers,
};
use usvg::{self, Path};

pub fn iterate_fill(
    path: &Path,
    color: &Vec4,
    geometry: &mut VertexBuffers<Vertex, Index>,
) -> (Vec<Vertex>, Vec<Index>) {
    let mut fill_tess = FillTessellator::new();
    fill_tess
        .tessellate(
            convert_path(path),
            &FillOptions::tolerance(0.01),
            &mut BuffersBuilder::new(geometry, |vertex: FillVertex| {
                Vertex::from((&vertex, color))
            }),
        )
        .expect("Error during tesselation!");
    ((*geometry.vertices).to_vec(), (*geometry.indices).to_vec())
}
