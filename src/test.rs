use glam::{vec3, Quat, Vec3};
use graphics::{self, camera, model::{self, DrawModel, ModelVertex, Vertex, ModelInstance, ModelInstanceRaw}, resources, texture::Texture, window::create_render_pipeline, App};
use wgpu::{util::DeviceExt, Queue, RenderPass};

const NUM_INSTANCES_PER_ROW: u32 = 10;

fn main() {
    env_logger::init();
    let game = Game::new();
    graphics::run(Box::new(game));
}

struct GameState {
    render_pipeline: wgpu::RenderPipeline,
    camera: camera::CameraController,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    light_buffer: wgpu::Buffer,
    light_bind_group: wgpu::BindGroup,
    light_render_pipeline: wgpu::RenderPipeline,
    instances: Vec<ModelInstance>,
    instance_buffer: wgpu::Buffer,
    obj_model: model::MatModel,
}

struct Game {
    state: Option<GameState>,
    light_uniform: LightUniform,
}

impl Game {
    fn new() -> Self {
        let light_uniform = LightUniform {
            position: vec3(2.0, 2.0, 2.0),
            _padding: 0,
            color: vec3(1.0, 1.0, 1.0),
            _padding2: 0,
        };

        Self {
            state: None,
            light_uniform,
        }
    }

    fn state(&self) -> &GameState {
        if let Some(state) = &self.state {
            state
        } else {
            panic!("GameState does not exist")
        }
    }

    fn state_mut(&mut self) -> &mut GameState {
        if let Some(state) = &mut self.state {
            state
        } else {
            panic!("GameState does not exist")
        }
    }

    fn update_camera_buffer(&mut self, queue: &Queue) {
        queue.write_buffer(&self.state().camera_buffer, 0, bytemuck::cast_slice(&[self.state().camera.camera.uniform]));
    }
}

impl App for Game {
    fn update(
        &mut self,
        queue: &Queue,
    ) {
        let old_position: Vec3 = self.light_uniform.position.into();
        self.light_uniform.position =
            (Quat::from_axis_angle((0.0, 1.0, 0.0).into(), 1.0_f32.to_radians())
                * old_position)
                .into();
        queue.write_buffer(&self.state().light_buffer, 0, bytemuck::cast_slice(&[self.light_uniform]));
    }

    fn window_event(
        &mut self,
        event: &winit::event::WindowEvent,
        queue: &Queue,
    ) -> bool {
        let (processed, update_camera) = self.state_mut().camera.window_event(event);
        if processed {
            if update_camera {
                self.update_camera_buffer(queue);
            }
            return true;
        }
        false
    }

    fn device_event(
        &mut self,
        event: &winit::event::DeviceEvent,
        queue: &Queue,
    ) {
        if self.state_mut().camera.device_event(event) {
            self.update_camera_buffer(queue);
        }
    }

    fn render(
        &mut self,
        render_pass: &mut RenderPass,
    ) {
        unsafe {
            render_pass.set_vertex_buffer(1, std::mem::transmute(self.state().instance_buffer.slice(..)));

            render_pass.set_pipeline(to_static(&self.state().light_render_pipeline));
            render_pass.draw_model_no_mat(
                to_static(&self.state().obj_model),
                to_static_array(&[&self.state().camera_bind_group, &self.state().light_bind_group]),
            );

            render_pass.set_pipeline(to_static(&self.state().render_pipeline));
            render_pass.draw_model_instanced(
                to_static(&self.state().obj_model),
                0..self.state().instances.len() as u32,
                to_static_array(&[&self.state().camera_bind_group, &self.state().light_bind_group]),
            );
        }
    }
    
    fn setup(
        &mut self,
        queue: &Queue,
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
    ) {

        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        });

        let mut camera = camera::Camera::new(
            (0.0, 1.0, 2.0).into(),
            (0.0, 0.0, 0.0).into(),
            Vec3::Y,
            config.width as f32 / config.height as f32,
            45.0,
            0.1,
            100.0,
        );
        camera.build_view_projection_matrix();
        let camera = camera::CameraController::new(camera, 0.004);

        let camera_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera.camera.uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("camera_bind_group_layout"),
        });
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }
            ],
            label: Some("camera_bind_group"),
        });

        let light_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Light VB"),
                contents: bytemuck::cast_slice(&[self.light_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let light_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: None,
        });
    
        let light_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &light_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: light_buffer.as_entire_binding(),
            }],
            label: None,
        });

        let render_pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &texture_bind_group_layout,
                    &camera_bind_group_layout,
                    &light_bind_group_layout,
                ],
                push_constant_ranges: &[],
            }
        );
        let render_pipeline = {
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Normal Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("lit.wgsl").into()),
            };
            create_render_pipeline(
                &device,
                &render_pipeline_layout,
                config.format,
                Some(Texture::DEPTH_FORMAT),
                &[ModelVertex::desc(), ModelInstanceRaw::desc()],
                shader,
            )
        };
        
        let light_render_pipeline = {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Light Pipeline Layout"),
                bind_group_layouts: &[
                    &camera_bind_group_layout,
                    &light_bind_group_layout,
                    ],
                push_constant_ranges: &[],
            });
            let shader = wgpu::ShaderModuleDescriptor {
                label: Some("Light Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("light.wgsl").into()),
            };
            create_render_pipeline(
                &device,
                &layout,
                config.format,
                Some(Texture::DEPTH_FORMAT),
                &[ModelVertex::desc()],
                shader,
            )
        };

        const SPACE_BETWEEN: f32 = 3.0;
        let instances = (0..NUM_INSTANCES_PER_ROW).flat_map(|z| {
            (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                let x = SPACE_BETWEEN * (x as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
                let z = SPACE_BETWEEN * (z as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);

                let position = vec3(x, 0.0, z);

                let rotation = if position == Vec3::ZERO {
                    Quat::from_axis_angle(Vec3::Z, 0.0)
                } else {
                    Quat::from_axis_angle(position.normalize(), 45.0_f32.to_radians())
                };

                ModelInstance::new(position, rotation)
            })
        }).collect::<Vec<_>>();

        let instance_data = instances.iter().map(ModelInstance::to_raw).collect::<Vec<_>>();
        let instance_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&instance_data),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        let obj_model = resources::load_model("cube.obj", &device, &queue, &texture_bind_group_layout).unwrap();

        self.state = Some(GameState {
            render_pipeline,
            camera,
            camera_buffer,
            camera_bind_group,
            light_buffer,
            light_bind_group,
            light_render_pipeline,
            instances,
            instance_buffer,
            obj_model,
        });
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct LightUniform {
    position: Vec3,
    // Due to uniforms requiring 16 byte (4 float) spacing, we need to use a padding field here
    _padding: u32,
    color: Vec3,
    // Due to uniforms requiring 16 byte (4 float) spacing, we need to use a padding field here
    _padding2: u32,
}

unsafe fn to_static<T>(src: &T) -> &'static T {
    std::mem::transmute(src)
}

unsafe fn to_static_array<T>(src: &[&T]) -> &'static [&'static T] {
    std::mem::transmute(src)
}