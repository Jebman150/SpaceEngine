use std::hash::Hash;

use bevy::{platform::collections::HashMap, prelude::*};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct VertexIndex( pub usize );

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vertex {
    pub pos: Vec3
}

impl Vertex {
    pub fn zero() -> Self {
        Self {
            pos: Vec3::ZERO
        }
    }
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

    pub fn add(&mut self, vertex: Vertex) -> VertexIndex {
        self.data.push(vertex);
        VertexIndex(self.data.len() - 1)
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

    pub fn update(&mut self, i: &VertexIndex, val: Vec3) {
        self.data[i.0].pos = val;
    }

    pub fn to_raw(&self) -> Vec<[f32; 3]> {
        let mut result: Vec<[f32; 3]> = Vec::new();
        for v in &self.data {
            result.push(v.pos.to_array());
        }
        result
    }

    pub fn to_raw_flat_shading(&self, triangles: &PolygonArray<3>) -> (Vec<[f32; 3]>, Vec<u32>) {
        let mut result: Vec<[f32; 3]> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
        for t in &triangles.data {
            for i in t.verts {
                result.push(self.data[i.0].pos.to_array());
                indices.push(indices.len() as u32);
            }
        }
        (result, indices)
    }
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
        ).0 + self.existing_data.count();

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
        VertexIndex(local_idx.0 + self.existing_data.count())
    }

    pub fn retrieve_data(self) -> VertexArray {
        self.generated_data
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Polygon<const N: usize> {
    pub verts: [VertexIndex; N]
}

pub type Line = Polygon<2>;
pub type Triangle = Polygon<3>;
pub type Quad = Polygon<4>;
//pub type Pentagon = Polygon<5>;
pub type Hexagon = Polygon<6>;

impl<const N: usize> Polygon<N> {
    pub fn find_edge_positions(&self, edge: &(VertexIndex, VertexIndex)) -> Option<(usize, usize)> {
        for (i, v) in self.verts.iter().enumerate() {
            if *v != edge.0 {
                continue;
            }

            let j = (i+1) % N;
            if self.verts[j] == edge.1 {
                return Some((i, j));
            }
            let j = if i == 0 {
                N-1
            } else {
                i-1
            };
            if self.verts[j] == edge.1 {
                return Some((j, i));
            }
        }
        None
    }

    pub fn get_edges(&self) -> Vec<(VertexIndex, VertexIndex)> {
        let mut result = Vec::new();
        for (i, v) in self.verts.iter().enumerate() {
            let next = self.verts[(i+1) % N];
            if *v < next {
                if result.contains(&(*v, next)) {
                    continue;
                }
                result.push((*v, next));
            } else {
                if result.contains(&(next, *v)) {
                    continue;
                }
                result.push((next, *v));
            }
        }
        result
    }

    pub fn get_all_pairs(&self) -> Vec<(VertexIndex, VertexIndex)> {
        let mut result = Vec::new();
        for (i, v) in self.verts.iter().enumerate() {
            let next = self.verts[(i+1) % N];
            result.push((*v, next));
            result.push((next, *v));
        }
        result
    }
}

#[derive(Clone, Debug, Default)]
pub struct PolygonArray<const N: usize> {
    pub data: Vec<Polygon<N>>
}

impl<const N: usize> PolygonArray<N> {
    pub fn new(data: &[Polygon<N>]) -> Self {
        Self { data: data.to_vec() }
    }

    pub fn to_raw(&self) -> Vec<u32> {
        let mut result: Vec<u32> = Vec::new();
        for polygon in &self.data {
            for i in polygon.verts {
                result.push(i.0 as u32);
            }
        }
        result
    }

    pub fn count(&self) -> usize {
        self.data.len()
    }

    pub fn add(&mut self, polygon: Polygon<N>) {
        self.data.push(polygon);
    }

    pub fn append(&mut self, mut other: Self) {
        self.data.append(&mut other.data);
    }
}

#[derive(Clone, Debug, Default)]
pub struct MeshData {
    pub normal: Vec3,
    pub tangent: Vec3,
    pub bitangent: Vec3,
    pub vertices: VertexArray,
    pub boundary_indices: Vec<VertexIndex>,
    pub lines: PolygonArray<2>,
    pub triangles: PolygonArray<3>,
    pub quads: PolygonArray<4>,
    pub hex: PolygonArray<6>,
}

impl MeshData {
    pub fn set_orientation(&mut self, normal: Vec3, tangent: Vec3) {
        self.normal = normal;
        self.tangent = tangent;
        self.bitangent = tangent.cross(normal);
    }

    pub fn scale(&mut self, factor: f32) {
        for v in &mut self.vertices.data {
            v.pos *= factor;
        }
    }

    pub fn get_edges_of_triangles(&self) -> HashMap<(VertexIndex, VertexIndex), Vec<usize>> {
        let mut edge_map: HashMap<(VertexIndex, VertexIndex), Vec<usize>> = HashMap::new();
        for (tri_idx, tri) in self.triangles.data.iter().enumerate() {
            let edges = tri.get_edges();

            for e in edges {
                edge_map.entry(e)
                    .or_default()
                    .push(tri_idx);
            }
        }
        edge_map
    }

    pub fn generate_neighbour_map(&self) -> Vec<Vec<VertexIndex>> {
        let mut neighbours: Vec<Vec<VertexIndex>> = vec![Vec::new(); self.vertices.count()];

        for quad in &self.quads.data {
            let edges = quad.get_all_pairs();
            for e in edges {
                if !neighbours[e.0.0].contains(&e.1) {
                    neighbours[e.0.0].push(e.1);
                }
            }
        }
        neighbours
    }

    pub fn subdivide_to_quads(&mut self) {
        let triangle_center_start_index = self.vertices.count();
        for t in &self.triangles.data {
            let center = center_pos(&[
                self.vertices.get(&t.verts[0]),
                self.vertices.get(&t.verts[1]),
                self.vertices.get(&t.verts[2])]
            );
            let _ = self.vertices.add(center);
        }
        let quad_center_start_index = self.vertices.count();
        for q in &self.quads.data {
            let center = center_pos(&[
                self.vertices.get(&q.verts[0]),
                self.vertices.get(&q.verts[1]),
                self.vertices.get(&q.verts[2]),
                self.vertices.get(&q.verts[3])]
            );
            let _ = self.vertices.add(center);
        }
        let mut edge_centers = HashedVertexArray::new(
            &self.vertices,
            |id: &(VertexIndex, VertexIndex)| {
                center_pos(&[self.vertices.get(&id.0), self.vertices.get(&id.1)])
            }
        );

        let mut resulting_quads: PolygonArray<4> = PolygonArray::default();
        for (i, triangle) in self.triangles.data.iter().enumerate() {
            let edges = [
                (triangle.verts[0], triangle.verts[1]),
                (triangle.verts[1], triangle.verts[2]),
                (triangle.verts[2], triangle.verts[0]),
            ];
            let mut mid_points = [VertexIndex(0); 3];
            for (i, edge) in edges.iter().enumerate() {
                let sorted_edge = if edge.0.0 < edge.1.0 {
                    (edge.0, edge.1)
                } else {
                    (edge.1, edge.0)
                };
                mid_points[i] = edge_centers.get_idx(&sorted_edge);
                if self.boundary_indices.contains(&sorted_edge.0) && self.boundary_indices.contains(&sorted_edge.1) {
                    self.boundary_indices.push(mid_points[i]);
                }
            }

            resulting_quads.add(
                Quad { verts: [
                    edges[0].0,
                    mid_points[0],
                    VertexIndex(triangle_center_start_index+i),
                    mid_points[2]
                ]}
            );
            resulting_quads.add(
                Quad { verts: [
                    mid_points[0],
                    edges[0].1,
                    mid_points[1],
                    VertexIndex(triangle_center_start_index+i)
                ]}
            );
            resulting_quads.add(
                Quad { verts: [
                    mid_points[1],
                    edges[1].1,
                    mid_points[2],
                    VertexIndex(triangle_center_start_index+i)
                ]}
            );
        }

        for (i, quad) in self.quads.data.iter().enumerate() {
            let edges = [
                (quad.verts[0], quad.verts[1]),
                (quad.verts[1], quad.verts[2]),
                (quad.verts[2], quad.verts[3]),
                (quad.verts[3], quad.verts[0]),
            ];
            let mut mid_points = [VertexIndex(0); 4];
            for (i, edge) in edges.iter().enumerate() {
                let sorted_edge = if edge.0.0 < edge.1.0 {
                    (edge.0, edge.1)
                } else {
                    (edge.1, edge.0)
                };
                mid_points[i] = edge_centers.get_idx(&sorted_edge);
                if self.boundary_indices.contains(&sorted_edge.0) && self.boundary_indices.contains(&sorted_edge.1) {
                    self.boundary_indices.push(mid_points[i]);
                }
            }

            resulting_quads.add(
                Quad { verts: [
                    mid_points[3],
                    edges[3].1,
                    mid_points[0],
                    VertexIndex(quad_center_start_index+i),
                ]}
            );
            resulting_quads.add(
                Quad { verts: [
                    mid_points[0],
                    edges[0].1,
                    mid_points[1],
                    VertexIndex(quad_center_start_index+i)
                ]}
            );
            resulting_quads.add(
                Quad { verts: [
                    mid_points[1],
                    edges[1].1,
                    mid_points[2],
                    VertexIndex(quad_center_start_index+i)
                ]}
            );
            resulting_quads.add(
                Quad { verts: [
                    mid_points[2],
                    edges[2].1,
                    mid_points[3],
                    VertexIndex(quad_center_start_index+i)
                ]}
            );
            
        }
        self.vertices.append(edge_centers.retrieve_data());
        self.triangles = PolygonArray::default();
        self.quads = resulting_quads;
    }

    pub fn subdivide_triangles(&mut self) {
        let mut edge_centers = HashedVertexArray::new(
            &self.vertices,
            |id: &(VertexIndex, VertexIndex)| {
                center_pos(&[self.vertices.get(&id.0), self.vertices.get(&id.1)])
            }
        );

        let mut new_triangles: PolygonArray<3> = PolygonArray::default();
        for triangle in &mut self.triangles.data {
            let edges = [
                (triangle.verts[0], triangle.verts[1]),
                (triangle.verts[1], triangle.verts[2]),
                (triangle.verts[2], triangle.verts[0]),
            ];
            let mut mid_points = [VertexIndex(0); 3];
            for (i, edge) in edges.iter().enumerate() {
                let sorted_edge = if edge.0.0 < edge.1.0 {
                    (edge.0, edge.1)
                } else {
                    (edge.1, edge.0)
                };
                mid_points[i] = edge_centers.get_idx(&sorted_edge);
                if self.boundary_indices.contains(&sorted_edge.0) && self.boundary_indices.contains(&sorted_edge.1) {
                    self.boundary_indices.push(mid_points[i]);
                }
            }

            triangle.verts[0] = edges[0].0;
            triangle.verts[1] = mid_points[0];
            triangle.verts[2] = mid_points[2];

            new_triangles.add(
                Triangle { verts: [
                    mid_points[0],
                    edges[0].1,
                    mid_points[1]
                ]}
            );
            new_triangles.add(
                Triangle { verts: [
                    mid_points[1],
                    edges[1].1,
                    mid_points[2]
                ]}
            );
            new_triangles.add(
                Triangle { verts: [
                    mid_points[0],
                    mid_points[1],
                    mid_points[2]
                ]}
            );
        }
        self.vertices.append(edge_centers.retrieve_data());
        self.triangles.append(new_triangles);
    }

    pub fn reduce_to_triangles(&mut self) {
        while self.quads.count() != 0 {
            let verts = self.quads.data.last().expect("No quads?").verts;
            self.triangles.add(Triangle { verts: [verts[0], verts[1], verts[2]]});
            self.triangles.add(Triangle { verts: [verts[0], verts[2], verts[3]]});
            self.quads.data.pop();
        }
        while self.hex.count() != 0 {
            let verts = self.hex.data.last().expect("No hexagons?").verts;
            let mut vertices = Vec::new();
            for i in verts {
                vertices.push(self.vertices.get(&i));
            }
            let center = center_pos(&vertices);
            self.vertices.add(center);
            let center_idx = VertexIndex(self.vertices.count()-1);
            self.triangles.add(Triangle { verts: [center_idx, verts[0], verts[1]]});
            self.triangles.add(Triangle { verts: [center_idx, verts[1], verts[2]]});
            self.triangles.add(Triangle { verts: [center_idx, verts[2], verts[3]]});

            self.triangles.add(Triangle { verts: [center_idx, verts[3], verts[4]]});
            self.triangles.add(Triangle { verts: [center_idx, verts[4], verts[5]]});
            self.triangles.add(Triangle { verts: [center_idx, verts[5], verts[0]]});
            self.hex.data.pop();
        }
    }

    pub fn reduce_to_lines(&mut self) {
        while self.triangles.count() != 0 {
            let verts = self.triangles.data.last().expect("No triangles?").verts;
            self.lines.add(Line { verts: [verts[0], verts[1]] });
            self.lines.add(Line { verts: [verts[1], verts[2]] });
            self.lines.add(Line { verts: [verts[2], verts[0]] });
            self.triangles.data.pop();
        }
        while self.quads.count() != 0 {
            let verts = self.quads.data.last().expect("No quads?").verts;
            self.lines.add(Line { verts: [verts[0], verts[1]] });
            self.lines.add(Line { verts: [verts[1], verts[2]] });
            self.lines.add(Line { verts: [verts[2], verts[3]] });
            self.lines.add(Line { verts: [verts[3], verts[0]] });
            self.quads.data.pop();
        }
    }

    pub fn to_raw_triangles(&mut self) -> (Vec<[f32; 3]>, Vec<u32>) {
        (self.vertices.to_raw(), self.triangles.to_raw())
    }

    pub fn to_raw_lines(&mut self) -> (Vec<[f32; 3]>, Vec<u32>) {
        self.reduce_to_lines();
        info!("Reduced to {} lines", self.lines.count());
        (self.vertices.to_raw(), self.lines.to_raw())
    }

    pub fn to_raw_flat_shading(&mut self) -> (Vec<[f32; 3]>, Vec<u32>) {
        self.vertices.to_raw_flat_shading(&self.triangles)
    }
}

fn center_pos(vertices: &[&Vertex]) -> Vertex {
    let mut sum = Vec3::ZERO;
    for v in vertices {
        sum += v.pos;
    }
    sum /= vertices.len() as f32;
    Vertex{ pos: sum }
}

#[cfg(test)]
mod tests {
    use std::f32::consts::PI;
    use super::*;

    fn generate_vertex(i: usize) -> Vertex {
        if i == 0 {
            Vertex { pos: Vec3::ZERO }
        } else {
            let pos = Vec3::X.rotate_axis(Vec3::Y, i as f32 * PI/3.0);
            Vertex { pos: pos }
        }
    }

    fn fill_vertex_array(a: &mut VertexArray) {
        for i in 0..7 {
            a.add(generate_vertex(i));
        }
    }

    ////////////////////////////////////////
    /// VERTEX ARRAY TESTS
    ////////////////////////////////////////

    #[test]
    fn vertex_array_add() {
        let mut test_subject = VertexArray::default();
        for i in 0..3 {
            let vertex = Vertex::default();
            let index = test_subject.add(vertex);
            assert_eq!(index.0, i);
        }
        assert_eq!(test_subject.count(), 3);
        assert_eq!(test_subject.data, vec![Vertex::default(), Vertex::default(), Vertex::default()])
    }

    #[test]
    fn vertex_array_get() {
        let mut test_subject = VertexArray::default();
        fill_vertex_array(&mut test_subject);
        for i in 0..7 {
            let vert = test_subject.get(&VertexIndex(i));
            assert_eq!(vert.pos, generate_vertex(i).pos);
        }
    }

    #[test]
    fn vertex_array_count() {
        let mut test_subject = VertexArray::default();
        fill_vertex_array(&mut test_subject);
        assert_eq!(test_subject.count(), 7);
    }

    #[test]
    fn vertex_array_append() {
        let mut test_subject = VertexArray::default();
        let mut test_subject_b = VertexArray::default();
        fill_vertex_array(&mut test_subject);
        fill_vertex_array(&mut test_subject_b);
        test_subject.append(test_subject_b);
        for i in 0..14 {
            let vert = test_subject.get(&VertexIndex(i));
            assert_eq!(vert.pos, generate_vertex(i % 7).pos);
        }
    }

    #[test]
    fn vertex_array_to_raw() {
        let mut test_subject = VertexArray::default();
        fill_vertex_array(&mut test_subject);
        let raw = test_subject.to_raw();
        assert_eq!(raw.len(), 7);
        for i in 0..7 {
            let vert = test_subject.get(&VertexIndex(i));
            assert_eq!(vert.pos.x, raw[i][0]);
            assert_eq!(vert.pos.y, raw[i][1]);
            assert_eq!(vert.pos.z, raw[i][2]);
        }
    }

    ////////////////////////////////////////
    /// POLYGON TESTS
    ////////////////////////////////////////
    
    fn generate_polygons() -> (Line, Triangle, Quad, Hexagon) {
        (
            Line { verts: [VertexIndex(0), VertexIndex(1)] },
            Triangle { verts: [VertexIndex(0), VertexIndex(2), VertexIndex(3)]},
            Quad { verts: [VertexIndex(2), VertexIndex(3), VertexIndex(5), VertexIndex(6)]},
            Hexagon { verts: [VertexIndex(1), VertexIndex(2), VertexIndex(3), VertexIndex(4), VertexIndex(5), VertexIndex(6)]}
        )
    }

    #[test]
    fn polygon_find_edge_position() {
        let (line, triangle, quad, hexagon) = generate_polygons();

        let test_edge = (VertexIndex(0), VertexIndex(1));
        assert_eq!(line.find_edge_positions(&test_edge).expect("Edge not found"), (0, 1));

        let test_edge = (VertexIndex(3), VertexIndex(0));
        assert_eq!(triangle.find_edge_positions(&test_edge).expect("Edge not found"), (2, 0));

        let test_edge = (VertexIndex(5), VertexIndex(3));
        assert_eq!(quad.find_edge_positions(&test_edge).expect("Edge not found"), (1, 2));

        let test_edge = (VertexIndex(1), VertexIndex(6));
        assert_eq!(hexagon.find_edge_positions(&test_edge).expect("Edge not found"), (5, 0));
    }

    #[test]
    fn polygon_get_edges() {
        let (line, triangle, quad, _) = generate_polygons();

        assert_eq!(line.get_edges(), [(VertexIndex(0), VertexIndex(1))]);

        assert_eq!(triangle.get_edges(), [
            (VertexIndex(0), VertexIndex(2)),
            (VertexIndex(2), VertexIndex(3)),
            (VertexIndex(0), VertexIndex(3))
            ]);

        assert_eq!(quad.get_edges(), [
            (VertexIndex(2), VertexIndex(3)),
            (VertexIndex(3), VertexIndex(5)),
            (VertexIndex(5), VertexIndex(6)),
            (VertexIndex(2), VertexIndex(6))
            ]);
    }

    ////////////////////////////////////////
    /// MESH DATA TESTS
    ////////////////////////////////////////
    
    fn generate_mesh() -> MeshData {
        let mut vertices = VertexArray::default();
        fill_vertex_array(&mut vertices);
        let (line, triangle, quad, hex) = generate_polygons();
        let mut mesh = MeshData {
            vertices,
            lines: PolygonArray::new(&vec![line]),
            triangles: PolygonArray::new(&vec![triangle]),
            quads: PolygonArray::new(&vec![quad]),
            hex: PolygonArray::new(&vec![hex]),
            ..default()
        };
        mesh.set_orientation(Vec3::Y, Vec3::X);
        assert_eq!(mesh.bitangent, Vec3::Z);
        mesh
    }

    #[test]
    fn mesh_data_get_edges_of_triangle() {
        let mut test_subject = generate_mesh();
        test_subject.triangles.add(Triangle { verts: [
            VertexIndex(0), VertexIndex(3), VertexIndex(4)
        ]});
        let edges = test_subject.get_edges_of_triangles();

        assert_eq!(*edges.get(&(VertexIndex(0), VertexIndex(2))).expect("Could not find edge"), vec![0 as usize]);
        assert_eq!(*edges.get(&(VertexIndex(2), VertexIndex(3))).expect("Could not find edge"), vec![0 as usize]);
        let shared_edge = edges.get(&(VertexIndex(0), VertexIndex(3))).expect("Could not find edge");
        assert!(shared_edge.contains(&1));
        assert!(shared_edge.contains(&0));
    }

    #[test]
    fn mesh_data_generate_neighbour_map() {
        let test_subject = generate_mesh();
        let list = test_subject.generate_neighbour_map();

        print!("Neighbour list: {:?}", list);
        assert!(list[2].contains(&VertexIndex(3)) && list[2].contains(&VertexIndex(6)));
        assert!(list[3].contains(&VertexIndex(2)) && list[3].contains(&VertexIndex(5)));
        assert!(list[5].contains(&VertexIndex(3)) && list[5].contains(&VertexIndex(6)));
        assert!(list[6].contains(&VertexIndex(5)) && list[6].contains(&VertexIndex(2)));
    }

    #[test]
    fn mesh_data_reduce_to_lines() {
        let mut test_subject = generate_mesh();
        test_subject.reduce_to_lines();
        let lines = test_subject.lines.data;
        print!("{:?}", lines);
        assert!(lines.contains(&Line { verts: [VertexIndex(0), VertexIndex(1)]}));

        assert!(lines.contains(&Line { verts: [VertexIndex(0), VertexIndex(2)]}));
        assert!(lines.contains(&Line { verts: [VertexIndex(2), VertexIndex(3)]}));
        assert!(lines.contains(&Line { verts: [VertexIndex(3), VertexIndex(0)]}));

        assert!(lines.contains(&Line { verts: [VertexIndex(2), VertexIndex(3)]}));
        assert!(lines.contains(&Line { verts: [VertexIndex(3), VertexIndex(5)]}));
        assert!(lines.contains(&Line { verts: [VertexIndex(5), VertexIndex(6)]}));
        assert!(lines.contains(&Line { verts: [VertexIndex(6), VertexIndex(2)]}));
    }
}