extern crate cgmath;
use self::cgmath::{Vector3, Point3};

use std::f32;

use ray::Ray;

#[derive(Debug)]
pub struct AABB {
    pub min: Point3<f32>,
    pub max: Point3<f32>,
}

impl AABB {
    pub fn new() -> AABB {
        AABB {
            min: Point3 { x: f32::INFINITY, y: f32::INFINITY, z: f32::INFINITY },
            max: Point3 { x: -f32::INFINITY, y: -f32::INFINITY, z: -f32::INFINITY },
        }
    }
    pub fn extent(&self, position: &Point3<f32>) -> AABB {
        AABB {min : Point3 {x : self.min.x.min(position.x),
                            y : self.min.y.min(position.y),
                            z : self.min.z.min(position.z) },
              max : Point3 {x : self.max.x.max(position.x),
                            y : self.max.y.max(position.y),
                            z : self.max.z.max(position.z) }
        }
    }
    pub fn combine(&self, aabb: &AABB) -> AABB {
        AABB {min : Point3 {x : self.min.x.min(aabb.min.x),
                            y : self.min.y.min(aabb.min.y),
                            z : self.min.z.min(aabb.min.z) },
              max : Point3 {x : self.max.x.max(aabb.max.x),
                            y : self.max.y.max(aabb.max.y),
                            z : self.max.z.max(aabb.max.z) }
        }
    }
    pub fn size(&self) -> Vector3<f32> {
        self.max - self.min
    }

    pub fn intersect(&self, ray : & mut Ray) -> bool{
        let rcp_dir : Vector3<f32> = 1.0 / ray.direction;
        let min = self.min - ray.origin;
        let max = self.max - ray.origin;

        let imin = Vector3{ x : min.x * rcp_dir.x, y : min.y * rcp_dir.y, z : min.z * rcp_dir.z };
        let imax = Vector3{ x : max.x * rcp_dir.x, y : max.y * rcp_dir.y, z : max.z * rcp_dir.z };

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
    fn centre(&self) -> Point3<f32> {
        self.min + 0.5 * (self.max - self.min)
    }
}

#[test]
fn intersections_aabb() {
    let aabb = AABB::new().extent(&Point3::new(1.0,1.0,1.0)).extent(&Point3::new(-1.0,-1.0,3.0));

    // Intersects forwards
    let mut r1 = Ray::new(Point3::new(0.0,0.0,0.0), Vector3::new(0.0,0.0,1.0), f32::INFINITY);
    assert!(aabb.intersect(&mut r1));

    // Doesn't intersect backwards.
    let mut r1 = Ray::new(Point3::new(0.0,0.0,0.0), Vector3::new(0.0,0.0,-1.0), f32::INFINITY);
    assert!(!aabb.intersect(&mut r1));

    // Barely intersects top.
    // Doesn't work with a y of 1.0
    let mut r1 = Ray::new(Point3::new(0.0,0.99,0.0), Vector3::new(0.0,0.0,1.0), f32::INFINITY);
    assert!(aabb.intersect(&mut r1));

    // Intersects on ray origin.
    // Doesn't work with a z of 1.0
    let mut r1 = Ray::new(Point3::new(0.0,0.0,1.01), Vector3::new(0.0,1.0,0.0), f32::INFINITY);
    assert!(aabb.intersect(&mut r1));

    // Intersects on ray origin.
    let mut r1 = Ray::new(Point3::new(0.0,0.0,1.0), Vector3::new(0.0,0.0,-1.0), f32::INFINITY);
    assert!(aabb.intersect(&mut r1));

    // Intersects inside.
    let mut r1 = Ray::new(Point3::new(0.0,0.0,2.0), Vector3::new(0.0,0.0,1.0), f32::INFINITY);
    assert!(aabb.intersect(&mut r1));

    // Intersects inside.
    let mut r1 = Ray::new(Point3::new(0.0,0.0,2.5), Vector3::new(0.0,0.0,1.0), f32::INFINITY);
    assert!(aabb.intersect(&mut r1));

    // Intersects inside.
    let mut r1 = Ray::new(Point3::new(0.0,0.0,2.5), Vector3::new(0.0,0.0,-1.0), f32::INFINITY);
    assert!(aabb.intersect(&mut r1));
}
