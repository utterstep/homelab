#[derive(Debug)]
pub struct BackgroundState {
    data: Vec<u8>,
    hash: u32,
    name: String,
}

impl BackgroundState {
    pub fn new(data: Vec<u8>, hash: u32, name: String) -> Self {
        Self { data, hash, name }
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn hash(&self) -> u32 {
        self.hash
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
