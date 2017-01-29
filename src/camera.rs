extern crate cgmath;
extern crate rand;
extern crate sdl2;
use self::cgmath::{Vector3, Point3, Array};
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

    depth: u32,
}

fn refract(direction: &Vector3<f32>, normal: &Vector3<f32>, n1: f32, n2: f32) -> Option<Vector3<f32>> {
    // Refract
    let div = n1 / n2;
    let cosi = direction.dot(*normal);
    let sin_t2 = div * div * (1. - cosi * cosi);
    if sin_t2 <= 1. {
        Some(div * direction - (div * cosi + (1. - sin_t2).sqrt()) * normal)
    } else {
        None
    }
}
fn reflect(direction: &Vector3<f32>, normal: &Vector3<f32>) -> Vector3<f32> {
    // Reflect
    direction - 2. * direction.dot(*normal) * normal
}
fn schlick(direction: &Vector3<f32>, normal: &Vector3<f32>, n1: f32, n2: f32) -> f32 {
    // Schlick
    let div = (n1 - n2) / (n1 + n2);
    let r0 = div * div;
    let cosi = -direction.dot(*normal); // ray direction is towards normal invert answer
    r0 + (1. - r0) * (1. - cosi).powi(5)
}
fn diffuse(normal: &Vector3<f32>) -> Vector3<f32> {
    // Diffuse
    let Closed01(r0) = rand::random::<Closed01<f32>>();
    let r = (1. - r0 * r0).sqrt();
    let Closed01(r1) = rand::random::<Closed01<f32>>();
    let phi = 2. * f32::consts::PI * r1;
    let diffuse_dir = Vector3::new(phi.cos() * r, phi.sin() * r, r0);
    if diffuse_dir.dot(*normal) < 0. {
        -1. * diffuse_dir
    } else {
        diffuse_dir
    }
}
fn cosine_weighted_diffuse(normal: &Vector3<f32>) -> Vector3<f32> {
    // Cosine weighted Diffuse
    let Closed01(r0) = rand::random::<Closed01<f32>>();
    let r = r0.sqrt();
    let Closed01(r1) = rand::random::<Closed01<f32>>();
    let phi = 2. * f32::consts::PI * r1;
    from_tangent_to_local(normal, &Vector3::new(phi.cos() * r, phi.sin() * r, (1. - r0).sqrt()))
}
fn from_tangent_to_local(normal: &Vector3<f32>, tangent: &Vector3<f32>) -> Vector3<f32> {
    let t = (normal.cross(if normal.x.abs() > 0.99 { Vector3::new(0.0,1.0,0.0) } else { Vector3::new(1.0,0.0,0.0) })).normalize();
    let b = normal.cross(t);
    tangent.x * t + tangent.y * b + tangent.z * normal
}

