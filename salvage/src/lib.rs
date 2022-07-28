pub mod callback;
mod convert_path;
mod fill;
pub mod geometry;
mod stroke;
use callback::InitCallback;
use geometry::SvgSet;
pub use glam;
use glam::DMat4;

pub use usvg;

struct TransformVariable {
    transform: DMat4,
    transform_id: u16,
}

pub fn init(callback: InitCallback) -> SvgSet {
    // Parse and tessellate the geometry

    let mut s = SvgSet::new(include_str!("../../svg/life.svg"), callback);
    s.update_text(
        &"Player3 #dynamicText".to_string(),
        &"Player3: $1000a dkfj".to_string(),
    );
    s
}
