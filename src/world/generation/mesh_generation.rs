mod vertices;

use core::f32;
use std::f32::consts::PI;

use bevy::{asset::RenderAssetUsages, math::{NormedVectorSpace, ops::sqrt}, mesh::{Indices, PrimitiveTopology}, platform::collections::HashMap, prelude::*};
use rand::seq::SliceRandom;

use vertices::*;

#[derive(Clone, Debug)]
pub enum RenderMode {
    Flat,
    Smooth,
    Frame,
}

impl Default for RenderMode {
    fn default() -> Self {
        RenderMode::Frame
    }
}

#[derive(Clone, Debug, Default)]
pub struct RenderOptions {
    pub scale: f32,
    pub mode: RenderMode,
}

#[derive(Clone, Debug)]
pub struct PlanetTile<const N: usize> {
    pub transform: Transform,
    pub outer_polygon: vertices::Polygon<N>,
    pub tile_mesh: MeshData,
}

#[derive(Clone, Debug, Default)]
pub struct PlanetMesh {
    pub hex_faces: Vec<PlanetTile<6>>,
    pub pent_faces: Vec<PlanetTile<5>>,
    pub outer_frame: MeshData,
    pub face_count: usize,
}

impl PlanetMesh {
    pub fn get_single_face(&self, i: usize, rendering_options: &RenderOptions) -> Mesh {
        let ((mut raw_vertices, raw_indices), index_mode) =
        match rendering_options.mode {
            RenderMode::Flat => {
                if i < 12 {(
                    self.pent_faces[i].tile_mesh.to_raw_flat_shading(),
                    PrimitiveTopology::TriangleList
                )} else {(
                    self.hex_faces[i-12].tile_mesh.to_raw_flat_shading(),
                    PrimitiveTopology::TriangleList
                )}
                }
            RenderMode::Smooth => {
                if i < 12 {(
                    self.pent_faces[i].tile_mesh.to_raw_triangles(),
                    PrimitiveTopology::TriangleList
                )} else {(
                    self.hex_faces[i-12].tile_mesh.to_raw_triangles(),
                    PrimitiveTopology::TriangleList
                )}
                }
            RenderMode::Frame => {
                if i < 12 {(
                    self.pent_faces[i].tile_mesh.to_raw_lines(),
                    PrimitiveTopology::LineList
                )} else {(
                    self.hex_faces[i-12].tile_mesh.to_raw_lines(),
                    PrimitiveTopology::LineList
                )}
                }
        };
        let usage = RenderAssetUsages::RENDER_WORLD;

        if i < 12 {
            let frame = self.pent_faces[i].outer_polygon;
            let sample_vertices = frame.verts;
            let mut origin = [Vec3::ZERO; 5];
            for v in 0..5 {
                origin[v] = self.pent_faces[i].tile_mesh.vertices.get(&VertexIndex(v)).pos;
            }
            let mut destination = [Vec3::ZERO; 5];
            for v in 0..5 {
                destination[v] = self.outer_frame.vertices.get(&sample_vertices[v]).pos * rendering_options.scale;
            }

            for v in &mut raw_vertices {
                let mut p = Vec3 { x: v[0], y: v[1], z: v[2]};
                barycentric_projection::<5>(&mut p, &origin, &destination);
                *v = [p.x, p.y, p.z];
            }
        } else {
            let frame = self.hex_faces[i-12].outer_polygon;
            let sample_vertices = frame.verts;
            let mut origin = [Vec3::ZERO; 6];
            for v in 0..6 {
                origin[v] = self.hex_faces[i-12].tile_mesh.vertices.get(&VertexIndex(v)).pos;
            }
            let mut destination = [Vec3::ZERO; 6];
            for v in 0..6 {
                destination[v] = self.outer_frame.vertices.get(&sample_vertices[v]).pos * rendering_options.scale;
            }

            for v in &mut raw_vertices {
                let mut p = Vec3 { x: v[0], y: v[1], z: v[2]};
                barycentric_projection::<6>(&mut p, &origin, &destination);
                *v = [p.x, p.y, p.z];
            }
        }

        Mesh::new(index_mode, usage)
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, raw_vertices)
            .with_inserted_indices(Indices::U32(raw_indices))
    }

    pub fn get_outer_grid(&self, rendering_options: &RenderOptions) -> Mesh {
        let ((mut raw_vertices, raw_indices), index_mode) =
        match rendering_options.mode {
            RenderMode::Flat => {(
                self.outer_frame.to_raw_flat_shading(),
                PrimitiveTopology::TriangleList
            )}
            RenderMode::Smooth => {(
                self.outer_frame.to_raw_triangles(),
                PrimitiveTopology::TriangleList
            )}
            RenderMode::Frame => {
                (
                self.outer_frame.to_raw_lines(),
                PrimitiveTopology::LineList
            )}
        };
        let usage = RenderAssetUsages::RENDER_WORLD;

        for v in &mut raw_vertices {
            v[0] *= rendering_options.scale;
            v[1] *= rendering_options.scale;
            v[2] *= rendering_options.scale;
        }

        Mesh::new(index_mode, usage)
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, raw_vertices)
            .with_inserted_indices(Indices::U32(raw_indices))
    }
}

