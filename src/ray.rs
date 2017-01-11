extern crate cgmath;

use self::cgmath::{Vector3, Point3};
use material::Material;

#[derive(Debug)]
pub struct Intersection<'m> {
    pub normal: Vector3<f32>,
    pub inside: bool,
    pub material: &'m Material,
}

#[derive(Debug)]
pub struct Ray<'m> {
    pub origin: Point3<f32>,
    pub direction: Vector3<f32>,
    pub distance: f32,
    pub intersection: Option<Intersection<'m>>, // the closest intersection
}

impl<'m> Ray<'m> {
    pub fn new(origin: Point3<f32>, direction: Vector3<f32>, distance: f32) -> Ray<'static> {
        Ray {
            origin: origin,
            direction: direction,
            distance: distance,
            intersection: None,
        }
    }
}
