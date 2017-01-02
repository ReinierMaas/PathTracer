extern crate cgmath;
use self::cgmath::{Point3, InnerSpace};

use std::f32;

use super::Primitive;
use super::aabb::AABB;

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
    fn intersect(&self, ray : & mut Ray) -> bool {
        let distance = self.position - ray.origin;
        let tca = distance.dot(ray.direction);

        // inside hits are accounted for
        //if tca  < 0.0 {
        //    return false
        //}

        let d2 = distance.dot(distance) - tca*tca;

        if d2 > self.radius * self.radius {
            return false;
        }

        let thc = (self.radius * self.radius - d2).sqrt();
        let t0 = tca - thc;
        let t1 = tca + thc;

        if t0 >= 0.0 {
            if t0 >= ray.distance {
                return false;
            }
            ray.normal = (ray.origin + ray.direction * t0 - self.position).normalize();
            return true;
        } else if t1 >= 0.0 {
            if t1 >= ray.distance {
                return false;
            }
            ray.normal = (ray.origin + ray.direction * t1 - self.position).normalize();
            return true;
        }
        false
    }
    fn centre(&self) -> Point3<f32> {
        self.position
    }
    fn bounds(&self) -> AABB {
        AABB {min : Point3 {x : self.position.x - self.radius,
                            y : self.position.y - self.radius,
                            z : self.position.z - self.radius },
              max : Point3 {x : self.position.x + self.radius,
                            y : self.position.y + self.radius,
                            z : self.position.z + self.radius }}
    }
}

#[test]
fn intersections_sphere() {
    let sphere = Sphere::light(Point3::new(0.0, 0.0, 2.0), 1.0);

    // Intersects forwards
    let mut r1 = Ray::new(Point3::new(0.0,0.0,0.0), Vector3::new(0.0,0.0,1.0), f32::INFINITY);
    assert!(sphere.intersect(&mut r1));

    // Doesn't intersect backwards.
    let mut r1 = Ray::new(Point3::new(0.0,0.0,0.0), Vector3::new(0.0,0.0,-1.0), f32::INFINITY);
    assert!(!sphere.intersect(&mut r1));

    // Barely intersects top.
    let mut r1 = Ray::new(Point3::new(0.0,1.0,0.0), Vector3::new(0.0,0.0,1.0), f32::INFINITY);
    assert!(sphere.intersect(&mut r1));

    // Intersects on ray origin.
    let mut r1 = Ray::new(Point3::new(0.0,0.0,1.0), Vector3::new(0.0,1.0,0.0), f32::INFINITY);
    assert!(sphere.intersect(&mut r1));

    // Intersects on ray origin.
    let mut r1 = Ray::new(Point3::new(0.0,0.0,1.0), Vector3::new(0.0,0.0,-1.0), f32::INFINITY);
    assert!(sphere.intersect(&mut r1));

    // Intersects inside.
    let mut r1 = Ray::new(Point3::new(0.0,0.0,2.0), Vector3::new(0.0,0.0,1.0), f32::INFINITY);
    assert!(sphere.intersect(&mut r1));

    // Intersects inside.
    let mut r1 = Ray::new(Point3::new(0.0,0.0,2.5), Vector3::new(0.0,0.0,1.0), f32::INFINITY);
    assert!(sphere.intersect(&mut r1));

    // Intersects inside.
    let mut r1 = Ray::new(Point3::new(0.0,0.0,2.5), Vector3::new(0.0,0.0,-1.0), f32::INFINITY);
    assert!(sphere.intersect(&mut r1));
}