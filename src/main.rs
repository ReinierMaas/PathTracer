
extern crate cgmath;
extern crate sdl2;

use std::io;
use std::io::prelude::*;

use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

use std::collections::HashSet;

mod ray;
mod scene;
mod path_tracer;
mod camera;

use camera::Camera;
use scene::Scene;
use ray::Ray;

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

    let mut accumulator = Vec::with_capacity((WIDTH*HEIGHT*3) as usize);
    fill(&mut accumulator, 0.0);


    texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
        for y in 0..(HEIGHT as usize) {
            for x in 0..(WIDTH as usize) {
                let offset: usize = y*pitch + x*3;
                buffer[offset + 0] = x as u8;
                buffer[offset + 1] = y as u8;
                buffer[offset + 2] = 0;
            }
        }
        }).expect("mutate texture");

        renderer.clear();
        renderer.copy(&texture, None, Some(Rect::new(0, 0, WIDTH, HEIGHT))).unwrap();
        renderer.present();


    //let s = Scene::default_scene().unwrap();

    let mut camera = Camera::new(WIDTH, HEIGHT);


    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut key_presses = HashSet::new();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..}  => {
                    print!("{}", "YOOO");
                    break 'running
                },
                Event::KeyDown { keycode: Some(keycode) , ..} => {
                    if keycode == Keycode::Escape {
                        break 'running
                    }
                    key_presses.insert(keycode);
                    /*print!("{:?}\n", key_presses);
                    io::stdout().flush().ok().expect("no");*/
                },
                Event::KeyUp { keycode: Some(keycode) , ..} => {
                    key_presses.remove(&keycode);
                    /*print!("{:?}\n", key_presses);
                    io::stdout().flush().ok().expect("no");*/
                },
                _ => {}
            }
            if camera.handle_input(key_presses) {
                print!("{:?}", camera);
                io::stdout().flush().ok().unwrap();
            }
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
