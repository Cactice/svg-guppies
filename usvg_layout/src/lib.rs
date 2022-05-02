pub mod iterator;

pub use glam;
use glam::Vec2;
use iterator::{fills::iterate_fill, iterate_stroke, Index, Vertex};
use std::path::Path;

pub type Vertices = Vec<Vertex>;
pub type Indices = Vec<Index>;
pub type DrawPrimitives = (Vertices, Indices);
pub type Size = Vec2;
pub type Position = Vec2;
pub type Rect = (Position, Size);

// This example renders a very tiny subset of SVG (only filled and stroke paths with solid color
// patterns and transforms).
//
// Parsing is done via the usvg crate. In this very simple example, paths are all tessellated directly
// into a static mesh during parsing.
// vertices embed a primitive ID which lets the vertex shader fetch the per-path information such like
// the color from uniform buffer objects.
// No occlusion culling optimization here (see the wgpu example).
//
// Most of the code in this example is related to working with the GPU.

pub const FALLBACK_COLOR: usvg::Color = usvg::Color {
    red: 0,
    green: 0,
    blue: 0,
};
pub fn init() -> (DrawPrimitives, Rect) {
    // Parse and tessellate the geometry

    let filename = Path::new("/Users/yuya/git/gpu-gui/svg/Resting.svg");

    let mut opt = usvg::Options::default();
    opt.fontdb.load_system_fonts();
    let file_data = std::fs::read(filename).unwrap();
    let rtree = usvg::Tree::from_data(&file_data, &opt.to_ref()).unwrap();

    let view_box = rtree.svg_node().view_box;
    let rect: Rect = (
        Vec2::new(view_box.rect.x() as f32, view_box.rect.y() as f32),
        Vec2::new(view_box.rect.width() as f32, view_box.rect.height() as f32),
    );

    let mut vertices: Vec<Vertex> = vec![];
    let mut indices: Vec<Index> = vec![];
    for node in rtree.root().descendants() {
        if let usvg::NodeKind::Path(ref p) = *node.borrow() {
            if let Some(ref stroke) = p.stroke {
                let (path_vertices, path_indices) = iterate_stroke(&p.data, stroke.width.value());
                vertices.extend(path_vertices);
                indices.extend(path_indices);
            }
            if let Some(ref fill) = p.fill {
                let color = match fill.paint {
                    usvg::Paint::Color(c) => c,
                    _ => FALLBACK_COLOR,
                };
                let (path_vertices, path_indices) = iterate_fill(&p.data, &color);
                vertices.extend(path_vertices);
                indices.extend(path_indices);
            }
        }
    }
    ((vertices, indices), rect)
}
