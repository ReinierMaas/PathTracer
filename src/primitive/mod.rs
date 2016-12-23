pub mod sphere;

use ray::Ray;

pub trait Primitive {
    fn intersect(&self, ray : & mut Ray);
}