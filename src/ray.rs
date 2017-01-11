extern crate cgmath;
use self::cgmath::{Vector3, Point3};
use std::sync::Arc;

use material::Material;

#[derive(Debug)]
pub struct Intersection {
    pub normal: Vector3<f32>,
    pub inside: bool,
    pub material: Arc<Material>,
}

#[derive(Debug)]
pub struct Ray {
    pub origin: Point3<f32>,
    pub direction: Vector3<f32>,
    pub distance: f32,
    pub intersection: Option<Intersection>, // the closest intersection
}

impl Ray {
    pub fn new(origin: Point3<f32>, direction: Vector3<f32>, distance: f32) -> Ray {
        Ray {
            origin: origin,
            direction: direction,
            distance: distance,
            intersection: None,
        }
    }
}
