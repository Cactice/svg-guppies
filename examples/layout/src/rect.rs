use salvage::usvg::PathBbox;

#[derive(Copy, Clone, Default, Debug)]
pub struct MyRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
impl MyRect {
    pub fn right(&self) -> f32 {
        self.x + self.width
    }
    pub fn left(&self) -> f32 {
        self.x
    }
    pub fn x_center(&self) -> f32 {
        self.x + (self.width / 2.)
    }
    pub fn top(&self) -> f32 {
        self.y + self.height
    }
    pub fn bottom(&self) -> f32 {
        self.y
    }
    pub fn y_center(&self) -> f32 {
        self.y + (self.height / 2.)
    }
}
impl From<PathBbox> for MyRect {
    fn from(bbox: PathBbox) -> Self {
        Self {
            x: bbox.x() as f32,
            y: bbox.y() as f32,
            width: bbox.width() as f32,
            height: bbox.height() as f32,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum XConstraint {
    Left(f32),
    Right(f32),
    LeftAndRight { left: f32, right: f32 },
    Center { rightward_from_center: f32 },
    Scale,
}

impl Default for XConstraint {
    fn default() -> Self {
        Self::LeftAndRight {
            left: 0.,
            right: 0.,
        }
    }
}
pub enum YConstraint {
    Top(f32),
    Bottom(f32),
    TopAndBottom { top: f32, bottom: f32 },
    Center { downward_from_center: f32 },
    Scale,
}

impl Default for YConstraint {
    fn default() -> Self {
        Self::TopAndBottom {
            top: 0.,
            bottom: 0.,
        }
    }
}

pub struct Constraint {
    pub x: XConstraint,
    pub y: YConstraint,
}
