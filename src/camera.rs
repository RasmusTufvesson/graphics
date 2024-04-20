use glam::{Mat4, Quat, Vec3};
use winit::event::{DeviceEvent, ElementState, MouseButton, MouseScrollDelta, WindowEvent};

pub struct Camera {
    eye: Vec3,
    target: Vec3,
    up: Vec3,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
    pub uniform: CameraUniform,
}

impl Camera {
    pub fn new(eye: Vec3, target: Vec3, up: Vec3, aspect: f32, fovy: f32, znear: f32, zfar: f32) -> Self {
        Self { eye, target, up, aspect, fovy, znear, zfar, uniform: CameraUniform::new() }
    }

    pub fn build_view_projection_matrix(&mut self) {
        let view = Mat4::look_at_rh(self.eye, self.target, self.up);
        let proj = Mat4::perspective_rh(self.aspect, self.fovy.to_radians(), self.znear, self.zfar);
        self.uniform.view_proj = OPENGL_TO_WGPU_MATRIX * proj * view;
    }
}

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: glam::Mat4 = glam::Mat4::from_cols_array(&[
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
]);

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: Mat4,
}

impl CameraUniform {
    fn new() -> Self {
        Self {
            view_proj: Mat4::IDENTITY,
        }
    }
}

pub struct CameraController {
    pub camera: Camera,
    rotation: Quat,
    distance: f32,
    mouse_down: bool,
    sensitivity: f32,
}

impl CameraController {
    pub fn new(camera: Camera, sensitivity: f32) -> Self {
        let distance = camera.eye.distance(camera.target);
        Self { camera: camera, rotation: Quat::from_euler(glam::EulerRot::XYZ, 0.0, std::f32::consts::PI, 0.0), distance, mouse_down: false, sensitivity }
    }

    pub fn window_event(&mut self, event: &WindowEvent) -> (bool, bool) {
        match event {
            WindowEvent::MouseInput {
                device_id: _,
                state,
                button
            } if button == &MouseButton::Left => {
                self.mouse_down = state == &ElementState::Pressed;
                (true, false)
            },
            WindowEvent::MouseWheel {
                device_id: _,
                delta,
                phase: _
            } => {
                match delta {
                    MouseScrollDelta::LineDelta(_, y) => {
                        self.distance = (self.distance - 0.2 * y).max(0.5);
                    }
                    MouseScrollDelta::PixelDelta(delta) => {
                        self.distance = (self.distance - 0.2 * delta.y as f32).max(0.5);
                    }
                }
                self.update_camera();
                (true, true)
            }
            _ => (false, false),
        }
    }

    pub fn device_event(&mut self, event: &DeviceEvent) -> bool {
        match event {
            DeviceEvent::MouseMotion {
                delta
            } if self.mouse_down => {
                self.rotation *= Quat::from_euler(glam::EulerRot::XYZ, -delta.1 as f32 * self.sensitivity, delta.0 as f32 * self.sensitivity, 0.0);
                self.update_camera();
                true
            },
            _ => false,
        }
    }

    fn update_camera(&mut self) {
        let eye_direction = self.rotation * Vec3::Z;
        self.camera.eye = self.camera.target + -eye_direction * self.distance;
        self.camera.build_view_projection_matrix();
    }
}