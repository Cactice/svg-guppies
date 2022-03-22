use std::{collections::HashMap, hash::Hash};

use enumflags2::bitflags;

#[derive(Default)]
pub struct Rect {
    width: i32,
    height: i32,
    x: i32,
    y: i32,
}

#[derive(Default, Copy, Clone)]
pub struct Point {
    x: i32,
    y: i32,
}
pub type Points = Vec<Point>;

#[derive(Copy, Clone)]
pub enum PathSegment {
    MoveTo {
        x: f64,
        y: f64,
    },
    LineTo {
        x: f64,
        y: f64,
    },
    CurveTo {
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        x: f64,
        y: f64,
    },
    ClosePath,
}
pub struct PathData(pub Vec<PathSegment>);

impl PathData {
    fn get_bbox(&self) -> Rect {
        Rect::default()
    }
}

pub type Layout<'a, D, SvgID, Label = Area> = (D, SvgID, &'a dyn FnMut(Point, Label) -> Point);
pub type Memo<'a, D> = (D, &'a dyn FnMut());
pub type Callback<'a, D> = &'a dyn FnMut() -> D;
pub type Labeller<'a, SvgID, Label = Area> = &'a fn(Points, SvgID) -> [(Points, Label)];
#[derive(Default)]
// S: State, D: Diff
pub struct Presenter<'a, D, SvgID, Label = Area> {
    pub layouts: &'a [Layout<'a, D, SvgID, Label>],
    pub callbacks: &'a [Callback<'a, D>],
    pub memos: &'a [Memo<'a, D>],
}

impl<'a, D, SvgID> Presenter<'a, D, SvgID> {}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
enum Attribute {
    ClickableBBox,
}

type Attributes<SvgID> = HashMap<Attribute, Vec<SvgID>>;
type SvgPaths<SvgID> = HashMap<SvgID, PathData>;
pub type CharPoints = Points;

#[derive(Default)]
struct Component<'a, D: Copy, SvgID: Hash + Eq + Clone + Copy, Label = Area> {
    presenter: Presenter<'a, D, SvgID, Label>,
    svg: String,
    labellers: &'a [Labeller<'a, SvgID, Label>],
    attributes: Attributes<SvgID>,
    svg_paths: SvgPaths<SvgID>,
}

fn is_point_in_rect(rect: Rect, point: Point) -> bool {
    rect.x < point.x
        && rect.y < point.y
        && point.x < rect.x + rect.width
        && point.y < rect.y + rect.height
}
impl<'a, D: Copy, SvgID: Hash + Eq + Clone + Copy> Component<'a, D, SvgID> {
    fn click(&self, point: Point) {
        if let Some(clickable) = self.attributes.get(&Attribute::ClickableBBox) {
            clickable.iter().for_each(|id| {
                if let Some(path) = self.svg_paths.get(id) {
                    is_point_in_rect(path.get_bbox(), point);
                }
            })
        }
    }
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

pub struct TextRenderer {
    pub text: String,
    pub line_height: i32,
    pub bbox: Rect,
    pub texts: Vec<String>,
    pub selected: bool,
    pub selected_range: [CharPoints; 2],
}
