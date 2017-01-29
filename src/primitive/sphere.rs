extern crate cgmath;
use self::cgmath::{Point3, InnerSpace, Vector3};

use std::f32;
use rand;

use super::Primitive;
use super::aabb::AABB;

use ray::{Ray, Intersection};
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
            material: Material::Emissive {
                color: LIGHT_COLOR,
            }
        }
    }
}

impl Primitive for Sphere {

    fn intersect(&self, ray: & mut Ray) -> Option<Intersection> {
        let distance = self.position - ray.origin;
        let tca = distance.dot(ray.direction);

        // inside hits are accounted for
        //if tca  < 0.0 {
        //    return false
        //}

        let d2 = distance.dot(distance) - tca*tca;

        if d2 > self.radius * self.radius {
            return None
        }

        let thc = (self.radius * self.radius - d2).sqrt();
        let t0 = tca - thc;
        let t1 = tca + thc;

        if t0 >= 0.0 {
            if t0 >= ray.distance {
                None
            } else {
                ray.distance = t0;
                Some(Intersection{
                    normal: (ray.origin + ray.direction * t0 - self.position).normalize(),
                    inside:  false,
                    material: &self.material,
                })
            }
        } else if t1 >= 0.0 {
            if t1 >= ray.distance {
                None
            } else {
                ray.distance = t1;
                Some(Intersection{
                    normal: (ray.origin + ray.direction * t1 - self.position).normalize(),
                    inside: true,
                    material: &self.material,
                })
            }
        } else {
            None
        }
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
    fn is_light(&self) -> Option<Vector3<f32>> {
        match self.material {
            Material::Emissive { color } => Some(color),
            _ => None,
        }
    }
    fn random_point(&self) -> Point3<f32> {
        use std::f32;
        use rand::distributions::*;
        let mut rng = rand::thread_rng();

        let phi_range: Range<f32> = Range::new(0.0, 2.0*f32::consts::PI);
        let cos_theta_range: Range<f32> = Range::new(-1.0, 1.0);
        let u_range: Range<f32> = Range::new(0.0,1.0);

        let phi =  phi_range.ind_sample(&mut rng);
        let cos_theta = cos_theta_range.ind_sample(&mut rng);
        let u = u_range.ind_sample(&mut rng);

        let theta = cos_theta.acos();

        let r = self.radius * u.cbrt();
        let x = r * theta.sin() * phi.cos();
        let y = r * theta.sin() * phi.cos();
        let z = r * theta.cos();
        Point3::new(x + self.position.x, y + self.position.y, z + self.position.z)
    }
    fn area(&self) -> f32 {
        f32::consts::PI * self.radius * self.radius
    }
}

#[test]
fn intersections_sphere() {
    use self::cgmath::Vector3;
    let sphere = Sphere::light(Point3::new(0.0, 0.0, 2.0), 1.0);

    // Intersects forwards
    let mut r1 = Ray::new(Point3::new(0.0,0.0,0.0), Vector3::new(0.0,0.0,1.0), f32::INFINITY);
    assert!(sphere.intersect(&mut r1).is_some());

    // Doesn't intersect backwards.
    let mut r1 = Ray::new(Point3::new(0.0,0.0,0.0), Vector3::new(0.0,0.0,-1.0), f32::INFINITY);
    assert!(!sphere.intersect(&mut r1).is_some());

    // Barely intersects top.
    let mut r1 = Ray::new(Point3::new(0.0,1.0,0.0), Vector3::new(0.0,0.0,1.0), f32::INFINITY);
    assert!(sphere.intersect(&mut r1).is_some());

    // Intersects on ray origin.
    let mut r1 = Ray::new(Point3::new(0.0,0.0,1.0), Vector3::new(0.0,1.0,0.0), f32::INFINITY);
    assert!(sphere.intersect(&mut r1).is_some());

    // Intersects on ray origin.
    let mut r1 = Ray::new(Point3::new(0.0,0.0,1.0), Vector3::new(0.0,0.0,-1.0), f32::INFINITY);
    assert!(sphere.intersect(&mut r1).is_some());

    // Intersects inside.
    let mut r1 = Ray::new(Point3::new(0.0,0.0,2.0), Vector3::new(0.0,0.0,1.0), f32::INFINITY);
    assert!(sphere.intersect(&mut r1).is_some());

    // Intersects inside.
    let mut r1 = Ray::new(Point3::new(0.0,0.0,2.5), Vector3::new(0.0,0.0,1.0), f32::INFINITY);
    assert!(sphere.intersect(&mut r1).is_some());

    // Intersects inside.
    let mut r1 = Ray::new(Point3::new(0.0,0.0,2.5), Vector3::new(0.0,0.0,-1.0), f32::INFINITY);
    assert!(sphere.intersect(&mut r1).is_some());
}
