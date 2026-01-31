use derive_more::Deref;
use wgame_shader::{Attribute, BytesSink};
use wgpu::util::DeviceExt;

use crate::{ShapesState, shader::Vertex};

#[derive(Clone, PartialEq, Eq, Hash, Debug, Deref)]
pub struct Vertices {
    count: u32,
    #[deref]
    buffer: wgpu::Buffer,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, Deref)]
pub struct Indices {
    count: u32,
    num_vertices: u32,
    #[deref]
    buffer: wgpu::Buffer,
}

impl Vertices {
    pub fn new(state: &ShapesState, vertices: &[Vertex]) -> Self {
        Self {
            count: vertices.len() as u32,
            buffer: state
                .device()
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: &{
                        let mut bytes = BytesSink::default();
                        for v in vertices {
                            v.store(&mut bytes);
                        }
                        bytes.into_data()
                    },
                    usage: wgpu::BufferUsages::VERTEX,
                }),
        }
    }

    pub fn count(&self) -> u32 {
        self.count
    }
    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
}

impl Indices {
    pub fn new(state: &ShapesState, indices: &[u32]) -> Self {
        Self {
            count: indices.len() as u32,
            num_vertices: indices.iter().max().map(|x| x + 1).unwrap_or(0),
            buffer: state
                .device()
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: None,
                    contents: bytemuck::cast_slice::<u32, _>(indices),
                    usage: wgpu::BufferUsages::INDEX,
                }),
        }
    }

    pub fn count(&self) -> u32 {
        self.count
    }
    /// Minimum number of vertices to index by `self`.
    pub fn num_vertices(&self) -> u32 {
        self.num_vertices
    }
    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
    }
}

#[must_use]
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Mesh {
    vertices: Vertices,
    indices: Option<Indices>,
}

impl Mesh {
    pub fn new(vertices: Vertices, indices: Option<Indices>) -> Self {
        if let Some(indices) = &indices {
            assert!(indices.num_vertices() <= vertices.count());
        };
        Self { vertices, indices }
    }
    pub fn from_arrays(state: &ShapesState, vertices: &[Vertex], indices: Option<&[u32]>) -> Self {
        let vertices = Vertices::new(state, vertices);
        let indices = indices.map(|indices| Indices::new(state, indices));
        Self::new(vertices, indices)
    }

    pub fn count(&self) -> u32 {
        match &self.indices {
            None => self.vertices.count(),
            Some(indices) => indices.count(),
        }
    }
    pub fn vertices(&self) -> &Vertices {
        &self.vertices
    }
    pub fn indices(&self) -> Option<&Indices> {
        self.indices.as_ref()
    }
}
