use super::Chunk;
use num_traits::ToPrimitive;

#[repr(u32)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Direction {
    PosX = 0,
    NegX = 1,
    PosY = 2,
    NegY = 3,
    PosZ = 4,
    NegZ = 5,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct VoxelCoord {
    pub vec: uv::IVec3,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct ChunkCoord {
    pub vec: uv::IVec3,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct ChunkIndex(pub u32);

impl Direction {
    pub const NORTH: Self = Self::PosY;
    pub const EAST: Self = Self::PosX;
    pub const SOUTH: Self = Self::NegY;
    pub const WEST: Self = Self::NegX;
    pub const UP: Self = Self::PosZ;
    pub const DOWN: Self = Self::NegZ;
    pub const COUNT: usize = 6;

    pub fn iterate() -> impl Iterator<Item = Self> {
        [
            Self::PosX,
            Self::NegX,
            Self::PosY,
            Self::NegY,
            Self::PosZ,
            Self::NegZ,
        ]
        .iter()
        .cloned()
    }

    pub fn vec(&self) -> uv::IVec3 {
        match *self {
            Direction::PosX => uv::IVec3::new(1, 0, 0),
            Direction::NegX => uv::IVec3::new(-1, 0, 0),
            Direction::PosY => uv::IVec3::new(0, 1, 0),
            Direction::NegY => uv::IVec3::new(0, -1, 0),
            Direction::PosZ => uv::IVec3::new(0, 0, 1),
            Direction::NegZ => uv::IVec3::new(0, 0, -1),
        }
    }
}

impl VoxelCoord {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self {
            vec: uv::IVec3::new(x, y, z),
        }
    }

    pub fn advance(&self, direction: Direction) -> Self {
        Self {
            vec: self.vec + direction.vec(),
        }
    }

    pub fn chunk(&self) -> ChunkCoord {
        ChunkCoord::new(
            self.vec.x >> Chunk::BITS,
            self.vec.y >> Chunk::BITS,
            self.vec.z >> Chunk::BITS,
        )
    }

    pub(in crate::voxel) fn chunk_index(&self) -> ChunkIndex {
        ChunkIndex::new_unchecked(
            (self.vec.x & Chunk::BIT_MASK as i32) as u32,
            (self.vec.y & Chunk::BIT_MASK as i32) as u32,
            (self.vec.z & Chunk::BIT_MASK as i32) as u32,
        )
    }
}

impl ChunkCoord {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self {
            vec: uv::IVec3::new(x, y, z),
        }
    }

    pub fn advance(&self, direction: Direction) -> Self {
        Self {
            vec: self.vec + direction.vec(),
        }
    }

    pub fn mat(&self) -> uv::Mat4 {
        uv::Mat4::from_translation(uv::Vec3::new(
            (self.vec.x << Chunk::BITS) as f32,
            (self.vec.y << Chunk::BITS) as f32,
            (self.vec.z << Chunk::BITS) as f32,
        ))
    }
}

impl ChunkIndex {
    pub fn new(x: i32, y: i32, z: i32) -> Option<Self> {
        let (x, y, z) = (x.to_u32()?, y.to_u32()?, z.to_u32()?);
        (x < Chunk::LENGTH && y < Chunk::LENGTH && z < Chunk::LENGTH)
            .then(|| Self::new_unchecked(x, y, z))
    }

    pub fn new_unchecked(x: u32, y: u32, z: u32) -> Self {
        Self(z << Chunk::BITS2 | y << Chunk::BITS | x)
    }

    pub fn x(&self) -> u32 {
        self.0 & Chunk::BIT_MASK
    }

    pub fn y(&self) -> u32 {
        (self.0 >> Chunk::BITS) & Chunk::BIT_MASK
    }

    pub fn z(&self) -> u32 {
        (self.0 >> Chunk::BITS2) & Chunk::BIT_MASK
    }

    pub fn iterate() -> impl Iterator<Item = Self> {
        (0..Chunk::VOLUME).map(|x| Self(x))
    }
}

impl std::fmt::Debug for ChunkIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChunkIndex")
            .field("x", &self.x())
            .field("y", &self.y())
            .field("z", &self.z())
            .finish()
    }
}
