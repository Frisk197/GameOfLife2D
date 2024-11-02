use bevy::math::Vec3;

#[derive(Hash, Eq, PartialEq, Copy, Clone)]
pub struct uVec3{
    pub x: u32,
    pub y: u32,
    pub z: u32,
}

impl uVec3{
    pub fn new(x: u32, y: u32, z: u32) -> uVec3{
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