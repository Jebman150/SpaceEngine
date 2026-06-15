mod vertices;

use core::f32;
use std::f32::consts::PI;

use bevy::{asset::RenderAssetUsages, math::NormedVectorSpace, mesh::{Indices, PrimitiveTopology}, platform::collections::HashMap, prelude::*};
use rand::seq::SliceRandom;

use crate::world::celestial::IntensityMap;
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
pub struct MeshDescriptor {
    pub radius: f32,
    pub subdivisions: usize,
    pub mode: RenderMode,
}

pub fn generate_mesh(
    mesh_descriptor: MeshDescriptor,
    _height_map: &IntensityMap
) -> Mesh {
    let mut mesh_data = MeshData::default();
    mesh_data.set_orientation(Vec3::Y, Vec3::X);
    initialize_stalberg(2, &mut mesh_data);
    combine_triangles_to_quads_randomly(&mut mesh_data);

    mesh_data.subdivide_to_quads();
    relax_vertices(&mut mesh_data);
    mesh_data.scale(mesh_descriptor.radius);

    /*
    let ico_vertices = icosahedron_vertices().to_vec();
    let mut ico_triangles = icosahedron_indices().to_vec();

    let mut ico_vertices = VertexArray::new(ico_vertices);

    subdivide_triangles(&mut ico_vertices, &mut ico_triangles);
    for v in ico_vertices.ref_mut() {
        v.pos = v.pos.normalize();
        v.pos *= mesh_descriptor.radius;
    }

    let (_vertices, _triangles) = goldberg_dual_mesh(&ico_vertices, &ico_triangles);
    */

    let ((raw_vertices, raw_indices), index_mode) =
    match mesh_descriptor.mode {
        RenderMode::Flat => {(
            mesh_data.to_raw_flat_shading(),
            PrimitiveTopology::TriangleList
        )}
        RenderMode::Smooth => {(
            mesh_data.to_raw_triangles(),
            PrimitiveTopology::TriangleList
        )}
        RenderMode::Frame => {
            (
            mesh_data.to_raw_lines(),
            PrimitiveTopology::LineList
        )}
    };
    let usage = RenderAssetUsages::RENDER_WORLD;

    Mesh::new(index_mode, usage)
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, raw_vertices)
        .with_inserted_indices(Indices::U32(raw_indices))
}

fn initialize_stalberg(resolution: usize, mesh: &mut MeshData) {
    let mut verts = [VertexIndex::default(); 6];
    for i in 0..6 {
        let pos = mesh.tangent.rotate_axis(mesh.normal, i as f32 * PI/3.0);
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
    let tolerance = 0.0001;
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
    info!("It: {}", it);
}

/*
fn goldberg_dual_mesh(vertices: &VertexArray, triangles: &[Triangle]) -> (VertexArray, Vec<Triangle>) {
    let binding = VertexArray::default();
    let mut new_vertices = HashedVertexArray::new(
        &binding,
        |id: &usize| {
            center_pos(&[
                vertices.get(&triangles[*id].verts[0]),
                vertices.get(&triangles[*id].verts[1]),
                vertices.get(&triangles[*id].verts[2])
                ])
        }
    );

    let mut triangle_fans: Vec<Triangle> = Vec::new();

    // Precalculate the adjacent triangles to each vertex in O(n)
    let mut adjacent_triangles: Vec<Vec<usize>> = vec![Vec::new(); vertices.count()];
    for (tr_idx, tr) in triangles.iter().enumerate() {
        for i in tr.verts {
            adjacent_triangles[i.0].push(tr_idx);
        }
    }

    // For each vertex, create the surrounding hexagon (and 12 pentagons) as triangle fan
    for i in 0..vertices.count() {
        // Center of hexagon face
        let center_idx = VertexIndex(i);
        let adjacent_triangles = &adjacent_triangles[i];
        let center = vertices.get(&center_idx);

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

        // Generate new center from corners, which is now guarenteed to form a flat face with the corners
        let center_vertex = center_pos(&sorted_corner_vertices);
        let center_idx = new_vertices.insert_unmapped(&center_vertex);

        // Create the actual triangle fan
        for (i, corner) in sorted_corners.iter().enumerate() {
            let next_corner = sorted_corners[(i + 1) % sorted_corners.len()];

            triangle_fans.push(
                Triangle { verts: [
                    center_idx,
                    *corner,
                    next_corner
                ]}
            );
        }
    }
    (new_vertices.retrieve_data(), triangle_fans)
}

fn to_raw_indices(triangles: &[Triangle]) -> Vec<u32> {
    let mut result: Vec<u32> = Vec::new();
    for t in triangles {
        for i in t.verts {
            result.push(i.0 as u32);
        }
    }
    result
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

    */