pub fn generate_mesh(
    resolution: usize
) -> PlanetMesh {
    // First, generate the goldberg polyhedron out of hexagons and 12 pentagons
    let mut polyhedron = MeshData::default();
    polyhedron.vertices = VertexArray::new(icosahedron_vertices().to_vec());
    polyhedron.triangles = PolygonArray { data: icosahedron_indices().to_vec()};
    polyhedron.subdivide_triangles();
    polyhedron.subdivide_triangles();
    goldberg_dual_mesh(&mut polyhedron);

    // For each face in the polyhedron, generate a map tile
    let mut planet_mesh = PlanetMesh::default();
    planet_mesh.outer_frame = polyhedron.clone();
    planet_mesh.outer_frame.reduce_to_lines();
    planet_mesh.face_count = polyhedron.hex.count() + polyhedron.pents.count();
    let mut work_done = 0;
    info!("Generating planet...");
    info!("Total faces: {}", planet_mesh.face_count);

    while let Some(hex) = polyhedron.hex.data.pop() {
        let v_origin = polyhedron.vertices.get(&hex.verts[1]);
        let basis_1 = polyhedron.vertices.get(&hex.verts[0]).pos - v_origin.pos;
        let basis_2 = polyhedron.vertices.get(&hex.verts[2]).pos - v_origin.pos;
        let normal = basis_2.cross(basis_1);
        let tangent = basis_2.cross(normal);

        let mut vertices = Vec::new();
        for idx in hex.verts {
            vertices.push(polyhedron.vertices.get(&idx));
        }
        let offset = center_pos(&vertices).pos;

        let transform = Transform::from_xyz(offset.x, offset.y, offset.z).looking_to(tangent, normal);
        
        let mut mesh_data = MeshData::default();
        mesh_data.set_orientation(Vec3::Y, Vec3::X);
        initialize_stalberg_hex(resolution, &mut mesh_data);
        combine_triangles_to_quads_randomly(&mut mesh_data);

        mesh_data.subdivide_to_quads();
        relax_vertices(&mut mesh_data);
        mesh_data.calc_lines_no_reduce();
        mesh_data.reduce_to_triangles();

        let planet_tile = PlanetTile::<6> {
            outer_polygon: hex,
            transform: transform,
            tile_mesh: mesh_data
        };
        planet_mesh.hex_faces.push(planet_tile);
        work_done += 1;
        info!(" | Progress: {:.2}%", 100.0 * work_done as f32 / planet_mesh.face_count as f32)
    }

    while let Some(pent) = polyhedron.pents.data.pop() {
        let v_origin = polyhedron.vertices.get(&pent.verts[1]);
        let basis_1 = polyhedron.vertices.get(&pent.verts[0]).pos - v_origin.pos;
        let basis_2 = polyhedron.vertices.get(&pent.verts[2]).pos - v_origin.pos;
        let normal = basis_2.cross(basis_1);
        let tangent = basis_2.cross(normal);

        let mut vertices = Vec::new();
        for idx in pent.verts {
            vertices.push(polyhedron.vertices.get(&idx));
        }
        let offset = center_pos(&vertices).pos;

        let mut transform = Transform::from_xyz(offset.x, offset.y, offset.z)
            .looking_to(tangent, normal);
        transform.rotate_local_y(-PI/2.0);
        
        let mut mesh_data = MeshData::default();
        mesh_data.set_orientation(Vec3::Y, Vec3::X);
        initialize_stalberg_pent(resolution, &mut mesh_data);
        combine_triangles_to_quads_randomly(&mut mesh_data);

        mesh_data.subdivide_to_quads();
        relax_vertices(&mut mesh_data);
        mesh_data.calc_lines_no_reduce();
        mesh_data.reduce_to_triangles();

        let planet_tile = PlanetTile::<5> {
            outer_polygon: pent,
            transform: transform,
            tile_mesh: mesh_data
        };
        planet_mesh.pent_faces.push(planet_tile);
        work_done += 1;
        info!(" | Progress: {:.2}%", 100.0 * work_done as f32 / planet_mesh.face_count as f32)
    }
    planet_mesh
}

