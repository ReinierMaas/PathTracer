extern crate cgmath;
use self::cgmath::Point3;

pub mod aabb;
pub mod sphere;
pub mod triangle;

use self::aabb::AABB;

use ray::Ray;

pub trait Primitive {
    fn intersect(&self, ray : & mut Ray) -> bool;
    fn centre(&self) -> Point3<f32>;
    fn bounds(&self) -> AABB;
}