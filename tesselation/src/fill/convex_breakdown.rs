use glam::DVec2;

use super::get_first_convex_index::get_first_convex_index;

pub fn convex_breakdown(polygon: &mut Vec<DVec2>) -> (Vec<Vec<DVec2>>, Vec<Vec<DVec2>>) {
    let mut clockwise_convexes: Vec<Vec<DVec2>> = vec![];
    let mut counter_clockwise_convexes: Vec<Vec<DVec2>> = vec![];
    let mut last_clockwise = DVec2::NAN;
    let mut last_counter_clockwise = DVec2::NAN;
    while polygon.len() >= 3 {
        let (i, sign) = get_first_convex_index(polygon);
        let rest = polygon.split_off(i + 1);
        if sign > 0. {
            clockwise_convexes.push(polygon.to_vec());
        } else if sign < 0. {
            counter_clockwise_convexes.push(polygon.to_vec());
        }

        let mut rest_with_clipped = vec![
            *polygon.first().expect("Polygon len is insufficient"),
            *polygon.last().expect("Polygon len is insufficient"),
        ];
        rest_with_clipped.extend(rest);
        *polygon = rest_with_clipped;
    }
    let (fill, mask) = (clockwise_convexes, counter_clockwise_convexes);
    (fill, mask)
}
