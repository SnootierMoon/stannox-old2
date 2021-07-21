use super::{Chunk, ChunkCoord, Direction, Mesh, Voxel, VoxelCoord};

pub struct Object {
    chunks: std::collections::HashMap<ChunkCoord, Chunk>,
}

impl Object {
    pub fn new() -> Self {
        Self {
            chunks: std::collections::HashMap::new(),
        }
    }

    pub fn new_test() -> Self {
        let mut chunks = std::collections::HashMap::new();
        chunks.insert(ChunkCoord::new(-1, 0, 0), Chunk::new_sphere());
        chunks.insert(ChunkCoord::new(0, 0, 0), Chunk::new_sphere());
        chunks.insert(ChunkCoord::new(1, 0, 0), Chunk::new_sphere());
        chunks.insert(ChunkCoord::new(2, 0, 0), Chunk::new_sphere());
        chunks.insert(ChunkCoord::new(0, 1, 0), Chunk::new_sphere());
        chunks.insert(ChunkCoord::new(0, 0, 1), Chunk::new_sphere());
        Self { chunks }
    }

    pub fn fuck_it_mesh_all(&self) -> Vec<Mesh> {
        let mut meshes = Vec::new();
        for (coord, chunk) in &self.chunks {
            meshes.push(crate::voxel::mesh::mesh_with_chunk(self, chunk, *coord))
        }
        meshes
    }

    pub fn neighbors(&self, coord: ChunkCoord) -> [(Direction, Option<&Chunk>); 6] {
        [
            (Direction::PosX, self.chunk(coord.advance(Direction::PosX))),
            (Direction::NegX, self.chunk(coord.advance(Direction::NegX))),
            (Direction::PosY, self.chunk(coord.advance(Direction::PosY))),
            (Direction::NegY, self.chunk(coord.advance(Direction::NegY))),
            (Direction::PosZ, self.chunk(coord.advance(Direction::PosZ))),
            (Direction::NegZ, self.chunk(coord.advance(Direction::NegZ))),
        ]
    }

    pub fn chunk(&self, coord: ChunkCoord) -> Option<&Chunk> {
        self.chunks.get(&coord)
    }

    pub fn chunk_mut(&mut self, coord: ChunkCoord) -> &mut Chunk {
        self.chunks.entry(coord).or_insert(Chunk::new_void())
    }
}

impl std::ops::Index<ChunkCoord> for Object {
    type Output = Chunk;

    fn index(&self, index: ChunkCoord) -> &Self::Output {
        &self.chunks[&index]
    }
}

impl std::ops::Index<VoxelCoord> for Object {
    type Output = Voxel;

    fn index(&self, index: VoxelCoord) -> &Self::Output {
        match self.chunk(index.chunk()) {
            Some(chunk) => &chunk[index.chunk_index()],
            None => &Voxel::VOID,
        }
    }
}

impl std::ops::IndexMut<VoxelCoord> for Object {
    fn index_mut(&mut self, index: VoxelCoord) -> &mut Self::Output {
        &mut self.chunk_mut(index.chunk())[index.chunk_index()]
    }
}
