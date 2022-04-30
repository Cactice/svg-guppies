use glam::DVec2;

pub fn is_convex(polygon: Vec<DVec2>) -> bool {
    if polygon.len() < 3 {
        return false;
    }
    let N = polygon.len() - 1;

    let mut wSign = 0.0; // First nonzero orientation (positive or negative)

    let mut xSign = 0.0;
    let mut xFirstSign = 0.0; // Sign of first nonzero edge vector x
    let mut xFlips = 0.0; // Number of sign changes in x

    let mut ySign = 0.0;
    let mut yFirstSign = 0.0; // Sign of first nonzero edge vector y
    let mut yFlips = 0.0; // Number of sign changes in y

    let curr = polygon[N - 1]; // Second-to-last vertex
    let next = polygon[N]; // Last vertex

    for v in polygon {
        // Each vertex, in order
        let prev = curr; // Previous vertex
        let curr = next; // Current vertex
        let next = v; // Next vertex

        // Previous edge vector ("before"):
        let bx = curr.x - prev.x;
        let by = curr.y - prev.y;

        // Next edge vector ("after"):
        let ax = next.x - curr.x;
        let ay = next.y - curr.y;

        // Calculate sign flips using the next edge vector ("after"),
        // recording the first sign.
        if ax > 0.0 {
            if xSign == 0.0 {
                xFirstSign = 1.0;
            } else if xSign < 0.0 {
                xFlips += 1.0
            }
            xSign = 1.0
        } else if ax < 0.0 {
            if xSign == 0.0 {
                xFirstSign = -1.0
            } else if xSign > 0.0 {
                xFlips += 1.0
            }
            xSign = -1.0
        }

        if xFlips > 2.0 {
            return false;
        }

        if ay > 0.0 {
            if ySign == 0.0 {
                yFirstSign = 1.0
            } else if ySign < 0.0 {
                yFlips += 1.0
            }
            ySign = 1.0
        } else if ay < 0.0 {
            if ySign == 0.0 {
                yFirstSign = -1.0
            } else if ySign > 0.0 {
                yFlips += 1.0
            }
            ySign = -1.0
        }

        if yFlips > 2.0 {
            return false;
        }

        // Find out the orientation of this pair of edges,
        // && ensure it does not differ from previous ones.
        let w = bx * ay - ax * by;
        if (wSign == 0.0) && (w != 0.0) {
            wSign = w
        } else if (wSign > 0.0) && (w < 0.0) {
            return false;
        } else if (wSign < 0.0) && (w > 0.0) {
            return false;
        }
    }
    if (xSign != 0.0) && (xFirstSign != 0.0) && (xSign != xFirstSign) {
        xFlips += 1.0
    }
    if (ySign != 0.0) && (yFirstSign != 0.0) && (ySign != yFirstSign) {
        yFlips += 1.0
    }
    // Concave polygons have two sign flips along each axis.
    if (xFlips != 2.0) || (yFlips != 2.0) {
        return false;
    }

    // This is a convex polygon.
    true
}

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
