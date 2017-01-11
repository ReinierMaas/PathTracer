extern crate cgmath;
use self::cgmath::{Vector3, Point3, InnerSpace, EuclideanSpace};
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

impl Triangle {
    pub fn light(p0 : Point3<f32>, p1 : Point3<f32>, p2 : Point3<f32>, n0 : Vector3<f32>, n1 : Vector3<f32>, n2 : Vector3<f32>) -> Triangle {
        Triangle {
            position0: p0,
            position1: p1,
            position2: p2,
            normal0: n0,
            normal1: n1,
            normal2: n2,
            material: Material::Realistic {
                refl: 0.0,
                refr: 0.0,
                emissive: true,
                diffuse: LIGHT_COLOR,
            }
        }
    }
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
        if t < 0.0 {
            return false; // the intersection is behind the ray's origin
        }
        return true;
    }
    fn centre(&self) -> Point3<f32> {
        (self.position0 + self.position1.to_vec() + self.position2.to_vec()) * (1.0 / 3.0)
    }
    fn bounds(&self) -> AABB {
        AABB {min : Point3 {x : self.position0.x.min(self.position1.x).min(self.position2.x),
                            y : self.position0.y.min(self.position1.y).min(self.position2.y),
                            z : self.position0.z.min(self.position1.z).min(self.position2.z) },
              max : Point3 {x : self.position0.x.max(self.position1.x).max(self.position2.x),
                            y : self.position0.y.max(self.position1.y).max(self.position2.y),
                            z : self.position0.z.max(self.position1.z).max(self.position2.z) }}
    }
    fn is_light(&self) -> bool {
        match self.material {
            Material::Realistic{ refl: refl, refr: refr, emissive: emissive, diffuse: diffuse } => emissive,
            _ => false,
        }
    }
}

#[test]
fn intersections_triangle() {
    let triangle = Triangle::light(Point3::new(1.0, 1.0, 2.0),Point3::new(1.0, -1.0, 2.0),Point3::new(-1.0, 0.0, 2.0),Vector3::new(0.0, 0.0, -1.0),Vector3::new(0.0, 0.0, -1.0),Vector3::new(0.0, 0.0, -1.0));

    // Intersects forwards
    let mut r1 = Ray::new(Point3::new(0.0,0.0,0.0), Vector3::new(0.0,0.0,1.0), f32::INFINITY);
    assert!(triangle.intersect(&mut r1));

    // Doesn't intersect backwards.
    let mut r1 = Ray::new(Point3::new(0.0,0.0,0.0), Vector3::new(0.0,0.0,-1.0), f32::INFINITY);
    assert!(!triangle.intersect(&mut r1));

    // Barely intersects top.
    let mut r1 = Ray::new(Point3::new(1.0,1.0,0.0), Vector3::new(0.0,0.0,1.0), f32::INFINITY);
    assert!(triangle.intersect(&mut r1));

    // Doesn't intersect on ray origin, is parrallel to triangle.
    let mut r1 = Ray::new(Point3::new(0.0,0.0,2.0), Vector3::new(0.0,1.0,0.0), f32::INFINITY);
    assert!(!triangle.intersect(&mut r1));

    // Intersects on ray origin.
    let mut r1 = Ray::new(Point3::new(0.0,0.0,2.0), Vector3::new(0.0,0.0,-1.0), f32::INFINITY);
    assert!(triangle.intersect(&mut r1));

    // Intersects on ray origin.
    let mut r1 = Ray::new(Point3::new(0.0,0.0,2.0), Vector3::new(0.0,0.0,1.0), f32::INFINITY);
    assert!(triangle.intersect(&mut r1));

    // Doesn't intersect ray in front of triangle.
    let mut r1 = Ray::new(Point3::new(0.0,0.0,2.5), Vector3::new(0.0,0.0,1.0), f32::INFINITY);
    assert!(!triangle.intersect(&mut r1));

    // Intersects triangle from other side.
    let mut r1 = Ray::new(Point3::new(0.0,0.0,2.5), Vector3::new(0.0,0.0,-1.0), f32::INFINITY);
    assert!(triangle.intersect(&mut r1));
}