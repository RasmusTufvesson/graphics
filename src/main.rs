mod window;
mod texture;
mod camera;

fn main() {
    env_logger::init();
    let _ = pollster::block_on(window::run());
}
