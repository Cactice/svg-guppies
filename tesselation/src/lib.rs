pub mod callback;
mod convert_path;
mod fill;
pub mod geometry;
mod stroke;
use callback::Callback;
use geometry::SvgSet;
pub use glam;
use glam::DMat4;

pub use usvg;

struct TransformVariable {
    transform: DMat4,
    transform_index: u16,
}

pub fn init(callback: Callback) -> SvgSet {
    // Parse and tessellate the geometry

    SvgSet::new(include_str!("../../svg/life_text.svg"), callback)
}
