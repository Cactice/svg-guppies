use crate::{
    convert_path::convert_path,
    geometry::{Index, Vertex},
};
use glam::Vec4;
use lyon::lyon_tessellation::{
    BuffersBuilder, FillOptions, FillTessellator, FillVertex, VertexBuffers,
};
use usvg::{self, Path};

pub fn iterate_fill(path: &Path, color: &Vec4, geometry: &mut VertexBuffers<Vertex, Index>) {
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
}
