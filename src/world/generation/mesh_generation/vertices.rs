use std::hash::Hash;

use bevy::{platform::collections::HashMap, prelude::*};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct VertexIndex( pub usize );

#[derive(Clone, Copy, Debug, Default)]
pub struct Vertex {
    pub pos: Vec3
}

#[derive(Clone, Debug, Default)]
pub struct VertexArray {
    data: Vec<Vertex>
}

impl VertexArray {
    pub fn new(data: Vec<Vertex>) -> Self {
        Self {
            data
        }
    }

    pub fn add(&mut self, vertex: Vertex) -> usize {
        self.data.push(vertex);
        self.data.len() - 1
    }

    pub fn get(&self, idx: &VertexIndex) -> &Vertex {
        &self.data[idx.0]
    }

    pub fn count(&self) -> usize {
        self.data.len()
    }

    pub fn append(&mut self, mut other: Self) {
        self.data.append(&mut other.data);
    }

    pub fn ref_mut(&mut self) -> &mut Vec<Vertex> {
        &mut self.data
    }

    pub fn to_raw(self) -> Vec<[f32; 3]> {
        let mut result: Vec<[f32; 3]> = Vec::new();
        for v in self.data {
            result.push(v.pos.to_array());
        }
        result
    }

    pub fn to_raw_flat_shading(self, triangles: Vec<Triangle>) -> (Vec<[f32; 3]>, Vec<u32>) {
        let mut result: Vec<[f32; 3]> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
        for t in triangles {
            for i in t.verts {
                result.push(self.data[i.0].pos.to_array());
                indices.push(indices.len() as u32);
            }
        }
        (result, indices)
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Triangle {
    pub verts: [VertexIndex; 3]
}

#[derive(Debug)]
pub struct HashedVertexArray<'a, K, Gen> {
    existing_data: &'a VertexArray,
    generated_data: VertexArray,
    map: HashMap<K, usize>,
    pub generator: Gen,
}

impl<'a, K, Gen> HashedVertexArray<'a, K, Gen>
where
    K: std::cmp::Eq + Hash + Clone,
    Gen: Fn(&K) -> Vertex
{
    pub fn new(vertex_array: &'a VertexArray, generator: Gen) -> Self {
        Self {
            existing_data: vertex_array,
            generated_data: VertexArray::default(),
            map: HashMap::new(),
            generator: generator
        }
    }

    pub fn get_idx(&mut self, key: &K) -> VertexIndex {
        if let Some(idx) = self.map.get(key) {
            return VertexIndex(*idx);
        }
        let idx = self.generated_data.add(
            (self.generator)(key)
        ) + self.existing_data.count();

        self.map.insert(key.clone(), idx);
        VertexIndex(idx)
    }

    pub fn get_vertex(&self, idx: &VertexIndex) -> &Vertex {
        if idx.0 < self.existing_data.count() {
            return self.existing_data.get(idx)
        } else if idx.0 - self.existing_data.count() < self.generated_data.count() {
            let local = VertexIndex(idx.0 - self.existing_data.count());
            return self.generated_data.get(&local)
        } else {
            panic!("Out of bounds access in hashed vertex array!");
        }
    }

    pub fn insert_unmapped(&mut self, vertex: &Vertex) -> VertexIndex {
        let local_idx = self.generated_data.add(*vertex);
        VertexIndex(local_idx + self.existing_data.count())
    }

    pub fn retrieve_data(self) -> VertexArray {
        self.generated_data
    }
}