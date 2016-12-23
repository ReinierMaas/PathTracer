extern crate cgmath;
use self::cgmath::{Vector3, Point3, InnerSpace};

use super::Primitive;

use ray::Ray;
use material::{Material, LIGHT_COLOR};

#[derive(Debug)]
pub struct Sphere {
    pub position: Point3<f32>,
    pub radius: f32,
    pub material: Material,
}

impl Sphere {
    pub fn light(position: Point3<f32>, radius: f32) -> Sphere {
        Sphere {
            position: position,
            radius: radius,
            material: Material::Realistic {
                refl: 0.0,
                refr: 0.0,
                emissive: true,
                diffuse: LIGHT_COLOR,
            }
        }
    }
}

impl Primitive for Sphere {
    fn intersect(&self, ray : & mut Ray) {
        let distance = self.position - ray.origin;
        let tca = distance.dot(ray.direction);

        if tca  < 0.0 {
            return
        }

        let d2 = distance.dot(distance) - tca*tca;

        if d2 > self.radius {
            return
        }

        let thc = (self.radius - d2).sqrt();
        let t0 = tca - thc;
        let t1 = tca + thc;

        if t0 > 0.0 {
            if t0 > ray.distance {
                return
            }
            ray.normal = (ray.origin + ray.direction - self.position).normalize();
        }
    }
}
