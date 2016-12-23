extern crate cgmath;
use self::cgmath::{Vector3, Point3, InnerSpace};
use std::f32;

use super::Primitive;
use super::aabb::AABB;

use ray::Ray;
use material::{Material, LIGHT_COLOR};

#[derive(Debug)]
pub struct Triangle {
    pub position0: Point3<f32>,
    pub position1: Point3<f32>,
    pub position2: Point3<f32>,
    pub normal0: Vector3<f32>,
    pub normal1: Vector3<f32>,
    pub normal2: Vector3<f32>,
    pub material: Material,
}

impl Primitive for Triangle {
    fn intersect(&self, ray : & mut Ray) -> bool{
        let edge1 = self.position1 - self.position0;
        let edge2 = self.position2 - self.position0;
        let h = ray.direction.cross(edge2);
        let a = edge1.dot(h);

        if a > -f32::EPSILON && a < f32::EPSILON {
            return false;
        }

        let f = 1.0 / a;
        let s = ray.origin - self.position0;
        let u = f * s.dot(h);

        if u < 0.0 || u > 1.0 {
            return false;
        }

        let q = s.cross(edge1);
        let v = f * ray.direction.dot(q);

        if v < 0.0 || u + v > 1.0 {
            return false;
        }

        // at this stage we can compute t to find out where
        // the intersection point is on the ray
        let t = f * edge2.dot(q);
        return true;
    }
    fn bounds(&self) -> AABB {
        AABB {min : Point3 {x : self.position0.x.min(self.position1.x).min(self.position2.x),
                            y : self.position0.y.min(self.position1.y).min(self.position2.y),
                            z : self.position0.z.min(self.position1.z).min(self.position2.z) },
              max : Point3 {x : self.position0.x.min(self.position1.x).min(self.position2.x),
                            y : self.position0.y.min(self.position1.y).min(self.position2.y),
                            z : self.position0.z.min(self.position1.z).min(self.position2.z) }}
    }
}
