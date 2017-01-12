
extern crate cgmath;
extern crate sdl2;
extern crate num_cpus;
extern crate spmc;
extern crate scoped_threadpool;
use scoped_threadpool::Pool;

use std::io;
use std::sync::mpsc;
use std::sync::Mutex;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::thread;

use std::collections::HashSet;

use cgmath::{Vector3};

mod ray;
mod material;
mod primitive;
mod scene;
mod camera;
mod bvh;

use camera::Camera;
use scene::Scene;

const WIDTH: usize = 800;
const HEIGHT: usize = 600;

// https://gist.github.com/jaredwinick/5073432
fn interleave_morton(x: u32, y: u32) -> u32 {
    let B = [0x55555555, 0x33333333, 0x0F0F0F0F, 0x00FF00FF];
    let S = [1, 2, 4, 8];

    let x = (x | (x << S[3])) & B[3];
    let x = (x | (x << S[2])) & B[2];
    let x = (x | (x << S[1])) & B[1];
    let x = (x | (x << S[0])) & B[0];
    let y = (y | (y << S[3])) & B[3];
    let y = (y | (y << S[2])) & B[2];
    let y = (y | (y << S[1])) & B[1];
    let y = (y | (y << S[0])) & B[0];
    let z = x | (y << 1);
    z
}

fn deinterleave_morton(z: u32) -> (u32, u32) {
      let x = z & 0x55555555;
      let x = (x | (x >> 1)) & 0x33333333;
      let x = (x | (x >> 2)) & 0x0F0F0F0F;
      let x = (x | (x >> 4)) & 0x00FF00FF;
      let x = (x | (x >> 8)) & 0x0000FFFF;

      let y = (z >> 1) & 0x55555555;
      let y = (y | (y >> 1)) & 0x33333333;
      let y = (y | (y >> 2)) & 0x0F0F0F0F;
      let y = (y | (y >> 4)) & 0x00FF00FF;
      let y = (y | (y >> 8)) & 0x0000FFFF;

      (x,y)

}

fn vec_to_rgb(vec : Vector3<f32>) -> Vector3<u8> {
    Vector3::new((255.0 as f32).min( 256.0 * 1.5 * vec.x.sqrt()) as u8,
                 (255.0 as f32).min( 256.0 * 1.5 * vec.y.sqrt()) as u8,
                 (255.0 as f32).min( 256.0 * 1.5 * vec.z.sqrt()) as u8)
}
fn main() {
    const WIDTH: usize = 800;
    const HEIGHT: usize = 600;
    let sdl_context = sdl2::init().expect("SDL Context");
    let video_subsystem = sdl_context.video().expect("Video subsystem");



    let window = video_subsystem.window("Pathtracer", WIDTH as u32, HEIGHT as u32)
        .position_centered()
        .opengl()
        .build()
        .expect("Window");


    let mut renderer = window.renderer().build().expect("Renderer");

    let mut texture = renderer.create_texture_streaming(
        PixelFormatEnum::RGB24, WIDTH as u32, HEIGHT as u32).expect("Texture");



    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut accum = [Vector3::new(0.,0.,0.); WIDTH*HEIGHT];
    let mut spp = 0.;


    let scene = Scene::default_scene().expect("scene");
    let mut camera = Camera::new(WIDTH, HEIGHT, scene);

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

        // TICK

        if camera.handle_input(&key_presses) {
            for mut x in &mut accum[..] {
                *x = Vector3::new(0.,0.,0.);
            }
            spp = 0.;
        }
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let mut ray = camera.generate(x,y);
                let idx = (x + y * WIDTH);
                accum[idx] += camera.sample(&mut ray, 20);
            }
        }

        spp += 1.0;

        // RENDER
        let scale = 1.0 / spp;
        texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
            for y in 0..HEIGHT {
                for x in 0..WIDTH {
                    let offset: usize = y*pitch + x*3;
                    let rgb = vec_to_rgb(scale*accum[x+y*WIDTH]);
                    buffer[offset + 0] = rgb.x;
                    buffer[offset + 1] = rgb.y;
                    buffer[offset + 2] = rgb.z;
                }
            }
            }).expect("mutate texture");
        //game.tick(&key_presses, &mut accum, &mut samples_per_pixel);
        //game.render(&mut texture, &accum, samples_per_pixel);
        renderer.copy(&texture, None, Some(Rect::new(0, 0, WIDTH as u32, HEIGHT as u32))).unwrap();
        renderer.present();
    }
}

