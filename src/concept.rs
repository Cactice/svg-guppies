use enumflags2::bitflags;

pub struct Rect {
    width: i32,
    height: i32,
    center: i32,
    min_x: i32,
    max_x: i32,
    min_y: i32,
    max_y: i32,
}

#[derive(Default)]
pub struct Point {
    x: i32,
    y: i32,
}
pub type Points = Vec<Point>;

// S: State, D: Diff
#[derive(Default)]
pub struct Presenter<'a, S, D, SvgIDs, Labels = Area> {
    pub layouts: &'a [(D, fn(&S, (SvgIDs, Labels)) -> Points)],
    pub callbacks: &'a [fn() -> (S, D)],
}

pub struct TextRenderer {
    pub text: String,
    pub line_height: i32,
    pub bbox: Rect,
    pub texts: Vec<String>,
}

struct Initialization<'a, S, D, SvgIDs, Labels = Area> {
    presenter: Presenter<'a, S, D, SvgIDs, Labels>,
    svg: String,
    labeller: &'a [fn(Points, SvgIDs) -> [(Points, Labels)]],
}

#[bitflags]
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Area {
    T,
    B,
    L,
    R,
}

#[bitflags]
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq)]
enum windowE {
    width,
    height,
}
