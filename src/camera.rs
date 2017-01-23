extern crate cgmath;
extern crate rand;
extern crate sdl2;
use self::cgmath::{Vector3, Point3};
use self::cgmath::InnerSpace;
use self::cgmath::ElementWise;
use super::ray::{Ray, Intersection};
use std::f32;
use std::collections::HashSet;
use self::sdl2::keyboard::Keycode;
use self::rand::Closed01;

use scene::Scene;
use material::Material;
use primitive::Primitive;

#[derive(Debug)]
pub struct Camera<T: Primitive> {
    origin: Point3<f32>,
    target: Point3<f32>,
    focal_distance: f32,
    direction: Vector3<f32>,

    // screen plane
    p1: Point3<f32>,
    p2: Point3<f32>,
    p3: Point3<f32>,

    up: Vector3<f32>,
    right: Vector3<f32>,
    width: usize,
    height: usize,
    lens_size: f32,
    scene: Scene<T>,
}

impl<T: Primitive> Camera<T> {
    pub fn new(width: usize, height: usize, scene: Scene<T>) -> Camera<T> {
        let mut camera = Camera {
            width: width,
            height: height,
            lens_size: 0.04,
            origin: Point3::new(-0.94, -0.037, -3.342),
            target: Point3::new(-0.418, -0.026, -2.435),
            direction: Vector3::new(0.0, 0.0, 0.0),
            focal_distance: 0.0,
            p1: Point3::new(0.0, 0.0, 0.0),
            p2: Point3::new(0.0, 0.0, 0.0),
            p3: Point3::new(0.0, 0.0, 0.0),
            right: Vector3::new(0.0, 0.0, 0.0),
            up: Vector3::new(0.0, 0.0, 0.0),
            scene: scene,
        };
        camera.update();
        camera
    }

    pub fn handle_input(&mut self, key_presses: &HashSet<Keycode>) -> bool {
        self.target = self.origin + self.direction;

        let changed = if key_presses.contains(&Keycode::A) {
            self.origin = self.origin + (-0.1 * self.right);
            self.target = self.target + (-0.1 * self.right);
            true
        } else {
            false
        };
        let changed = if key_presses.contains(&Keycode::D) {
            self.origin = self.origin + (0.1 * self.right);
            self.target = self.target + (0.1 * self.right);
            true
        } else {
            changed
        };
        let changed = if key_presses.contains(&Keycode::W) {
            self.origin = self.origin + (0.1 * self.direction);
            true
        } else {
            changed
        };
        let changed = if key_presses.contains(&Keycode::S) {
            self.origin = self.origin + (-0.1 * self.direction);
            true
        } else {
            changed
        };
        let changed = if key_presses.contains(&Keycode::R) {
            self.origin = self.origin + (0.1 * self.up);
            self.target = self.target + (0.1 * self.up);
            true
        } else {
            changed
        };
        let changed = if key_presses.contains(&Keycode::F) {
            self.origin = self.origin + (-0.1 * self.up);
            self.target = self.target + (-0.1 * self.up);
            true
        } else {
            changed
        };
        let changed = if key_presses.contains(&Keycode::Up) {
            self.target = self.target + (-0.1 * self.up);
            true
        } else {
            changed
        };
        let changed = if key_presses.contains(&Keycode::Down) {
            self.target = self.target + (0.1 * self.up);
            true
        } else {
            changed
        };
        let changed = if key_presses.contains(&Keycode::Left) {
            self.target = self.target + (-0.1 * self.right);
            true
        } else {
            changed
        };
        let changed = if key_presses.contains(&Keycode::Right) {
            self.target = self.target + (0.1 * self.right);
            true
        } else {
            changed
        };
        if changed {
            self.update();
            true
        } else {
            false
        }

    }

    fn update(&mut self) {
        self.direction = (self.target - self.origin).normalize();
        let unit_y = Vector3::new(0.0, 1.0, 0.0);
        self.right = unit_y.cross(self.direction);
        self.up = self.direction.cross(self.right);

        let mut ray = Ray::new(self.origin, self.direction, f32::INFINITY);
        let _intersection = self.scene.intersect(&mut ray);

        let aspect_ratio = (self.width as f32) / (self.height as f32);

        self.focal_distance = f32::min(20.0, ray.distance);

        let c = self.origin + self.focal_distance * self.direction;

        self.p1 = c + (-0.5 * self.focal_distance * aspect_ratio * self.right) +
                  (0.5 * self.focal_distance * self.up);
        self.p2 = c + (0.5 * self.focal_distance * aspect_ratio * self.right) +
                  (0.5 * self.focal_distance * self.up);
        self.p3 = c + (-0.5 * self.focal_distance * aspect_ratio * self.right) +
                  (-0.5 * self.focal_distance * self.up);

    }

