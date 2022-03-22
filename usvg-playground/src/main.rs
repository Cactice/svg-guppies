use std::{f64::NAN, fs};
use usvg::{self, NodeExt, XmlOptions};
pub const FALLBACK_COLOR: usvg::Color = usvg::Color {
    red: 0,
    green: 0,
    blue: 0,
};
fn main() {
    let mut prev_transform = usvg::Transform {
        a: NAN,
        b: NAN,
        c: NAN,
        d: NAN,
        e: NAN,
        f: NAN,
    };
    let filename = "Resting.svg";
    let file_data = std::fs::read(filename).unwrap();
    let mut opt = usvg::Options::default();

    opt.fontdb.load_system_fonts();
    let rtree = usvg::Tree::from_data(&file_data, &opt.to_ref()).unwrap();
    let str = rtree.to_string(&XmlOptions::default());
    fs::write("./out.svg", str).expect("Unable to write file");

    let mut transforms = Vec::new();
    let mut primitives = Vec::new();
    for node in rtree.root().descendants() {
        if let usvg::NodeKind::Path(ref p) = *node.borrow() {
            let t = node.transform();
            if t != prev_transform {
                transforms.push(GpuTransform {
                    data0: [t.a as f32, t.b as f32, t.c as f32, t.d as f32],
                    data1: [t.e as f32, t.f as f32, 0.0, 0.0],
                });
            }
            prev_transform = t;

            let transform_idx = transforms.len() as u32 - 1;

            if let Some(ref bbox) = p.cluster_bbox {
                println!("{:?}", bbox)
            }
            if let Some(ref fill) = p.fill {
                // fall back to always use color fill
                // no gradients (yet?)
                let color = match fill.paint {
                    usvg::Paint::Color(c) => c,
                    _ => FALLBACK_COLOR,
                };

                primitives.push(GpuPrimitive::new(
                    transform_idx,
                    color,
                    fill.opacity.value() as f32,
                ));
            }

            // if let Some(ref stroke) = p.stroke {
            // let (stroke_color, stroke_opts) = convert_stroke(stroke);
            // primitives.push(GpuPrimitive::new(
            //     transform_idx,
            //     stroke_color,
            //     stroke.opacity.value() as f32,
            // ));
            // }
        }
    }
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct GpuGlobals {
    pub zoom: [f32; 2],
    pub pan: [f32; 2],
    pub aspect_ratio: f32,
    pub _pad: f32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct GpuTransform {
    pub data0: [f32; 4],
    pub data1: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct GpuPrimitive {
    pub transform: u32,
    pub color: u32,
    pub _pad: [u32; 2],
}
impl GpuPrimitive {
    pub fn new(transform_idx: u32, color: usvg::Color, alpha: f32) -> Self {
        GpuPrimitive {
            transform: transform_idx,
            color: ((color.red as u32) << 24)
                + ((color.green as u32) << 16)
                + ((color.blue as u32) << 8)
                + (alpha * 255.0) as u32,
            _pad: [0; 2],
        }
    }
}

unsafe impl bytemuck::Pod for GpuGlobals {}
unsafe impl bytemuck::Zeroable for GpuGlobals {}
unsafe impl bytemuck::Pod for GpuPrimitive {}
unsafe impl bytemuck::Zeroable for GpuPrimitive {}
unsafe impl bytemuck::Pod for GpuTransform {}
unsafe impl bytemuck::Zeroable for GpuTransform {}
