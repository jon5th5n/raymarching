use macroquad::prelude::{Mat3, Vec3};

pub struct RMCamera {
    width: u32,
    height: u32,
    fov: f32,
    depth: f32,
    position: Vec3,
    direction: Vec3,
    up: Vec3,
    right: Vec3,
}
impl RMCamera {
    pub fn new(
        width: u32,
        height: u32,
        fov: f32,
        depth: f32,
        position: Vec3,
        direction: Vec3,
        up: Vec3,
    ) -> Self {
        Self {
            width,
            height,
            fov,
            depth,
            position,
            direction: direction.normalize(),
            up: up.normalize(),
            right: -direction.cross(up).normalize(),
        }
    }

    pub fn get_width(&self) -> u32 {
        self.width
    }
    pub fn set_width(&mut self, width: u32) {
        self.width = width;
    }
    pub fn get_height(&self) -> u32 {
        self.height
    }
    pub fn set_height(&mut self, height: u32) {
        self.height = height;
    }

    pub fn get_fov(&self) -> f32 {
        self.fov
    }
    pub fn set_fov(&mut self, fov: f32) {
        self.fov = fov;
    }

    pub fn get_depth(&self) -> f32 {
        self.depth
    }
    pub fn set_depth(&mut self, depth: f32) {
        self.depth = depth;
    }

    pub fn get_position(&self) -> Vec3 {
        self.position
    }
    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
    }
    pub fn move_forward(&mut self, amount: f32) {
        self.position += amount * self.direction;
    }
    pub fn move_right(&mut self, amount: f32) {
        self.position += amount * self.right;
    }
    pub fn move_up(&mut self, amount: f32) {
        self.position += amount * self.up;
    }

    pub fn get_direction(&self) -> Vec3 {
        self.direction
    }
    pub fn set_direction(&mut self, direction: Vec3, up: Vec3) {
        self.direction = direction.normalize();
        self.up = up.normalize();
        self.right = -direction.cross(up).normalize();
    }
    pub fn get_up(&self) -> Vec3 {
        self.up
    }
    pub fn get_right(&self) -> Vec3 {
        self.right
    }
    pub fn rotate(&mut self, rotation_axis: Vec3, angle: f32) {
        let k_mat = Mat3 {
            x_axis: Vec3 {
                x: 0.0,
                y: -rotation_axis.z,
                z: rotation_axis.y,
            },
            y_axis: Vec3 {
                x: rotation_axis.z,
                y: 0.0,
                z: -rotation_axis.x,
            },
            z_axis: Vec3 {
                x: -rotation_axis.y,
                y: rotation_axis.x,
                z: 0.0,
            },
        };

        let i_mat = Mat3 {
            x_axis: Vec3 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            y_axis: Vec3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            z_axis: Vec3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
        };

        let rot_mat = i_mat + f32::sin(angle) * k_mat + (1.0 - f32::cos(angle)) * (k_mat * k_mat);

        self.direction = rot_mat * self.direction;
        self.up = rot_mat * self.up;
        self.right = rot_mat * self.right;
    }
    pub fn rotate_horizontal(&mut self, angle: f32) {
        self.rotate(self.up, angle);
    }
    pub fn rotate_vertical(&mut self, angle: f32) {
        self.rotate(self.right, angle);
    }
    pub fn rotate_center(&mut self, angle: f32) {
        self.rotate(self.direction, angle);
    }
}
