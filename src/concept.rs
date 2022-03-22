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

pub type Layout<'a, D, SvgIDs, Label = Area> = (D, SvgIDs, &'a dyn FnMut(Point, Label) -> Point);
pub type Memo<'a, D> = (D, &'a dyn FnMut());
pub type Callback<'a, D> = &'a dyn FnMut() -> D;
// S: State, D: Diff
#[derive(Default)]
pub struct Presenter<'a, D, SvgIDs, Label = Area> {
    pub layouts: &'a [Layout<'a, D, SvgIDs, Label>],
    pub callbacks: &'a [Callback<'a, D>],
    pub memo: &'a [Memo<'a, D>],
}

pub type CharPoints = Points;
pub struct TextRenderer {
    pub text: String,
    pub line_height: i32,
    pub bbox: Rect,
    pub texts: Vec<String>,
    pub selected: bool,
    pub selected_range: [CharPoints; 2],
}

struct Initialization<'a, D, SvgIDs, Label = Area> {
    presenter: Presenter<'a, D, SvgIDs, Label>,
    svg: String,
    labeller: &'a [fn(Points, SvgIDs) -> [(Points, Label)]],
}
impl<'a, D, SvgIDs> Initialization<'a, D, SvgIDs> {}

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