fn initialize_stalberg_hex(resolution: usize, mesh: &mut MeshData) {
    let mut verts = [VertexIndex::default(); 6];
    for i in 0..6 {
        let pos = mesh.tangent.rotate_axis(mesh.normal, i as f32 * (2.0 * PI) / 6.0).normalize();
        let idx = mesh.vertices.add(Vertex { pos });
        mesh.boundary_indices.push(idx);
        verts[i] = idx;
    }
    mesh.hex.add(Hexagon { verts });
    mesh.reduce_to_triangles();
    for _ in 0..resolution {
        mesh.subdivide_triangles();
    }
}

fn initialize_stalberg_pent(resolution: usize, mesh: &mut MeshData) {
    let mut verts = [VertexIndex::default(); 5];
    for i in 0..5 {
        let pos = mesh.tangent.rotate_axis(mesh.normal, i as f32 * (2.0 * PI) / 5.0);
        let idx = mesh.vertices.add(Vertex { pos });
        mesh.boundary_indices.push(idx);
        verts[i] = idx;
    }
    mesh.pents.add(Pentagon { verts });
    mesh.reduce_to_triangles();
    for _ in 0..resolution {
        mesh.subdivide_triangles();
    }
}

#[derive(Debug)]
struct MergeCandidate {
    edge: (VertexIndex, VertexIndex),
    tr_a: usize,
    tr_b: usize,
    opposite_b: VertexIndex,
}

fn combine_triangles_to_quads_randomly(mesh: &mut MeshData) {
    let edge_map: HashMap<(VertexIndex, VertexIndex), Vec<usize>> = mesh.get_edges_of_triangles();
    let mut candidates = Vec::new();

    for (e, triangles) in edge_map {
        if triangles.len() == 2 {
            let opp_b = mesh.triangles.data[triangles[1]].verts
                .iter().find(|v| **v != e.0 && **v != e.1)
                .expect("Could not find vertex to add");
            candidates.push(MergeCandidate {
                edge: e,
                tr_a: triangles[0],
                tr_b: triangles[1],
                opposite_b: *opp_b
            });
        }
    }

    let mut rng = rand::rng();
    candidates.shuffle(&mut rng);

    let mut used = vec![false; mesh.triangles.count()];
    let mut generated_quads: Vec<Quad> = Vec::new();
    
    while let Some(candidate) = candidates.pop() {
        if used[candidate.tr_a] || used[candidate.tr_b] {
            continue;
        }

        let triangles = (
            &mesh.triangles.data[candidate.tr_a],
            &mesh.triangles.data[candidate.tr_b]
        );
        let mut verts = triangles.0.verts.to_vec();
        let edge_pos = triangles.0.find_edge_positions(&candidate.edge).expect("Could not find edge in triangle");
        verts.insert(edge_pos.0+1, candidate.opposite_b);

        // Create the new quad
        let mut vert_array: [VertexIndex; 4] = [VertexIndex(0); 4];
        for (idx, v) in verts.iter().enumerate() {
            vert_array[idx] = *v;
        }
        let quad = Quad { verts: vert_array };
        generated_quads.push(quad);

        used[candidate.tr_a] = true;
        used[candidate.tr_b] = true;
    }

    // Retrieve the remaining triangles
    let mut remaining_triangles = Vec::new();
    for (idx, is_used) in used.iter().enumerate() {
        if !is_used {
            remaining_triangles.push(mesh.triangles.data[idx]);
        }
    }

    // Commit
    mesh.triangles = PolygonArray { data: remaining_triangles };
    mesh.quads = PolygonArray{ data: generated_quads };
}

