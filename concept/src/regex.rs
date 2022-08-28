#[derive(Clone, Debug, Default)]
pub struct RegexPattern {
    pub regex_pattern: String,
    pub index: usize,
}
#[derive(Clone, Debug, Default)]
pub struct RegexPatterns {
    pub inner: Vec<RegexPattern>,
}

impl RegexPatterns {
    pub fn add(&mut self, regex_pattern: &str) -> RegexPattern {
        let regex_pattern = RegexPattern {
            regex_pattern: regex_pattern.to_string(),
            index: self.inner.len(),
        };
        self.inner.push(regex_pattern.clone());
        regex_pattern
    }
}
