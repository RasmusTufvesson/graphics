use wgpu::{Device, Queue, RenderPass};
use winit::event::{DeviceEvent, WindowEvent};

pub mod window;
pub mod texture;
pub mod camera;
pub mod model;
pub mod resources;
pub mod shapes;

pub trait App {
    fn update(
        &mut self,
        queue: &Queue,
    );

    fn window_event(
        &mut self,
        event: &WindowEvent,
        queue: &Queue,
    ) -> bool;
    
    fn device_event(
        &mut self,
        event: &DeviceEvent,
        queue: &Queue,
    );

    fn render(
        &mut self,
        render_pass: &mut RenderPass,
    );

    fn setup(
        &mut self,
        queue: &Queue,
        device: &Device,
        config: &wgpu::SurfaceConfiguration,
    );
}

pub fn run(app: Box<dyn App>) {
    let _ = pollster::block_on(window::run(app));
}