use ultraviolet::*;

pub struct Camera {
    pub aspect_w_div_h: f32,
    pub vertical_fov_in_rad: f32,
    pub z_near: f32,
    pub z_far: f32,
    pub projection_matrix: Mat4,
}

impl Camera {
    pub fn new(aspect_w_div_h: f32, vertical_fov_in_rad: f32, z_near: f32, z_far: f32) -> Self {
        Self {
            aspect_w_div_h,
            vertical_fov_in_rad,
            z_near,
            z_far,

            projection_matrix: projection::rh_yup::perspective_wgpu_dx(
                vertical_fov_in_rad,
                aspect_w_div_h,
                z_near,
                z_far,
            ),
        }
    }
}
