pub struct Callback<'a, A, T> {
    func: Box<dyn FnMut(&A) -> T + 'a + Send>,
}

impl<'a, A, T> Callback<'a, A, T> {
    pub fn new(c: impl FnMut(&A) -> T + 'a + Send) -> Self {
        Self { func: Box::new(c) }
    }
    pub fn process_events(&mut self, arg: &A) -> T {
        (self.func)(arg)
    }
}

pub struct MutCallback<'a, A, T> {
    func: Box<dyn Fn(&mut A) -> T + 'a>,
}

impl<'a, A, T> MutCallback<'a, A, T> {
    pub fn new(c: impl Fn(&mut A) -> T + 'a) -> Self {
        Self { func: Box::new(c) }
    }
    pub fn process_events(&mut self, arg: &mut A) -> T {
        (self.func)(arg)
    }
}
