
extern crate cgmath;
extern crate sdl2;

use std::io;
use std::io::prelude::*;

use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use std::collections::HashSet;

use cgmath::{Vector3};

mod ray;
mod material;
mod primitive;
mod scene;
mod path_tracer;
mod camera;
mod bvh;

use camera::Camera;
use scene::Scene;

struct Accumulator {
    spp: u32,
    buf: Vec<Vector3<f32>>,
    width: u32,
    height: u32,
}

impl Accumulator {
    pub fn new(width: u32, height: u32) -> Accumulator {
        print!("Setting up accumulator...\n");
        let mut accum = Accumulator {
           buf: vec![Vector3::new(0.0,0.0,0.0); (width*height) as usize],
           spp: 0,
           width: width,
           height: height,
        };
        accum.clear();
        print!("Finished setting up accumulator...\n");
        accum
    }
    pub fn clear(&mut self) {
        for i in 0..self.buf.len() {
            self.buf[i] = Vector3::new(0.0,0.0,0.0);
        }
        self.spp = 0;
    }
}

struct Game {
    accumulator: Accumulator,
    camera: Camera,
}

impl Game {
    fn new(width: u32, height: u32) -> Result<Game, io::Error> {
        print!("Setting up game...\n");
        let scene = try!(Scene::default_scene());
        let accum = Accumulator::new(width, height);
        let camera = Camera::new(width, height, scene);
        let game = Game { accumulator: accum, camera: camera };
        print!("Finished setting up game...\n");
        Ok(game)
    }

    fn tick(&mut self, key_presses : &HashSet<Keycode>) {
        print!("tick!\n");
        if (self.camera.handle_input(&key_presses)) {
            print!("clear\n");
            self.accumulator.clear();
        }

        self.accumulator.spp += 1;

        for y in 0..self.accumulator.height {
            for x in 0..self.accumulator.width {
                let mut ray = self.camera.generate(x,y);
                let idx = (x + y * self.accumulator.width) as usize;
                self.accumulator.buf[idx] += self.camera.sample(&mut ray, 0);
            }
        }




    }
    fn render(&self, texture : &mut sdl2::render::Texture) {
        let width = self.accumulator.width as usize;
        let height = self.accumulator.height as usize;
        let scale = 1.0 / (self.accumulator.spp as f32);
        texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
            for y in 0..(height as usize) {
                for x in 0..(width as usize) {
                    let offset: usize = y*pitch + x*3;
                    let rgb = vec_to_rgb(scale*self.accumulator.buf[x+y*width as usize]);
                    buffer[offset + 0] = rgb.x;
                    buffer[offset + 1] = rgb.y;
                    buffer[offset + 2] = rgb.z;
                }
            }
            }).expect("mutate texture");
    }
}

fn vec_to_rgb(vec : Vector3<f32>) -> Vector3<u8> {
    Vector3::new((255.0 as f32).min( 256.0 * 1.5 * vec.x.sqrt()) as u8,
                 (255.0 as f32).min( 256.0 * 1.5 * vec.x.sqrt()) as u8,
                 (255.0 as f32).min( 256.0 * 1.5 * vec.x.sqrt()) as u8)
}
fn main() {
    const WIDTH: u32 = 800;
    const HEIGHT: u32 = 600;
    let sdl_context = sdl2::init().expect("SDL Context");
    let video_subsystem = sdl_context.video().expect("Video subsystem");

    let window = video_subsystem.window("Pathtracer", WIDTH, HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .expect("Window");


    let mut renderer = window.renderer().build().expect("Renderer");

    let mut texture = renderer.create_texture_streaming(
        PixelFormatEnum::RGB24, WIDTH, HEIGHT).expect("Texture");



    let mut event_pump = sdl_context.event_pump().unwrap();


    let mut game = Game::new(WIDTH, HEIGHT).unwrap();


    let mut key_presses = HashSet::new();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..}  => {
                    break 'running
                },
                Event::KeyDown { keycode: Some(keycode) , ..} => {
                    if keycode == Keycode::Escape {
                        break 'running
                    }
                    key_presses.insert(keycode);
                },
                Event::KeyUp { keycode: Some(keycode) , ..} => {
                    key_presses.remove(&keycode);
                },
                _ => {}
            }

        }

        game.tick(&key_presses);
        game.render(&mut texture);
        renderer.copy(&texture, None, Some(Rect::new(0, 0, WIDTH, HEIGHT))).unwrap();
        renderer.present();
    }
}

/// Fills a vector with a value
fn fill<T : Copy>(vec : &mut Vec<T>, value : T) {
    for i in 0..vec.len() {
        vec[i] = value;
    }
}
