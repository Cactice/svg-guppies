use glam::DVec2;

type Line = (DVec2, DVec2);

// Offsets line by width/2 in both perpendicular direction to create parallel lines
pub fn line_to_parallel_lines(line: Line, width: f64) -> (Line, Line) {
    let perp_unit = (line.1 - line.0).perp().normalize();
    let perp = perp_unit * width / 2.0;
    let perp_negative = perp_unit * width / -2.0;
    let line0: Line = (line.0 + perp, line.1 + perp);
    let line1: Line = (line.0 + perp_negative, line.1 + perp_negative);
    (line0, line1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_to_box_is_correct() {
        let line: Line = (DVec2::new(0.0, 0.0), DVec2::new(0.0, 1.0));
        assert_eq!(
            (
                (DVec2::new(-0.5, 0.0), DVec2::new(-0.5, 1.0),),
                (DVec2::new(0.5, 0.0), DVec2::new(0.5, 1.0))
            ),
            line_to_parallel_lines(line, 1.0)
        );
    }
}
