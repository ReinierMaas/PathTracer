extern crate cgmath;

use self::cgmath::{Vector3, Point3};
use material::Material;

#[derive(Debug)]
pub struct Ray {
    pub origin: Point3<f32>,
    pub direction: Vector3<f32>,
    pub normal: Vector3<f32>,
    pub distance: f32,
    pub inside: bool,
    pub material: Option<Material>, // the last material that it hit
}

impl Ray {
    pub fn new(origin: Point3<f32>, direction: Vector3<f32>, distance: f32) -> Ray {
        Ray {
            origin: origin,
            direction: direction,
            normal: Vector3::new(0.0,0.0,0.0),
            distance: distance,
            inside: false,
            material: None,
        }
    }
}
