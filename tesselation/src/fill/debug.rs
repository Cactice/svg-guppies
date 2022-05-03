use std::hash::{BuildHasher, Hasher};

use glam::Vec3;
pub(crate) fn rand_f32() -> f32 {
    (std::collections::hash_map::RandomState::new()
        .build_hasher()
        .finish()
        % u8::MAX as u64) as f32
        / u8::MAX as f32
}

pub(crate) fn rand_Vec3() -> Vec3 {
    Vec3::new(rand_f32(), rand_f32(), rand_f32())
}
