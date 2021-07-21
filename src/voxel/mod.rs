pub use chunk::Chunk;
use coord::ChunkIndex;
pub use coord::{ChunkCoord, Direction, VoxelCoord};
pub use mesh::{Mesh, MeshFace};
pub use object::Object;

mod chunk;
mod coord;
mod mesh;
mod object;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Voxel(pub(in crate::voxel) u32);

impl Voxel {
    pub const VOID: Self = Self(0);

    pub fn from_id(id: u32) -> Self {
        Self(id)
    }

    pub fn is_void(&self) -> bool {
        *self == Self::VOID
    }
}
