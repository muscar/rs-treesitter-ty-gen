pub struct NameGen {
    idx: usize,
}

impl NameGen {
    pub fn new() -> Self {
        Self { idx: 0 }
    }

    pub fn get_fresh_name(&mut self, prefix: &str) -> String {
        let name = format!("{}_{}", prefix, self.idx);
        self.idx += 1;
        name
    }
}
