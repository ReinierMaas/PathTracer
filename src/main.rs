#![feature(thread_local)]
#![allow(dead_code)]

extern crate rand;
extern crate cgmath;
extern crate sdl2;
extern crate num_cpus;
extern crate spmc;
extern crate scoped_threadpool;
extern crate tobj;
extern crate thread_local;

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
mod camera;
mod bvh;
mod mesh;

use camera::Camera;
use scene::Scene;
use primitive::sphere::Sphere;
use primitive::triangle::Triangle;

//const WIDTH: usize = 800;
//const HEIGHT: usize = 600;

// https://gist.github.com/jaredwinick/5073432
fn interleave_morton(x: u32, y: u32) -> u32 {
    let b = [0x55555555, 0x33333333, 0x0F0F0F0F, 0x00FF00FF];
    let s = [1, 2, 4, 8];

    let x = (x | (x << s[3])) & b[3];
    let x = (x | (x << s[2])) & b[2];
    let x = (x | (x << s[1])) & b[1];
    let x = (x | (x << s[0])) & b[0];
    let y = (y | (y << s[3])) & b[3];
    let y = (y | (y << s[2])) & b[2];
    let y = (y | (y << s[1])) & b[1];
    let y = (y | (y << s[0])) & b[0];
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

    let mut accum = vec![Vector3::new(0.,0.,0.); WIDTH*HEIGHT];
    let mut spp: f32 = 0.;


    //let scene = Scene::<Sphere>::default_scene().expect("scene");
    //let scene = Scene::<Triangle>::scene(&std::path::Path::new("./models/cube.obj")).expect("scene");
    let scene = Scene::<Triangle>::scene(&std::path::Path::new("./models/dragon.obj")).expect("scene");
    //let scene = Scene::<Triangle>::scene(&std::path::Path::new("./models/buddha.obj")).expect("scene");
    //let scene = Scene::<Triangle>::scene(&std::path::Path::new("./models/rungholt.obj")).expect("scene");
    //let scene = Scene::<Triangle>::scene(&std::path::Path::new("./models/powerplant.obj")).expect("scene");
    let mut camera = Camera::new(WIDTH, HEIGHT, scene);


    let num_cpus = num_cpus::get();
    let mut key_presses = HashSet::new();
    let mut pool = scoped_threadpool::Pool::new(num_cpus as u32);

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

        if camera.handle_input(&key_presses) {
            for mut x in &mut accum[..] {
                *x = Vector3::new(0.,0.,0.);
            }
            spp = 0.;
        }
        spp += 1.0;

        let scale: f32 = 1.0 / spp;
        texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
            let camera = &camera;
            let accum = &mut accum;
            pool.scoped(|scope| {
                let accum_iter = accum.chunks_mut((WIDTH*HEIGHT)/num_cpus);
                let fb_iter = buffer.chunks_mut((WIDTH*HEIGHT*3)/num_cpus);
                for (core_id, (mut chunk, mut chunk2)) in &mut accum_iter.zip(fb_iter).enumerate() {
                    scope.execute(move||{
                        let start_y = core_id * (HEIGHT / num_cpus);
                        for y in 0.. HEIGHT / num_cpus {
                            for x in 0..WIDTH {
                                let mut ray = camera.generate(x,y+start_y);
                                let idx = x + y * WIDTH;
                                chunk[idx] += camera.sample(&mut ray, 32);
                                let offset = y*pitch + x*3;
                                let rgb = vec_to_rgb(scale*chunk[idx]);
                                chunk2[offset + 0] = rgb.x;
                                chunk2[offset + 1] = rgb.y;
                                chunk2[offset + 2] = rgb.z;

                            }
                        }
                    });
                }
            });
        }).unwrap();
        //game.tick(&key_presses, &mut accum, &mut samples_per_pixel);
        //game.render(&mut texture, &accum, samples_per_pixel);
        renderer.copy(&texture, None, Some(Rect::new(0, 0, WIDTH as u32, HEIGHT as u32))).unwrap();
        renderer.present();
    }
}

