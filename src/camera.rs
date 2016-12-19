extern crate cgmath;
extern crate rand;
extern crate sdl2;
use self::cgmath::{Vector3, Point3};
use self::cgmath::InnerSpace;
use self::cgmath::One;
use super::ray::Ray;
use std::f32;
use std::collections::HashSet;
use self::sdl2::keyboard::Keycode;
use std::io;
use std::io::prelude::*;

use scene::Scene;

#[derive(Debug)]
pub struct Camera {
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
    width: u32,
    height: u32,
    lens_size: f32,
    scene: Scene,
}

impl Camera {

    pub fn new(width: u32, height: u32, scene: Scene) -> Camera {
        let mut camera = Camera {
            width: width,
            height: height,
            lens_size: 0.04,
            origin: Point3::new(-0.94, -0.037, -3.342),
            target: Point3::new(-0.418, -0.026, -2.435),
            direction: Vector3::new(0.0,0.0,0.0),
            focal_distance: 0.0,
            p1: Point3::new(0.0,0.0,0.0),
            p2: Point3::new(0.0,0.0,0.0),
            p3: Point3::new(0.0,0.0,0.0),
            right: Vector3::new(0.0,0.0,0.0),
            up: Vector3::new(0.0,0.0,0.0),
            scene: scene
        };
        camera.update();
        camera
    }

    pub fn handle_input(&mut self, key_presses : &HashSet<Keycode>) -> bool  {
        self.target = self.origin + self.direction;

        let changed = if key_presses.contains(&Keycode::A) {
            self.origin = self.origin + (-0.1 * self.right);
            self.target = self.target + (-0.1 * self.right);
            true
        } else { false };
        let changed = if key_presses.contains(&Keycode::D) {
            self.origin =  self.origin + (0.1 * self.right);
            self.target =  self.target + (0.1 * self.right);
            true
        } else { changed };
        let changed = if key_presses.contains(&Keycode::W) {
            self.origin = self.origin + (0.1 * self.direction);
            true
        } else { changed };
        let changed = if key_presses.contains(&Keycode::S) {
            self.origin = self.origin + (-0.1 * self.direction);
            true
        } else { changed };
        let changed = if key_presses.contains(&Keycode::R) {
            self.origin = self.origin + (0.1 * self.up);
            self.target =  self.target + (0.1 * self.up);
            true
        } else { changed };
        let changed = if key_presses.contains(&Keycode::F) {
            self.origin = self.origin + (-0.1 * self.up);
            self.target = self.target + (-0.1 * self.up);
            true
        } else { changed };
        let changed = if key_presses.contains(&Keycode::Up) {
            self.target = self.target + (-0.1 * self.up);
            true
        } else { changed };
        let changed = if key_presses.contains(&Keycode::Down) {
            self.target = self.target + (0.1 * self.up);
            true
        } else { changed };
        let changed = if key_presses.contains(&Keycode::Left) {
            self.target = self.target + (-0.1 * self.right);
            true
        } else { changed };
        let changed = if key_presses.contains(&Keycode::Right) {
            self.target = self.target + (0.1 * self.right);
            true
        } else { changed };
        if changed {
            self.update();
            true
        } else {
            false
        }

    }

    fn update(&mut self) {
        // construct a look-at matrix
        self.direction = (self.target-self.origin).normalize();
        self.up = Vector3::new(0.0,1.0,0.0);
        self.right = self.direction.cross(self.right);
        self.up = self.direction.cross(self.right);

        let mut ray = Ray::new(self.origin, self.direction, f32::INFINITY);
        self.scene.intersect(& mut ray);
    }


    /// sample a ray by shooting it through the scene
    pub fn sample(&self, ray : & mut Ray, depth: u32) -> Vector3<f32> {
        self.scene.intersect(ray);
        //print!("{:?}\n", ray.direction);
        let sample = match ray.material {
            None => {
                self.scene.sample_skybox(ray.direction)
            },
            Some(ref material) => {
                self.scene.sample_skybox(ray.direction)
            }
        };
        sample
    }

    /// generates a nice Ray (TODO better integer type)
    pub fn generate(&self, x: u32, y: u32) -> Ray {
        // NOTE: we do not have to keep track of a
        // pool of random number generators, each
        // thread in rust has its own random
        // number generator by default :)
        let r0:f32 = rand::random();
        let r1:f32 = rand::random();
        let r2:f32 = rand::random();
        let r2 = r2 - 0.5;
        let r3:f32 = rand::random();
        let r3 = r3 - 0.5;

        // calculate sub-pixel ray target position on screen plane
        let u = ((x as f32) + r0) / (self.width as f32);
        let v = ((y as f32) + r1) / (self.height as f32);
        let target = self.p1 + u * (self.p2 - self.p1) + v * (self.p3 - self.p1);
        let origin = self.origin + self.lens_size * (r2 * self.right + r3 * self.up);
        let direction = (target-origin).normalize();

        // hmm all directions are the same. that seems to be a bug =)

        print!("{:?}\n", direction);
        Ray::new(origin, direction, f32::INFINITY)

    }
}
