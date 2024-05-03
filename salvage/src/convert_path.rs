use lyon::{math::Point, path::PathEvent};
use usvg::{tiny_skia_path::PathSegment, Path};
fn point(x: &f64, y: &f64) -> Point {
    Point::new((*x) as f32, (*y) as f32)
}
pub struct PathConvIter<'a> {
    iter: std::slice::Iter<'a, usvg::Path>,
    prev: Point,
    first: Point,
    needs_end: bool,
    deferred: Option<PathEvent>,
}

impl<'l> Iterator for PathConvIter<'l> {
    type Item = PathEvent;
    fn next(&mut self) -> Option<PathEvent> {
        if self.deferred.is_some() {
            return self.deferred.take();
        }

        if let Some(next) = self.iter.next() {
            match next. {
                Some(PathSegment::MoveTo(Point { x, y, .. })) => {
                    if self.needs_end {
                        let last = self.prev;
                        let first = self.first;
                        self.needs_end = false;
                        self.prev = point(x, y);
                        self.deferred = Some(PathEvent::Begin { at: self.prev });
                        self.first = self.prev;
                        Some(PathEvent::End {
                            last,
                            first,
                            close: false,
                        })
                    } else {
                        self.first = point(x, y);
                        self.needs_end = true;
                        Some(PathEvent::Begin { at: self.first })
                    }
                }
                Some(PathSegment::LineTo { x, y }) => {
                    self.needs_end = true;
                    let from = self.prev;
                    self.prev = point(x, y);
                    Some(PathEvent::Line {
                        from,
                        to: self.prev,
                    })
                }
                Some(usvg::PathSegment::CurveTo {
                    x1,
                    y1,
                    x2,
                    y2,
                    x,
                    y,
                }) => {
                    self.needs_end = true;
                    let from = self.prev;
                    self.prev = point(x, y);
                    Some(PathEvent::Cubic {
                        from,
                        ctrl1: point(x1, y1),
                        ctrl2: point(x2, y2),
                        to: self.prev,
                    })
                }
                Some(usvg::PathSegment::ClosePath) => {
                    self.needs_end = false;
                    self.prev = self.first;
                    Some(PathEvent::End {
                        last: self.prev,
                        first: self.first,
                        close: true,
                    })
                }
                None => {
                    if self.needs_end {
                        self.needs_end = false;
                        let last = self.prev;
                        let first = self.first;
                        Some(PathEvent::End {
                            last,
                            first,
                            close: false,
                        })
                    } else {
                        None
                    }
                }
            }
        }
    }
}

pub fn convert_path(p: &usvg::Path) -> PathConvIter {
    PathConvIter {
        iter: p.data.iter(),
        first: Point::new(0.0, 0.0),
        prev: Point::new(0.0, 0.0),
        deferred: None,
        needs_end: false,
    }
}
