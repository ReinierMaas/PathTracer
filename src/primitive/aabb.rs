extern crate cgmath;
use self::cgmath::{Vector3, Point3, InnerSpace};

use super::Primitive;

use ray::Ray;

#[derive(Debug)]
pub struct AABB {
    pub min: Point3<f32>,
    pub max: Point3<f32>,
}

impl Primitive for AABB {
    fn intersect(&self, ray : & mut Ray) -> bool{
        let rcpDir : Vector3<f32> = 1.0 / ray.direction;
        let min = self.min - ray.origin;
        let max = self.max - ray.origin;

        let imin = Vector3{ x : min.x * rcpDir.x, y : min.y * rcpDir.y, z : min.z * rcpDir.z };
        let imax = Vector3{ x : max.x * rcpDir.x, y : max.y * rcpDir.y, z : max.z * rcpDir.z };

        let tmin =      imin.x.min(imax.x)
                   .max(imin.y.min(imax.y))
                   .max(imin.z.min(imax.z));
        let tmax =      imin.x.max(imax.x)
                   .min(imin.y.max(imax.y))
                   .min(imin.z.max(imax.z));

        if tmax < 0.0 {
            // ray intersects aabb but behind us
            return false;
        }
        if tmax < tmin {
            // ray doesn't intersect aabb
            return false;
        }
        if tmin < 0.0 {
            // ray originates inside aabb
            // hit is at the inside at tmax
            return true;
        }
        // ray originates outside aabb
        // hit is at the outside at tmin
        return true;
    }
    fn bounds(&self) -> AABB {
        AABB { min : self.min, max : self.max }
    }
}