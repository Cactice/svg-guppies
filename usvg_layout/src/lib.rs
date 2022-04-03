pub mod iterator;

use futures::channel::mpsc::Sender;
use iterator::{iterate, Index, Vertex};
use std::fs;
use std::{f64::NAN, path::Path};
use usvg::{NodeExt, XmlOptions};

pub type Vertices = Vec<Vertex>;
pub type Indices = Vec<Index>;
pub type DrawPrimitves = (Vertices, Indices);

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

pub fn init() -> DrawPrimitves {
    // Grab some parameters from the command line.

    env_logger::init();

    let msaa_samples = 4;

    // Parse and tessellate the geometry

    let filename = Path::new("/Users/yuya/git/gpu-gui/svg/Resting.svg");

    let mut opt = usvg::Options::default();
    opt.fontdb.load_system_fonts();
    let file_data = std::fs::read(filename).unwrap();
    let rtree = usvg::Tree::from_data(&file_data, &opt.to_ref()).unwrap();
    let str = rtree.to_string(&XmlOptions::default());
    // fs::write("./out2.svg", str).expect("Unable to write file");

    let mut prev_transform = usvg::Transform {
        a: NAN,
        b: NAN,
        c: NAN,
        d: NAN,
        e: NAN,
        f: NAN,
    };
    let view_box = rtree.svg_node().view_box;
    let mut vertices: Vec<Vertex> = vec![];
    let mut indices: Vec<Index> = vec![];
    for node in rtree.root().descendants() {
        if let usvg::NodeKind::Path(ref p) = *node.borrow() {
            // if let Some(ref fill) = p.fill {
            //     // fall back to always use color fill
            //     // no gradients (yet?)
            //     let color = match fill.paint {
            //         usvg::Paint::Color(c) => c,
            //         _ => FALLBACK_COLOR,
            //     };
            // }

            // if let Some(ref stroke) = p.stroke {}
            let (path_vertices, path_indices) = iterate(&p.data, 2.0);
            vertices.extend(path_vertices);
            indices.extend(path_indices);
        }
    }
    (vertices, indices)
}
