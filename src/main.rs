
extern crate cgmath;
extern crate sdl2;

use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

mod ray;
mod scene;
mod path_tracer;

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


    let s = scene::Scene::default_scene().unwrap();



    let mut event_pump = sdl_context.event_pump().unwrap();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} 
                | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => {}
            }
        }
    }
}
