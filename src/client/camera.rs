#[derive(Debug, Copy, Clone)]
pub struct ClientCamera {
    camera: crate::camera::Camera,
}

impl ClientCamera {
    pub fn new(pos: uv::Vec3, yaw: f32, pitch: f32) -> Self {
        Self {
            camera: crate::camera::Camera::new(pos, yaw, pitch),
        }
    }

    const MOVE_KEYS: [winit::event::VirtualKeyCode; 6] = [
        winit::event::VirtualKeyCode::W,
        winit::event::VirtualKeyCode::S,
        winit::event::VirtualKeyCode::A,
        winit::event::VirtualKeyCode::D,
        winit::event::VirtualKeyCode::Space,
        winit::event::VirtualKeyCode::LShift,
    ];

    pub fn update(&mut self, state: &super::window::ClientState) {
        self.camera.update_orientation(state.mouse_rel() / -60.);

        self.camera.pos += self.camera.move_mat()
            * state.move_vec(&Self::MOVE_KEYS)
            * 30.
            * state.frame_elapsed().as_secs_f32();
    }

    pub fn camera(&self) -> crate::camera::Camera {
        self.camera
    }
}
