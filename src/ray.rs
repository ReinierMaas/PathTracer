extern crate cgmath;

use self::cgmath::{Vector3, Point3};
use super::scene::Material;

pub struct Ray {
    origin: Point3<f32>,
    direction: Vector3<f32>,
    N: Vector3<f32>,
    distance: f32,
    inside: bool,
    material: Option<Material>, // the last material that it hit
}

impl Ray {
    pub fn new(origin: Point3<f32>, direction: Vector3<f32>, distance: f32) -> Ray {
        Ray {
            origin: origin,
            direction: direction,
            N: Vector3::new(0.0,0.0,0.0),
            distance: distance,
            inside: false,
            material: None,
        }
    }
}
