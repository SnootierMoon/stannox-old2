mod camera;
mod window;

use crate::vk::Instance;
use crate::*;

pub fn run() -> ! {
    let event_loop = winit::event_loop::EventLoop::new();
    let object = voxel::Object::new_test();
    let window = window::ClientWindow::new(&event_loop);

    let render_instance = vk::WindowedInstance::new(window.window(), true);
    let mut render_surface = vk::Swapchain::new(render_instance.clone(), window.size().into());
    let mut voxel_renderer = vk::VoxelMeshRenderer::new(render_instance.clone(), &render_surface);
    let mut voxel_manager = vk::VoxelMeshManager::new(render_instance.clone());

    for mesh in &object.fuck_it_mesh_all() {
        voxel_manager.upload_mesh(mesh)
    }

    println!("{}", voxel_manager.mesh_count());

    let mut camera = camera::ClientCamera::new(
        uv::Vec3::new(-90.0, 40.0, 40.0),
        0.0,
        std::f32::consts::FRAC_PI_2,
    );

    window.run(event_loop, move |window, state| {
        if state.quit() {
            render_instance.wait_idle();
            return;
        }
        camera.update(state);
        println!("{:?}", camera);
        if !render_surface.render(|command_buffer| {
            voxel_renderer.render(command_buffer, &voxel_manager, &camera.camera());
        }) {
            render_instance.wait_idle();
            render_surface.rebuild(window.size().into());
            voxel_renderer.rebuild(&render_surface);
        }
    });
}