fn relax_vertices(mesh: &mut MeshData) {
    let neighbour_map = mesh.generate_neighbour_map();
    let boundary_indices = &mesh.boundary_indices;
    let tolerance = 0.001;
    let relaxation_factor = 0.1;
    let mut max_change = tolerance + 1.0;
    let mut it = 1000;
    while max_change > tolerance {
        max_change = 0.0;
        let vertices = &mut mesh.vertices;
        for (i, neighbours) in neighbour_map.iter().enumerate() {
            if boundary_indices.contains(&VertexIndex(i)) {
                continue;
            }
            let mut average_center = Vec3::ZERO;
            for n in neighbours {
                average_center += vertices.get(n).pos
            }
            average_center /= neighbours.len() as f32;
            let current_pos = vertices.get(&VertexIndex(i)).pos;
            let updated_pos = current_pos + relaxation_factor * (average_center - current_pos);
            vertices.update(&VertexIndex(i), updated_pos);

            let change = (average_center - current_pos).norm();
            max_change = change.max(max_change);
        }

        it -= 1;
        if it == 0 {
            break;
        }
    }
}

fn goldberg_dual_mesh(mesh: &mut MeshData) {
    let binding = VertexArray::default();
    let mut new_vertices = HashedVertexArray::new(
        &binding,
        |id: &usize| {
            let verts = &mesh.triangles.data[*id].verts;
            center_pos(&[
                mesh.vertices.get(&verts[0]),
                mesh.vertices.get(&verts[1]),
                mesh.vertices.get(&verts[2])
                ])
        }
    );

    // Precalculate the adjacent triangles to each vertex in O(n)
    let mut adjacent_triangles: Vec<Vec<usize>> = vec![Vec::new(); mesh.vertices.count()];
    for (tr_idx, tr) in mesh.triangles.data.iter().enumerate() {
        for i in tr.verts {
            adjacent_triangles[i.0].push(tr_idx);
        }
    }

    let mut hexagons = PolygonArray::<6>::default();
    let mut pentagons = PolygonArray::<5>::default();

    // For each vertex, create the surrounding hexagon (and 12 pentagons) as triangle fan
    for (i, vertex) in mesh.vertices.inner().iter().enumerate() {
        // Center of hexagon face
        let adjacent_triangles = &adjacent_triangles[i];
        let center = vertex;

        // Local orientation of hexagon face
        let normal = center.pos.normalize();
        let tangent = if normal.x.abs() < 0.9 {
            normal.cross(Vec3::X).normalize()
        } else {
            normal.cross(Vec3::Y).normalize()
        };
        let bitangent = normal.cross(tangent);

        // Unsorted corner indices and their angles respective to the previously defined tangent
        let mut unsorted_corner_indices: Vec<(f32, VertexIndex)> = Vec::new();
        for i in adjacent_triangles {
            let triangle_idx = new_vertices.get_idx(&i);
            let delta = new_vertices.get_vertex(&triangle_idx).pos - center.pos;

            let x = delta.dot(tangent);
            let y = delta.dot(bitangent);
            let angle = y.atan2(x);

            unsorted_corner_indices.push((angle, triangle_idx));
        }

        // Sort by angle
        unsorted_corner_indices.sort_by(|(angle1, _), (angle2, _)| angle1.total_cmp(&angle2));

        // Copy data and retrieve corner vertex positions
        let mut sorted_corners = Vec::new();
        let mut sorted_corner_vertices = Vec::new();
        for (_, i) in unsorted_corner_indices {
            sorted_corners.push(i);
            sorted_corner_vertices.push(new_vertices.get_vertex(&i));
        }

        // Create the face
        if sorted_corners.len() == 6 {
            hexagons.add(Hexagon { verts: [
                sorted_corners[0],
                sorted_corners[1],
                sorted_corners[2],
                sorted_corners[3],
                sorted_corners[4],
                sorted_corners[5],
            ]});
        } else if sorted_corners.len() == 5 {
            pentagons.add(Pentagon { verts: [
                sorted_corners[0],
                sorted_corners[1],
                sorted_corners[2],
                sorted_corners[3],
                sorted_corners[4],
            ]});
        } else {
            panic!("Encountered face which is no pentagon or hexagon");
        }
    }

    mesh.vertices = new_vertices.retrieve_data();
    mesh.normalize();
    mesh.triangles = PolygonArray::default();
    mesh.pents = pentagons;
    mesh.hex = hexagons;
}

