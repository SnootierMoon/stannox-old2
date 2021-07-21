use super::{Chunk, ChunkCoord, ChunkIndex, Direction, Object, Voxel, VoxelCoord};

pub struct Mesh {
    pub coord: ChunkCoord,
    pub faces: Vec<MeshFace>,
}

#[derive(Copy, Clone)]
pub struct MeshFace {
    voxel: u32,
    location: u32,
}

impl MeshFace {
    pub(in crate::voxel) fn new(voxel: Voxel, index: ChunkIndex, direction: Direction) -> Self {
        Self {
            voxel: voxel.0,
            location: ((direction as u32) << Chunk::BITS3) | index.0,
        }
    }
}

pub fn mesh(object: &Object, coord: ChunkCoord) -> Option<Mesh> {
    Some(mesh_with_chunk(object, object.chunk(coord)?, coord))
}

pub fn mesh_with_chunk(object: &Object, chunk: &Chunk, coord: ChunkCoord) -> Mesh {
    let neighbors = object.neighbors(coord);
    let mut faces = Vec::new();
    for z in 0..Chunk::LENGTH as i32 {
        for y in 0..Chunk::LENGTH as i32 {
            for x in 0..Chunk::LENGTH as i32 {
                let coord = VoxelCoord::new(x, y, z);
                let idx = coord.chunk_index();
                if chunk[idx].is_void() {
                    continue;
                }
                for (direction, neighbor) in neighbors {
                    let neighbor_coord = coord.advance(direction);
                    let neighbor_chunk = if neighbor_coord.chunk().vec == uv::IVec3::zero() {
                        Some(chunk)
                    } else {
                        neighbor
                    };
                    let mesh_test = neighbor_chunk
                        .map(|chunk| chunk[neighbor_coord.chunk_index()])
                        .unwrap_or(Voxel::VOID)
                        .is_void();
                    if mesh_test {
                        faces.push(MeshFace::new(
                            Voxel::from_id(direction as u32),
                            idx,
                            direction,
                        ))
                    }
                }
            }
        }
    }
    Mesh { coord, faces }
}
