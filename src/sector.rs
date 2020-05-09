use crate::chunk::Chunk;

// a sectors is a size of 16x16x16 chunks

pub struct Sector {
    position: [u32; 2],
    size: u8,  // by chunks
    chunks: Vec<Chunk>,

    time_off: u8,  // time when offloaded
}

impl Sector {
    pub fn new(position: [u32; 2]) -> Self {
        Self {
            position: position,
            size: 16,
            chunks: Vec::new(),

            time_off: 0,
        }
    }

    // create chunks
    pub fn create(&mut self) {
        // self.chunks.push(Chunk::new());
    }

    pub fn update() {

    }
}
