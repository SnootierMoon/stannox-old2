#[derive(Debug, Copy, Clone)]
pub struct Camera {
    pub pos: uv::Vec3,
    yaw: f32,
    pitch: f32,
}

impl Camera {
    pub fn new(pos: uv::Vec3, yaw: f32, pitch: f32) -> Self {
        Self { pos, yaw, pitch }
    }

    pub fn move_mat(&self) -> uv::Mat3 {
        uv::Mat3::from_rotation_z(self.yaw)
    }

    pub fn look_mat(&self) -> uv::Mat4 {
        uv::Mat4::from_rotation_x(-self.pitch)
            * uv::Mat4::from_rotation_z(std::f32::consts::FRAC_PI_2 - self.yaw)
            * uv::Mat4::from_translation(-self.pos)
    }

    pub fn update_orientation(&mut self, d: uv::Vec2) {
        self.yaw = (self.yaw + d.x).rem_euclid(std::f32::consts::TAU);
        self.pitch = (self.pitch + d.y).clamp(0.0, std::f32::consts::PI)
    }
}
