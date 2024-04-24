use std::ops::Range;
use glam::{Mat3, Mat4, Quat, Vec2, Vec3};

use crate::texture;

pub trait Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static>;
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelVertex {
    pub position: Vec3,
    pub tex_coords: Vec2,
    pub normal: Vec3,
    pub tangent: Vec3,
    pub bitangent: Vec3,
}

impl Vertex for ModelVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<ModelVertex>() as wgpu::BufferAddress,
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
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 11]>() as wgpu::BufferAddress,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

pub struct MatModel {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<ModelMaterial>,
}

pub struct NoMatModel {
    pub meshes: Vec<Mesh>,
}

pub trait HasMeshes {
    fn meshes<'a>(
        &'a self,
    ) -> &'a Vec<Mesh>;
}
pub trait HasMaterials {
    fn materials<'a>(
        &'a self,
    ) -> &'a Vec<impl Material>;
}

impl HasMeshes for MatModel {
    fn meshes<'a>(
        &'a self,
    ) -> &'a Vec<Mesh> {
        &self.meshes
    }
}

impl HasMeshes for NoMatModel {
    fn meshes<'a>(
        &'a self,
    ) -> &'a Vec<Mesh> {
        &self.meshes
    }
}

impl HasMaterials for MatModel {
    fn materials<'a>(
        &'a self,
    ) -> &'a Vec<impl Material> {
        &self.materials
    }
}

pub struct ModelMaterial {
    pub name: String,
    pub diffuse_texture: texture::Texture,
    pub normal_texture: texture::Texture,
    pub bind_group: wgpu::BindGroup,
}

impl ModelMaterial {
    pub fn new(
        device: &wgpu::Device,
        name: &str,
        diffuse_texture: texture::Texture,
        normal_texture: texture::Texture,
        layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&normal_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&normal_texture.sampler),
                },
            ],
            label: Some(name),
        });

        Self {
            name: String::from(name),
            diffuse_texture,
            normal_texture,
            bind_group,
        }
    }
}

pub trait Material {
    fn bind_group<'a>(
        &'a self,
    ) -> &'a wgpu::BindGroup;
}

impl Material for ModelMaterial {
    fn bind_group<'a>(
        &'a self,
    ) -> &'a wgpu::BindGroup {
        &self.bind_group
    }
}

pub struct Mesh {
    pub name: String,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
    pub material: usize,
}

pub trait DrawModel<'a> {
    fn draw_mesh(
        &mut self,
        mesh: &'a Mesh,
        material: Option<&'a impl Material>,
        bind_groups: &[&'a wgpu::BindGroup],
    );
    fn draw_mesh_instanced(
        &mut self,
        mesh: &'a Mesh,
        material: Option<&'a impl Material>,
        instances: Range<u32>,
        bind_groups: &[&'a wgpu::BindGroup],
    );

    fn draw_model(
        &mut self,
        model: &'a (impl HasMeshes + HasMaterials),
        bind_groups: &[&'a wgpu::BindGroup],
    );
    fn draw_model_instanced(
        &mut self,
        model: &'a (impl HasMeshes + HasMaterials),
        instances: Range<u32>,
        bind_groups: &[&'a wgpu::BindGroup],
    );

    fn draw_model_no_mat(
        &mut self,
        model: &'a impl HasMeshes,
        bind_groups: &[&'a wgpu::BindGroup],
    );
    fn draw_model_no_mat_instanced(
        &mut self,
        model: &'a impl HasMeshes,
        instances: Range<u32>,
        bind_groups: &[&'a wgpu::BindGroup],
    );
}

impl<'a, 'b> DrawModel<'b> for wgpu::RenderPass<'a>
where
    'b: 'a,
{
    fn draw_mesh (
        &mut self,
        mesh: &'a Mesh,
        material: Option<&'a impl Material>,
        bind_groups: &[&'a wgpu::BindGroup],
    ) {
        self.draw_mesh_instanced(mesh, material, 0..1, bind_groups);
    }

    fn draw_mesh_instanced(
        &mut self,
        mesh: &'a Mesh,
        material: Option<&'a impl Material>,
        instances: Range<u32>,
        bind_groups: &[&'a wgpu::BindGroup],
    ) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        let mut index = if let Some(material) = material {
            self.set_bind_group(0, material.bind_group(), &[]);
            1
        } else {
            0
        };
        for bind_group in bind_groups {
            self.set_bind_group(index, &bind_group, &[]);
            index += 1;
        }
        self.draw_indexed(0..mesh.num_elements, 0, instances);
    }

    fn draw_model(
        &mut self,
        model: &'a (impl HasMeshes + HasMaterials),
        bind_groups: &[&'a wgpu::BindGroup],
    ) {
        self.draw_model_instanced(model, 0..1, bind_groups);
    }

    fn draw_model_instanced(
        &mut self,
        model: &'a (impl HasMeshes + HasMaterials),
        instances: Range<u32>,
        bind_groups: &[&'a wgpu::BindGroup],
    ) {
        let meshes = model.meshes();
        let materials = model.materials();
        for mesh in meshes {
            let material = &materials[mesh.material];
            self.draw_mesh_instanced(mesh, Some(material), instances.clone(), bind_groups);
        }
    }
    
    fn draw_model_no_mat(
        &mut self,
        model: &'b impl HasMeshes,
        bind_groups: &[&'b wgpu::BindGroup],
    ) {
        self.draw_model_no_mat_instanced(model, 0..1, bind_groups);
    }
    
    fn draw_model_no_mat_instanced(
        &mut self,
        model: &'b impl HasMeshes,
        instances: Range<u32>,
        bind_groups: &[&'b wgpu::BindGroup],
    ) {
        let meshes = model.meshes();
        for mesh in meshes {
            self.draw_mesh_instanced(mesh, None::<&ModelMaterial>, instances.clone(), bind_groups);
        }
    }
}

pub struct ModelInstance {
    position: Vec3,
    rotation: Quat,
}

impl ModelInstance {
    pub fn new(position: Vec3, rotation: Quat) -> Self {
        Self { position, rotation }
    }

    pub fn to_raw(&self) -> ModelInstanceRaw {
        ModelInstanceRaw {
            model: Mat4::from_translation(self.position) * Mat4::from_quat(self.rotation),
            normal: Mat3::from_quat(self.rotation),
            _padding: [0; 3],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ModelInstanceRaw {
    model: Mat4,
    normal: Mat3,
    _padding: [u32; 3],
}

impl ModelInstanceRaw {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<ModelInstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 19]>() as wgpu::BufferAddress,
                    shader_location: 10,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 22]>() as wgpu::BufferAddress,
                    shader_location: 11,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}