use concept::constraint::{Constraint, XConstraint, YConstraint};
use concept::svg_init::{regex::RegexSet, RegexPatterns};

fn get_y_constraint(_id: &str) -> YConstraint {
    YConstraint::Center(0.)
}
fn get_x_constraint(id: &str) -> XConstraint {
    let mut regex_patterns = RegexPatterns::default();
    let menu = regex_patterns.add(r"Menu #transform");
    let grab = regex_patterns.add(r"Grab #transform");
    let undo = regex_patterns.add(r"Undo #transform");
    let constraint_regex =
        RegexSet::new(regex_patterns.inner.iter().map(|r| &r.regex_pattern)).unwrap();
    let matches = constraint_regex.matches(id);
    if matches.matched(menu.index) {
        XConstraint::Left(15.)
    } else if matches.matched(grab.index) {
        XConstraint::Center(0.)
    } else if matches.matched(undo.index) {
        XConstraint::Right(-15.)
    } else {
        XConstraint::Scale
    }
}

pub fn get_constraint(id: &str) -> Constraint {
    Constraint {
        x: get_x_constraint(id),
        y: get_y_constraint(id),
    }
}
