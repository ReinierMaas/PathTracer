extern crate cgmath;
use self::cgmath::{Vector3, Point3};
use std::f32;
use material::Material;

#[derive(Debug)]
pub struct Intersection<'a> {
    pub normal: Vector3<f32>,
    pub inside: bool,
    pub material: &'a Material,
}

#[derive(Debug)]
pub struct Ray {
    pub origin: Point3<f32>,
    pub direction: Vector3<f32>,
    pub distance: f32,
}

impl Ray {
    pub fn new(origin: Point3<f32>, direction: Vector3<f32>, distance: f32) -> Ray {
        Ray {
            origin: origin,
            direction: direction,
            distance: distance,
        }
    }
    pub fn reset(&mut self, origin: Point3<f32>, direction: Vector3<f32>, distance: f32) {
        self.origin = origin + 20. * f32::EPSILON * direction; // advance ray
        self.direction = direction;
        self.distance = distance; // set length
    }
    pub fn intersection(&self) -> Point3<f32> {
        self.origin + self.distance * self.direction
    }
}