    /// sample a ray by shooting it through the scene
    pub fn sample(&self, ray: &mut Ray, depth: u32) -> Vector3<f32> {
        let mut sample = Vector3::new(1., 1., 1.);
        let mut intersection = self.scene.intersect(ray);
        let mut current_refraction_index = 1.; //Air
        for _ in 0..depth {
            match intersection {
                None => {
                    sample = sample.mul_element_wise(self.scene.sample_skybox(ray.direction));
                    break;
                }
                Some(Intersection{normal, inside, material}) => {
                    let intersection_point = ray.intersection();
                    match material {
                        &Material::Diffuse { speculaty, color} => {
                            if speculaty > 0. {
                                let Closed01(r0) = rand::random::<Closed01<f32>>();
                                if r0 < speculaty {
                                    // Specular sampling
                                    sample = sample.mul_element_wise(color);
                                    let reflected_dir = ray.direction - 2. * ray.direction.dot(normal) * normal;
                                    ray.reset(intersection_point, reflected_dir, f32::INFINITY);
                                    intersection = self.scene.intersect(ray);
                                    continue
                                }
                            }
                            // Diffuse sampling
                            let diffuse_dir = {
                                let Closed01(r0) = rand::random::<Closed01<f32>>();
                                let r = (1. - r0 * r0).sqrt();
                                let Closed01(r1) = rand::random::<Closed01<f32>>();
                                let phi = 2. * f32::consts::PI * r1;
                                let diffuse_dir = Vector3::new(phi.cos() * r, phi.sin() * r, r0);
                                if diffuse_dir.dot(normal) < 0. {
                                    -1. * diffuse_dir
                                } else {
                                    diffuse_dir
                                }
                            };
                            sample = sample.mul_element_wise(diffuse_dir.dot(normal) * color);
                            ray.reset(intersection_point, diffuse_dir, f32::INFINITY);
                            intersection = self.scene.intersect(ray);
                            continue
                        }
                        &Material::Dielectic { refraction_index, color } => {
                            let n1 = current_refraction_index;
                            let mut normal = normal;
                            let n2 = if n1 != 1. { normal = -normal; 1. } else { refraction_index };
                            let refracted_dir = {
                                // Refract
                                let div = n1 / n2;
                                let cosi = ray.direction.dot(normal);
                                let sin_t2 = div * div * (1. - cosi * cosi);
                                if sin_t2 <= 1. {
                                    Some(div * ray.direction - (div * cosi + (1. - sin_t2).sqrt()) * normal)
                                } else {
                                    None
                                }
                            };
                            if let Some(refracted_dir) = refracted_dir {
                                let reflection = {
                                    // Schlick
                                    let div = (n1 - n2) / (n1 + n2);
                                    let r0 = div * div;
                                    let cosi = -ray.direction.dot(normal);
                                    r0 + (1. - r0) * (1. - cosi).powi(5)
                                };
                                let refraction = 1. - reflection;
                                let Closed01(r0) = rand::random::<Closed01<f32>>();
                                if r0 < refraction {
                                    // Refraction sampling
                                    current_refraction_index = n2;
                                    ray.reset(intersection_point, refracted_dir, f32::INFINITY);
                                    intersection = self.scene.intersect(ray);
                                    if n2 != 1. {
                                        let absorbance = (Vector3::new(-1.,-1.,-1.) + color) * ray.distance;
                                        let transparency = Vector3::new(absorbance.x.exp(), absorbance.y.exp(), absorbance.z.exp());
                                        sample = sample.mul_element_wise(transparency);
                                    }
                                    continue
                                } else {
                                    // Reflected ray
                                    let reflected_dir = ray.direction - 2. * ray.direction.dot(normal) * normal;
                                    ray.reset(intersection_point, reflected_dir, f32::INFINITY);
                                    intersection = self.scene.intersect(ray);
                                    if n2 != 1. {
                                        sample = sample.mul_element_wise(color);
                                    }
                                    //else {
                                    //    let absorbance = (Vector3::new(-1.,-1.,-1.) + color) * ray.distance;
                                    //    let transparency = Vector3::new(absorbance.x.exp(), absorbance.y.exp(), absorbance.z.exp());
                                    //    sample = sample.mul_element_wise(transparency);
                                    //}
                                    continue
                                }
                            } else {
                                // Full internal reflection
                                let reflected_dir = ray.direction - 2. * ray.direction.dot(normal) * normal;
                                ray.reset(intersection_point, reflected_dir, f32::INFINITY);
                                intersection = self.scene.intersect(ray);
                                //let absorbance = (Vector3::new(-1.,-1.,-1.) + color) * ray.distance;
                                //let transparency = Vector3::new(absorbance.x.exp(), absorbance.y.exp(), absorbance.z.exp());
                                //sample = sample.mul_element_wise(transparency);
                                continue
                            }
                        }
                        &Material::Emissive { color } => {
                            sample = sample.mul_element_wise(color);
                            break;
                        }
                    }
                }
            };
        }
        sample
    }

    /// generates a nice Ray (TODO better integer type)
    pub fn generate(&self, x: usize, y: usize) -> Ray {
        // NOTE: we do not have to keep track of a
        // pool of random number generators, each
        // thread in rust has its own random
        // number generator by default :)
        let Closed01(r0) = rand::random::<Closed01<f32>>();
        let Closed01(r1) = rand::random::<Closed01<f32>>();
        let Closed01(r2) = rand::random::<Closed01<f32>>();
        let r2 = r2 - 0.5;
        let Closed01(r3) = rand::random::<Closed01<f32>>();
        let r3 = r3 - 0.5;

        // calculate sub-pixel ray target position on screen plane
        let u = ((x as f32) + r0) / (self.width as f32);
        let v = ((y as f32) + r1) / (self.height as f32);
        let target = self.p1 + u * (self.p2 - self.p1) + v * (self.p3 - self.p1);
        let origin = self.origin + self.lens_size * (r2 * self.right + r3 * self.up);
        let direction = (target - origin).normalize();

        // hmm all directions are the same. that seems to be a bug =)

        Ray::new(origin, direction, f32::INFINITY)

    }
}
