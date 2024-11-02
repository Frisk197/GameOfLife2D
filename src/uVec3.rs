use bevy::math::Vec3;

#[derive(Hash, Eq, PartialEq, Copy, Clone)]
pub struct uVec3{
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl uVec3{
    pub fn new(x: i32, y: i32, z: i32) -> uVec3{
        let mut a: uVec3 = uVec3 {
            x,
            y,
            z,
        };
        a
    }

    pub fn toVec3(self) -> Vec3{
        let mut a: Vec3 = Vec3::new(self.x as f32, self.y as f32, self.z as f32);
        a
    }
}