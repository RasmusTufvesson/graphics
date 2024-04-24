use std::f32::consts::TAU;
use glam::{vec2, vec3, Vec2, Vec3};
use wgpu::{util::DeviceExt, Buffer};
use crate::model::{Mesh, Vertex};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct SimpleVertex {
    pub position: Vec3,
    pub tex_coords: Vec2,
}

impl Vertex for SimpleVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<SimpleVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

fn direction(angle: f32) -> Vec2 {
    vec2(angle.cos(), angle.sin())
}

fn create_buffers(name: &str, vertices: &Vec<SimpleVertex>, indices: &Vec<u32>, device: &wgpu::Device) -> (Buffer, Buffer) {
    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(&format!("{:?} Vertex Buffer", name)),
        contents: bytemuck::cast_slice(vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });
    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(&format!("{:?} Index Buffer", name)),
        contents: bytemuck::cast_slice(indices),
        usage: wgpu::BufferUsages::INDEX,
    });
    (vertex_buffer, index_buffer)
}

pub fn circle(num_points: u32, radius: f32, material: usize, device: &wgpu::Device) -> Mesh {
    let mut points = vec![];
    let mut indices = vec![];
    let angle = TAU / num_points as f32;
    for i in 0..num_points {
        let angle = angle * i as f32;
        let vector = direction(angle);
        points.push(SimpleVertex { position: vec3(vector.x, vector.y, 0.0) * radius, tex_coords: vector });
        if i > 0 {
            indices.push(0);
            indices.push(i);
            indices.push(i - 1);
        }
    }
    indices.push(0);
    indices.push(num_points - 1);
    indices.push(0);

    let (vertex_buffer, index_buffer) = create_buffers("Circle", &points, &indices, device);
    Mesh { name: "Circle".to_owned(), vertex_buffer, index_buffer, num_elements: indices.len() as u32, material: material }
}