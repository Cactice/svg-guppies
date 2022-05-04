use glam::DVec2;

fn process_axis(a: &f64, flips: &mut i32, sign: &mut i32, first_sign: &mut i32) {
    if *a == 0.0 {
        return;
    }
    let next_sign = if *a > 0.0 { 1 } else { -1 };
    *flips += if *sign * next_sign < 0 { 1 } else { 0 };
    *sign = next_sign;
    if *first_sign == 0 {
        *first_sign = next_sign
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn triangle_is_convex() {
        let triangle: Vec<DVec2> = vec![[0.0, 0.0], [0.0, 1.0], [1.0, 1.0]]
            .iter()
            .map(|v| DVec2::from(*v))
            .collect();
        assert_eq!(get_first_convex_index(&triangle).0, triangle.len() - 1)
    }
    #[test]
    fn rec_is_convex() {
        let rec: Vec<DVec2> = vec![[0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0]]
            .iter()
            .map(|v| DVec2::from(*v))
            .collect();
        assert_eq!(get_first_convex_index(&rec).0, rec.len() - 1)
    }

    #[test]
    fn house_is_convex() {
        // the shape resembles a house
        let house: Vec<DVec2> = vec![[0.0, 0.0], [0.0, 1.0], [0.5, 1.5], [1.0, 1.0], [1.0, 0.0]]
            .iter()
            .map(|v| DVec2::from(*v))
            .collect();
        assert_eq!(get_first_convex_index(&house).0, house.len() - 1)
    }
    #[test]
    fn m_is_concave() {
        // the shape resembles the letter M
        let m: Vec<DVec2> = vec![[0.0, 0.0], [0.0, 1.0], [0.5, 0.5], [1.0, 1.0], [1.0, 0.0]]
            .iter()
            .map(|v| DVec2::from(*v))
            .collect();
        assert_ne!(get_first_convex_index(&m).0, m.len())
    }

    #[test]
    fn star_is_concave() {
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
        assert_ne!(get_first_convex_index(&star).0, star.len())
    }
}
// Inspiration: https://math.stackexchange.com/questions/1743995/determine-whether-a-polygon-is-convex-based-on-its-vertices/1745427#1745427
pub(crate) fn get_first_convex_index(polygon: &Vec<DVec2>) -> (usize, f64) {
    if polygon.len() < 3 {
        return (0, 0.0);
    }
    let n = polygon.len() - 1;

    let mut w_sign = 0.0; // First nonzero orientation (positive or negative)

    let mut x_sign = 0;
    let mut x_first_sign = 0; // Sign of first nonzero edge vector x
    let mut x_flips = 0; // Number of sign changes in x

    let mut y_sign = 0;
    let mut y_first_sign = 0; // Sign of first nonzero edge vector y
    let mut y_flips = 0; // Number of sign changes in y

    let mut curr = polygon[n - 1]; // Second-to-last vertex
    let mut next = polygon[n]; // Last vertex

    for (i, v) in polygon.iter().enumerate() {
        // Each vertex, in order
        let prev = curr; // Previous vertex
        curr = next; // Current vertex
        next = *v; // Next vertex

        // Previous edge vector ("before"):
        let bx = curr.x - prev.x;
        let by = curr.y - prev.y;

        // Next edge vector ("after"):
        let ax = next.x - curr.x;
        let ay = next.y - curr.y;
        process_axis(&ax, &mut x_flips, &mut x_sign, &mut x_first_sign);

        if x_flips > 2 {
            return (i, w_sign);
        }

        process_axis(&ay, &mut y_flips, &mut y_sign, &mut y_first_sign);

        if y_flips > 2 {
            return (i, w_sign);
        }

        // Find out the orientation of this pair of edges,
        // and ensure it does not differ from previous ones.
        let w = bx * ay - ax * by;
        if w_sign == 0.0 && w != 0.0 {
            w_sign = w
        } else if (w_sign > 0.0 && w < 0.0) || (w_sign < 0.0 && w > 0.0) {
            return (i, w_sign);
        }
    }
    if x_sign != 0 && x_first_sign != 0 && x_sign != x_first_sign {
        x_flips += 1
    }
    if y_sign != 0 && y_first_sign != 0 && y_sign != y_first_sign {
        y_flips += 1
    }
    // Concave polygons have two sign flips along each axis.
    if x_flips != 2 || y_flips != 2 {
        // todo: what to do in this scenario..?
        return (n, w_sign);
    }

    // This is a convex polygon.
    (n, w_sign)
}
