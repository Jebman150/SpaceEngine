mod vertices;

use bevy::{asset::RenderAssetUsages, math::ops::sqrt, mesh::{Indices, PrimitiveTopology},prelude::*};

use crate::world::celestial::IntensityMap;
use vertices::*;

#[derive(Clone, Debug)]
pub enum Shading {
    Flat,
    Smooth
}

impl Default for Shading {
    fn default() -> Self {
        Shading::Flat
    }
}

#[derive(Clone, Debug, Default)]
pub struct MeshDescriptor {
    pub radius: f32,
    pub subdivisions: usize,
    pub shading: Shading,
}

pub fn generate_mesh(
    mesh_descriptor: MeshDescriptor,
    _height_map: &IntensityMap) -> Mesh {
    let ico_vertices = icosahedron_vertices().to_vec();
    let mut ico_triangles = icosahedron_indices().to_vec();

    let mut ico_vertices = VertexArray::new(ico_vertices);

    subdivide_triangles(&mut ico_vertices, &mut ico_triangles);
    for v in ico_vertices.ref_mut() {
        v.pos = v.pos.normalize();
        v.pos *= mesh_descriptor.radius;
    }

    let (vertices, triangles) = goldberg_dual_mesh(&ico_vertices, &ico_triangles);

    let (raw_vertices, raw_indices) =
    match mesh_descriptor.shading {
        Shading::Flat => {
            vertices.to_raw_flat_shading(triangles)
        }
        Shading::Smooth => {(
            vertices.to_raw(),
            to_raw_indices(&triangles)
        )}
        
    };

    Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD)
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, raw_vertices)
        .with_inserted_indices(Indices::U32(raw_indices))
}

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

fn subdivide_triangles(vertices: &mut VertexArray, triangles: &mut Vec<Triangle>) {
    let mut edge_centers = HashedVertexArray::new(
        vertices,
        |id: &(VertexIndex, VertexIndex)| {
            center_pos(&[vertices.get(&id.0), vertices.get(&id.1)])
        }
    );

    let mut new_triangles: Vec<Triangle> = Vec::new();
    for triangle in &mut *triangles {
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
        }

        triangle.verts[0] = edges[0].0;
        triangle.verts[1] = mid_points[0];
        triangle.verts[2] = mid_points[2];

        new_triangles.push(
            Triangle { verts: [
                mid_points[0],
                edges[0].1,
                mid_points[1]
            ]}
        );
        new_triangles.push(
            Triangle { verts: [
                mid_points[1],
                edges[1].1,
                mid_points[2]
            ]}
        );
        new_triangles.push(
            Triangle { verts: [
                mid_points[0],
                mid_points[1],
                mid_points[2]
            ]}
        );
    }
    vertices.append(edge_centers.retrieve_data());
    triangles.append(&mut new_triangles);
}

fn center_pos(vertices: &[&Vertex]) -> Vertex {
    let mut sum = Vec3::ZERO;
    for v in vertices {
        sum += v.pos;
    }
    sum /= vertices.len() as f32;
    Vertex{ pos: sum }
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