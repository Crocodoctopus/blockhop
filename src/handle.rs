pub struct HandleGenerator {
    id: u32,
}

impl HandleGenerator {
    pub fn new() -> Self {
        Self { id: 0 }
    }

    pub fn gen(&mut self) -> Handle {
        self.id += 1;

        Handle {
            id: self.id - 1,
            subid: 0,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Handle {
    id: u32,
    subid: u32,
}

impl Handle {
    pub fn gen_next(&self) -> Self {
        Self {
            id: self.id,
            subid: self.subid + 1,
        }
    }
}
