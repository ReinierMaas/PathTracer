extern crate cgmath;
use self::cgmath::{Vector3, Point3};
use std::io;
use std::mem;
use ray::{Ray,Intersection};
use bvh::BVH;
use std::f32::consts::FRAC_1_PI;
extern crate memmap;
use self::memmap::*;

use primitive::Primitive;
use primitive::sphere::Sphere;
use material::Material;

#[derive(Debug)]
pub struct Scene {
    bvh: BVH,
    spheres: Vec<Sphere>,
    skybox: Vec<f32>,
}

impl Scene {
    // creates a new default scene
    fn new() -> Result<Scene, io::Error> {
        let spheres = Vec::new();
        let skybox = try!(Scene::read_skybox());
        let scene = Scene {
            bvh: BVH::new(Vec::new(), Vec::new()),
            spheres: spheres,
            skybox: skybox,
        };
        Ok(scene)
    }

    pub fn intersect(&self, ray : & mut Ray) -> Option<Intersection> {
        let mut intersection = None;
        for sphere in &self.spheres {
            match sphere.intersect(ray) {
                None => (),
                Some(x) => intersection = Some(x),
            }
        }
        intersection
    }

    pub fn default_scene() -> Result<Scene, io::Error> {
        print!("Setting up default_scene\n");
        let mut scene = try!(Scene::new());

        scene.add(Sphere::light(Point3::new(2.7,1.7,-0.5), 0.3));

        let bottom_plane = Sphere {
            position: Point3::new(0.0,-4999.0,0.0),
            radius: 4998.5,
            material: Material::Diffuse {
                speculaty: 0.,
                color: Vector3::new(0.0,1.0,1.0),
            },
        };

        let back_plane = Sphere {
            position: Point3::new(0.0,0.0,-5000.0),
            radius: 4998.5,
            material: Material::Diffuse {
                speculaty: 0.,
                color: Vector3::new(1.0,1.0,1.0),
            },
        };

        scene.add(bottom_plane);
        //scene.add(back_plane);
        scene.add(Sphere {
            position: Point3::new(-0.8, 0.0, -2.0),
            radius: 0.3,
            material: Material::Diffuse {
                speculaty: 0.8,
                color: Vector3::new(1.0,0.2,0.2),
            },
        });

        scene.add(Sphere {
            position: Point3::new(0.0,0.0,-2.0),
            radius: 0.3,
            material: Material::Dielectic {
                refraction_index: 1.3,
                color: Vector3::new(0.9,1.0,0.9),
            },
        });

        scene.add(Sphere {
            position: Point3::new(0.8,0.0,-2.0),
            radius: 0.3,
            material: Material::Diffuse {
                speculaty: 0.8,
                color: Vector3::new(0.2, 0.2, 1.0),
            },
        });

        scene.add(Sphere {
            position: Point3::new(-0.8,-0.8,-2.0),
            radius: 0.5,
            material: Material::Diffuse {
                speculaty: 0.,
                color: Vector3::new(1.0,1.0,1.0),
            },
        });
        scene.add(Sphere {
            position: Point3::new(-0.0,-0.8,-2.0),
            radius: 0.5,
            material: Material::Diffuse {
                speculaty: 0.,
                color: Vector3::new(1.0,1.0,1.0),
            },
        });
        scene.add(Sphere {
            position: Point3::new(0.8,-0.8,-2.0),
            radius: 0.5,
            material: Material::Diffuse {
                speculaty: 0.,
                color: Vector3::new(1.0,1.0,1.0),
            },
        });


        Ok(scene)
    }

    fn add(&mut self, sphere: Sphere) {
        self.spheres.push(sphere)
    }

    pub fn sample_skybox(&self, direction: Vector3<f32>) -> Vector3<f32> {
        let u = (2500.0 * 0.5 * (1.0 + direction.x.atan2(-direction.z) * FRAC_1_PI)) as usize;
        let v = (1250.0 * (direction.y.acos() * FRAC_1_PI)) as usize;
        let idx = u + v * 2500;
        Vector3::new(self.skybox[idx*3+0],
                     self.skybox[idx*3+1],
                     self.skybox[idx*3+2])
    }

    fn read_skybox() -> Result<Vec<f32>, io::Error> {
        let file = try!(Mmap::open_path("./assets/sky_15.raw", Protection::Read));
        let bytes: &[u8] = unsafe { file.as_slice() };
        let mut floats = vec![0.0 as f32; bytes.len() / 2];
        println!("{}",floats.len());
        for (chunk, mut float) in bytes.chunks(4).into_iter().zip(floats.iter_mut()) {
            // we assume big endian here!
            // but intel is little endian
            *float = unsafe { mem::transmute([chunk[0],chunk[1],chunk[2],chunk[3]]) };
        }
        Ok(floats)
    }


}