impl<T: Primitive> Camera<T> {
    pub fn new(width: usize, height: usize, scene: Scene<T>) -> Camera<T> {
        let mut camera = Camera {
            width: width,
            height: height,
            depth: 512,
            lens_size: 0.04,
            origin: Point3::new(-1.6, 0.0, -1.3),//normal
            //origin: Point3::new(-0.94, -0.037, -3.342),//normal
            //origin: Point3::new(150.94, 150.037, -3.342),//rungholt
            //origin: Point3::new(23000.0, 14000.0, 10000.0),//powerplant
            target: Point3::new(0.7, 0.0, 0.6),
            //target: Point3::new(-0.418, -0.026, -2.435),
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
        let changed = if key_presses.contains(&Keycode::E) {
            self.origin = self.origin + 10.0 * self.direction;
            self.target = self.target + 10.0 * self.direction;
            true
        } else {
            changed
        };
        let changed = if key_presses.contains(&Keycode::Q) {
            self.origin = self.origin + -10.0 * self.direction;
            self.target = self.target + -10.0 * self.direction;
            true
        } else {
            changed
        };
        let changed = if key_presses.contains(&Keycode::H) {
            self.depth = if self.depth == 512 { 2 } else { 512 };
            println!("depth: {:?}", 2);
            true
        } else {
            changed
        };
        if key_presses.contains(&Keycode::P) {
            println!("origin: {:?}, direction: {:?}", self.origin, self.direction);
        }
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

    fn focus(&self, mut ray : &mut Ray, distance: f32, depth: u8) -> f32 {
        if depth == 0 { return distance }
        match self.scene.bvh.intersect_closest(&mut ray) {
            Some(ref intersection) => {
                let distance = distance + ray.distance;
                match intersection.material {
                    &Material::Diffuse{speculaty,..} if speculaty > 0.5 => {
                        let reflect = reflect(&ray.direction, &intersection.normal);
                        let intersection_point = ray.intersection();
                        ray.reset(intersection_point, reflect, f32::INFINITY);
                        return self.focus(ray, distance, depth - 1);
                    },
                    &Material::Dielectric{refraction_index_n1, refraction_index_n2, ..} => {
                        let normal = intersection.normal;
                        let inside = intersection.inside;
                        let intersection_point = ray.intersection();
                        let refracted_dir = if inside {
                                refract(&ray.direction, &-normal, refraction_index_n2, refraction_index_n1)
                            } else {
                                refract(&ray.direction, &normal, refraction_index_n1, refraction_index_n2)
                        };
                        if let Some(refracted_dir) = refracted_dir {
                            let schlick_reflection = if inside {
                                    schlick(&ray.direction, &-normal, refraction_index_n2, refraction_index_n1)
                                } else {
                                    schlick(&ray.direction, &normal, refraction_index_n1, refraction_index_n2)
                            };
                            if 0.5 < schlick_reflection {
                                // Reflected ray
                                let reflected_dir = if inside {
                                        reflect(&ray.direction, &-normal)
                                    } else {
                                        reflect(&ray.direction, &normal)
                                };
                                ray.reset(intersection_point, reflected_dir, f32::INFINITY);
                            } else {
                                // Refraction sampling
                                ray.reset(intersection_point, refracted_dir, f32::INFINITY);
                            }
                        } else {
                            // Full internal reflection
                            let reflected_dir = if inside {
                                    reflect(&ray.direction, &-normal)
                                } else {
                                    reflect(&ray.direction, &normal)
                            };
                            ray.reset(intersection_point, reflected_dir, f32::INFINITY);
                        }
                        return self.focus(ray, distance, depth - 1);
                    },
                    _ => return if distance == 0. { f32::INFINITY } else { distance },
                }
            },
            None => return if distance == 0. { f32::INFINITY } else { distance },
        }

    }
    fn update(&mut self) {
        self.direction = (self.target - self.origin).normalize();
        let unit_y = Vector3::new(0.0, 1.0, 0.0);
        self.right = unit_y.cross(self.direction);
        self.up = self.direction.cross(self.right);

        let mut ray = Ray::new(self.origin, self.direction, f32::INFINITY);

        let aspect_ratio = (self.width as f32) / (self.height as f32);

        self.focal_distance = f32::min(20.0, self.focus(&mut ray, 0.0, 5));

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
        let mut accumalated_color = Vector3::new(0.,0.,0.);
        let mut transport = Vector3::new(1., 1., 1.);
        let mut diffuse_bounce = false;
        for _ in 0..depth {
            match self.scene.bvh.intersect_closest(ray) {
                None => {
                    accumalated_color += transport.mul_element_wise(0.01 * self.scene.sample_skybox(ray.direction));;
                    break;
                },
                Some(Intersection{normal, inside, material}) => {
                    let intersection_point = ray.intersection();
                    match material {
                        &Material::Emissive { color } => {
                            if !diffuse_bounce { accumalated_color += transport.mul_element_wise(color); }
                            break;
                        },
                        &Material::Diffuse { speculaty, color} => {
                            if inside { break };
                            if let Some((nr_ligths, random_light)) = self.scene.bvh.random_light() {
                                let (point_on_light, area)= random_light.random_point();
                                let light_dir = (point_on_light - intersection_point).normalize();
                                let mut god_ray = Ray::new(intersection_point + 20. * f32::EPSILON * light_dir, light_dir, f32::INFINITY);
                                if let Some(intersection_on_light) = random_light.intersect(&mut god_ray) {
                                    let cos_intersection = normal.dot(light_dir);
                                    let cos_light = -intersection_on_light.normal.dot(light_dir);
                                    if cos_intersection > 0.0 && cos_light > 0.0 {
                                        // light is not behind surface point, trace shadow ray
                                        god_ray.distance -= f32::EPSILON; // ray should not hit the light in the intersect_any test
                                        if let None = self.scene.bvh.intersect_any(&mut god_ray) {
                                            let brdf = f32::consts::FRAC_1_PI * color;
                                            let light_color = random_light.is_light().unwrap(); // we selected a light
                                            let solid_angle = (cos_light * area) / (god_ray.distance * god_ray.distance);
                                            let light_pdf = 1.0 / solid_angle;
                                            let hemisphere_pdf = f32::consts::FRAC_1_PI * cos_intersection;
                                            multiple_important_sampling_pdf = light_pdf + hemisphere_pdf;
                                            // the estimated times this light gets sampled is 1 / nr_ligths, so we multiply this sample by nr_ligths
                                            let nee_estimate = nr_ligths as f32 * transport.mul_element_wise((cos_intersection / multiple_important_sampling_pdf) * light_color.mul_element_wise(brdf));
                                            accumalated_color += nee_estimate;
                                        }
                                    }
                                }
                            }

                            let Closed01(r0) = rand::random::<Closed01<f32>>();
                            if r0 < speculaty {
                                // Specular sampling
                                diffuse_bounce = false;
                                let reflected_dir = reflect(&ray.direction, &normal);
                                transport = transport.mul_element_wise(color);
                                ray.reset(intersection_point, reflected_dir, f32::INFINITY);
                            } else {
                                // russian_roulette only rays on a diffuse surface which already sent their nex_event_estimation ray
                                let Closed01(russian_roulette) = rand::random::<Closed01<f32>>();
                                let survival = transport.max().max(0.1); //minimum of 0.1 chance to survive and maximum of transport
                                if russian_roulette < survival {
                                    transport /= survival;
                                } else {
                                    break;
                                }
                                // Diffuse sampling
                                diffuse_bounce = true;
                                let diffuse_dir = cosine_weighted_diffuse(&normal);
                                let cos_intersection = diffuse_dir.dot(normal);
                                let brdf = f32::consts::FRAC_1_PI * color;
                                let hemisphere_pdf = f32::consts::FRAC_1_PI * cos_intersection;
                                transport = transport.mul_element_wise((cos_intersection / hemisphere_pdf) * brdf);
                                ray.reset(intersection_point, diffuse_dir, f32::INFINITY);
                            }
                        }
                        &Material::Dielectric { refraction_index_n1, refraction_index_n2, color } => {
                            diffuse_bounce = false;
                            if inside {
                                let absorbance = (Vector3::new(-1.,-1.,-1.) + color) * ray.distance;
                                let transparency = Vector3::new(absorbance.x.exp(), absorbance.y.exp(), absorbance.z.exp());
                                transport = transport.mul_element_wise(transparency);
                            }
                            let refracted_dir = if inside {
                                    refract(&ray.direction, &-normal, refraction_index_n2, refraction_index_n1)
                                } else {
                                    refract(&ray.direction, &normal, refraction_index_n1, refraction_index_n2)
                            };
                            if let Some(refracted_dir) = refracted_dir {
                                let schlick_reflection = if inside {
                                        schlick(&ray.direction, &-normal, refraction_index_n2, refraction_index_n1)
                                    } else {
                                        schlick(&ray.direction, &normal, refraction_index_n1, refraction_index_n2)
                                };
                                let Closed01(r0) = rand::random::<Closed01<f32>>();
                                if r0 < schlick_reflection {
                                    // Reflected ray
                                    if !inside {
                                        transport = transport.mul_element_wise(color);
                                    }
                                    let reflected_dir = if inside {
                                            reflect(&ray.direction, &-normal)
                                        } else {
                                            reflect(&ray.direction, &normal)
                                    };
                                    ray.reset(intersection_point, reflected_dir, f32::INFINITY);
                                } else {
                                    // Refraction sampling
                                    ray.reset(intersection_point, refracted_dir, f32::INFINITY);
                                }
                            } else {
                                // Full internal reflection
                                let reflected_dir = if inside {
                                        reflect(&ray.direction, &-normal)
                                    } else {
                                        reflect(&ray.direction, &normal)
                                };
                                ray.reset(intersection_point, reflected_dir, f32::INFINITY);
                            }
                        }
                    }
                }
            };
        }
        accumalated_color
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