fn barycentric_projection<const N:usize> (
    p: &mut Vec3,
    origin: &[Vec3; N],
    destination: &[Vec3; N]
) {
    let e1 = (origin[1] - origin[0]).normalize();
    let normal = ((origin[1] - origin[0]).cross(origin[2] - origin[0])).normalize();
    let e2 = normal.cross(e1);

    let uv: Vec2 = Vec2 { x: p.dot(e1), y: p.dot(e2) };

    let r: Vec<Vec2> =
        origin.iter()
            .map(|v| Vec2 {x: (*v - *p).dot(e1), y: (*v - *p).dot(e2)})
            .collect();

    let d: Vec<f32> =
        r.iter()
        .map(|r| r.length())
        .collect();

    let mut theta = vec![0.0; N];

    for i in 0..N {
        let j = (i + 1) % N;
        theta[i] = r[i].angle_to(r[j]);
    }

    let mut w = vec![0.0; N];

    let eps = 0.00001;
    for i in 0..N {
        let prev = (i + N - 1) % N;

        if d[i] < eps {
            *p = destination[i];
            return;
        }

        w[i] =
            ((theta[prev] * 0.5).tan()
            + (theta[i]    * 0.5).tan())
            / d[i];
    }

    let sum: f32 = w.iter().sum();

    for wi in &mut w {
        *wi /= sum;
    }

    let mut warped = Vec3::ZERO;

    for i in 0..N {
        warped += destination[i] * w[i];
    }

    *p = warped;
}

fn icosahedron_indices() -> [Triangle; 20] {
    [
        Triangle { verts: [VertexIndex(9),  VertexIndex(6),  VertexIndex(3)] },
        Triangle { verts: [VertexIndex(9),  VertexIndex(3), VertexIndex(11)] },
        Triangle { verts: [VertexIndex(9), VertexIndex(11),  VertexIndex(2)] },
        Triangle { verts: [VertexIndex(9),  VertexIndex(2),  VertexIndex(4)] },
        Triangle { verts: [VertexIndex(9),  VertexIndex(4),  VertexIndex(6)] },

        Triangle { verts: [VertexIndex(11), VertexIndex(3),  VertexIndex(7)] },
        Triangle { verts: [VertexIndex(3),  VertexIndex(6),  VertexIndex(1)] },
        Triangle { verts: [VertexIndex(6),  VertexIndex(4),  VertexIndex(8)] },
        Triangle { verts: [VertexIndex(4),  VertexIndex(2),  VertexIndex(0)] },
        Triangle { verts: [VertexIndex(2), VertexIndex(11),  VertexIndex(5)] },

        Triangle { verts: [VertexIndex(10), VertexIndex(7),  VertexIndex(1)] },
        Triangle { verts: [VertexIndex(10), VertexIndex(1),  VertexIndex(8)] },
        Triangle { verts: [VertexIndex(10), VertexIndex(8),  VertexIndex(0)] },
        Triangle { verts: [VertexIndex(10), VertexIndex(0),  VertexIndex(5)] },
        Triangle { verts: [VertexIndex(10), VertexIndex(5),  VertexIndex(7)] },

        Triangle { verts: [VertexIndex(1),  VertexIndex(7),  VertexIndex(3)] },
        Triangle { verts: [VertexIndex(8),  VertexIndex(1),  VertexIndex(6)] },
        Triangle { verts: [VertexIndex(0),  VertexIndex(8),  VertexIndex(4)] },
        Triangle { verts: [VertexIndex(5),  VertexIndex(0),  VertexIndex(2)] },
        Triangle { verts: [VertexIndex(7),  VertexIndex(5), VertexIndex(11)] },
    ]
}

fn icosahedron_vertices() -> [Vertex; 12] {
    let sqrt5 = sqrt(5.0);
    let golden_ratio = (1.0 + sqrt5) / 2.0;

    let mut vertices = [Vertex::default(); 12];
    let mut idx = 0;
    for i in [-1.0, 1.0] {
        for j in [-1.0, 1.0] {
            vertices[idx].pos = Vec3 { x: 0.0, y: i, z: j * golden_ratio};
            vertices[idx + 4].pos = Vec3 { x: j * golden_ratio, y: 0.0, z: i};
            vertices[idx + 8].pos = Vec3 { x: i, y: j * golden_ratio, z: 0.0};

            idx += 1;
        }
    }
    for v in &mut vertices {
        v.pos /= sqrt5;
    }

    vertices
}