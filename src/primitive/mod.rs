extern crate cgmath;
use self::cgmath::{Point3,Vector3};

pub mod aabb;
pub mod sphere;
pub mod triangle;

use self::aabb::AABB;

use ray::{Ray,Intersection};

pub trait Primitive {
    fn intersect(&self, ray : & mut Ray) -> Option<Intersection>;
    fn centre(&self) -> Point3<f32>;
    fn bounds(&self) -> AABB;
    fn is_light(&self) -> Option<Vector3<f32>>;
    fn random_point(&self) -> Point3<f32>;
    fn area(&self) -> f32;
}
