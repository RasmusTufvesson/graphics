mod window;

fn main() {
    env_logger::init();
    let _ = pollster::block_on(window::run());
}
