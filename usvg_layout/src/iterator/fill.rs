use glam::DVec2;
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rec_is_convex() {
        let rec: Vec<DVec2> = vec![[0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0]]
            .iter()
            .map(|v| DVec2::from(*v))
            .collect();
        assert!(is_convex(rec))
    }

    #[test]
    fn m_is_not_convex() {
        // the shape resembles the letter M
        let m: Vec<DVec2> = vec![[0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [0.5, 0.5], [1.0, 0.0]]
            .iter()
            .map(|v| DVec2::from(*v))
            .collect();
        assert!(!is_convex(m));
    }

    #[test]
    fn star_is_not_convex() {
        let star: Vec<DVec2> = vec![
            [1.0, 3.0],
            [9.0, 7.0],
            [7.0, 9.0],
            [7.0, 2.0],
            [9.0, 6.0],
            [1.0, 8.0],
        ]
        .iter()
        .map(|v| DVec2::from(*v))
        .collect();
        assert!(!is_convex(star))
    }
}

pub fn is_convex(polygon: Vec<DVec2>) -> bool {
    if polygon.len() < 3 {
        return false;
    }

    let mut w_sign = 0.0; // First nonzero orientation (positive or negative)

    let mut x_sign = 0.0;
    let mut x_flips = 0.0; // Number of sign changes in x

    let mut y_sign = 0.0;
    let mut y_flips = 0.0; // Number of sign changes in y

    let mut curr = polygon[polygon.len() - 2]; // Second-to-last vertex
    let mut next = polygon[polygon.len() - 1]; // Last vertex
    let x_first_sign = if next.x - curr.x > 0.0 { -1.0 } else { 1.0 }; // Sign of first nonzero edge vector x
    let y_first_sign = if next.y - curr.y > 0.0 { -1.0 } else { 1.0 }; // Sign of first nonzero edge vector y

    for v in polygon {
        // Each vertex, in order
        let prev = curr; // Previous vertex
        curr = next; // Current vertex
        next = v; // Next vertex

        // Previous edge vector ("before"):
        let bx = curr.x - prev.x;
        let by = curr.y - prev.y;

        // Next edge vector ("after"):
        let ax = next.x - curr.x;
        let ay = next.y - curr.y;

        let next_x_sign = if ax > 0.0 { 1.0 } else { -1.0 };
        x_flips += if x_sign * -next_x_sign < 0.0 {
            1.0
        } else {
            0.0
        };
        x_sign = next_x_sign;

        if x_flips > 2.0 {
            return false;
        }

        let next_y_sign = if ay > 0.0 { 1.0 } else { -1.0 };
        y_flips += if y_sign * -next_y_sign < 0.0 {
            1.0
        } else {
            0.0
        };
        y_sign = next_y_sign;

        if y_flips > 2.0 {
            return false;
        }

        // Find out the orientation of this pair of edges,
        // && ensure it does not differ from previous ones.
        let w = bx * ay - ax * by;
        if (w_sign == 0.0) && (w != 0.0) {
            w_sign = w
        } else if (w_sign > 0.0 && w < 0.0) {
            return false;
        } else if (w_sign < 0.0 && w > 0.0) {
            return false;
        }
    }
    if (x_sign != 0.0) && (x_first_sign != 0.0) && (x_sign != x_first_sign) {
        x_flips += 1.0
    }
    if (y_sign != 0.0) && (y_first_sign != 0.0) && (y_sign != y_first_sign) {
        y_flips += 1.0
    }
    // Concave polygons have two sign flips along each axis.
    if (x_flips != 2.0) || (y_flips != 2.0) {
        return false;
    }

    // This is a convex polygon.
    true
}
