use super::{ChunkIndex, Voxel};

pub struct Chunk {
    voxels: Box<[Voxel; Self::VOLUME as usize]>,
}

impl Chunk {
    pub const LENGTH: u32 = 32;
    pub const AREA: u32 = 1024;
    pub const VOLUME: u32 = 32768;
    pub const BITS: u32 = 5;
    pub const BITS2: u32 = 10;
    pub const BITS3: u32 = 15;
    pub const BIT_MASK: u32 = 0x1F;

    pub fn new_void() -> Self {
        Self {
            voxels: Box::new([Voxel::VOID; Self::VOLUME as usize]),
        }
    }

    pub fn new_sphere() -> Self {
        let mut chunk = Self::new_void();
        let center = uv::Vec3::broadcast(15.5);
        for index in ChunkIndex::iterate() {
            let index_vec = uv::Vec3::new(index.x() as f32, index.y() as f32, index.z() as f32);
            if (index_vec - center).mag_sq() < 256.0 {
                chunk[index] = Voxel::from_id(1)
            }
        }
        chunk
    }
}

impl std::ops::Index<ChunkIndex> for Chunk {
    type Output = Voxel;

    fn index(&self, index: ChunkIndex) -> &Self::Output {
        &self.voxels[index.0 as usize]
    }
}

impl std::ops::IndexMut<ChunkIndex> for Chunk {
    fn index_mut(&mut self, index: ChunkIndex) -> &mut Self::Output {
        &mut self.voxels[index.0 as usize]
    }
}

impl std::fmt::Debug for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Chunk")
    }
}
