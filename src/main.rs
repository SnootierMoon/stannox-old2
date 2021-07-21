#![allow(dead_code)]

mod camera;
mod client;
mod vk;
mod voxel;

fn main() {
    env_logger::init();
    client::run()
}
