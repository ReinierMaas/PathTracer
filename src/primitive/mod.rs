pub mod aabb;
pub mod sphere;
pub mod triangle;

use self::aabb::AABB;

use ray::Ray;

pub trait Primitive {
    fn intersect(&self, ray : & mut Ray) -> bool;
    fn bounds(&self) -> AABB;
}