extern crate sdl2;

use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

fn main() {
    let sdl_context = sdl2::init().expect("SDL Context");
    let video_subsystem = sdl_context.video().expect("Video subsystem");

    let window = video_subsystem.window("Pathtracer", 800, 600)
        .position_centered()
        .opengl()
        .build()
        .expect("Window");


   let mut renderer = window.renderer().build().expect("Renderer"); 

   let mut texture = renderer.create_texture_streaming(
        PixelFormatEnum::RGB24, 256, 256).expect("Texture");


   texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
       for y in 0..256 {
           for x in 0..256 {
               let offset = y*pitch + x*3;
               buffer[offset + 0] = x as u8;
               buffer[offset + 1] = y as u8;
               buffer[offset + 2] = 0;
           }
       }
    }).expect("mutate texture");

    renderer.clear();
    renderer.copy(&texture, None, Some(Rect::new(100, 100, 256, 256)));
    renderer.copy_ex(&texture, None, 
        Some(Rect::new(450, 100, 256, 256)), 30.0, None, false, false).unwrap();
    renderer.present();

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
