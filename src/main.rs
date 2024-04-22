mod window;
mod texture;
mod camera;
mod model;
mod resources;

fn main() {
    env_logger::init();
    let _ = pollster::block_on(window::run());
}
