
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
mod scene;
mod path_tracer;
mod camera;

use camera::Camera;
use scene::Scene;
use ray::Ray;



use std::cell::Cell;

struct Accumulator {
    spp: u32,
    buf: Vec<Vector3<f32>>,
    width: u32,
    height: u32,
}

impl Accumulator {
    pub fn new(width: u32, height: u32) -> Accumulator {
        let mut accum = Accumulator {
           buf: Vec::with_capacity((width*height) as usize),
           spp: 0,
           width: width,
           height: height,
        };
        accum.clear();
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
    texture: sdl2::render::Texture,
    camera: Camera,
}

impl Game {
    fn new(width: u32, height: u32, texture: sdl2::render::Texture) -> Result<Game, io::Error> {
        let scene = try!(Scene::default_scene());
        Ok(Game {
            accumulator: Accumulator::new(width, height),
            texture: texture,
            camera: Camera::new(width, height, scene),
        })
    }

    fn tick(&mut self, key_presses : &HashSet<Keycode>) {
        if (self.camera.handle_input(&key_presses)) {
            self.accumulator.clear();
        }

        self.accumulator.spp += 1;
        let scale = 1.0 / (self.accumulator.spp as f32);

        for y in 0..self.accumulator.height {
            for x in 0..self.accumulator.width {
                let mut ray = self.camera.generate(x,y);
                let idx = (x + y * self.accumulator.width) as usize;
                self.accumulator.buf[idx] += self.camera.sample(&mut ray, 0);
            }
        }


        let width = self.accumulator.width as usize;
        let height = self.accumulator.height as usize;
        self.texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
            for y in 0..(width as usize) {
                for x in 0..(height as usize) {
                    let offset: usize = y*pitch + x*3;
                    let rgb = vec_to_rgb(scale*self.accumulator.buf[x+y*self.accumulator.width as usize]);

                    buffer[offset + 0] = rgb.x;
                    buffer[offset + 1] = rgb.y;
                    buffer[offset + 2] = rgb.z;
                }
            }
            }).expect("mutate texture");


    }
    fn render() {
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



        renderer.clear();
        renderer.copy(&texture, None, Some(Rect::new(0, 0, WIDTH, HEIGHT))).unwrap();
        renderer.present();

    let mut event_pump = sdl_context.event_pump().unwrap();


    let mut game = Game::new(WIDTH, HEIGHT, texture).unwrap();

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
            game.tick(&key_presses);
        }

        //tick(renderer);
    }
}

/// Fills a vector with a value
fn fill<T : Copy>(vec : &mut Vec<T>, value : T) {
    for i in 0..vec.len() {
        vec[i] = value;
    }
}
