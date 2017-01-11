
extern crate cgmath;
extern crate sdl2;
extern crate num_cpus;
extern crate spmc;

use std::io;
use std::sync::mpsc;
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

struct Accumulator {
    spp: u32,
    buf: Vec<Vector3<f32>>,
    width: usize,
    height: usize,
}

impl Accumulator {
    pub fn new(width: usize, height: usize) -> Accumulator {
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

struct Game<'a> {
    accumulator: Accumulator,
    camera: Camera,
    workers: Vec<Option<thread::JoinHandle<()>>>,
    tx: spmc::Sender<WorkerMsg<'a>>,
    rx: mpsc::Receiver<GameMsg<'a>>,
}

impl <'a> Drop for Game<'a> {
    fn drop (&mut self) {
        for handle in &mut self.workers {
            handle.take().unwrap().join().unwrap();
        }
    }
}

enum GameMsg<'a> {
    Traced(&'a mut [f32])
}

enum WorkerMsg<'a> {
    Quit,
    Trace(&'a mut [f32]),
}

impl <'a> Game<'a> {
    fn worker(tx : mpsc::Sender<GameMsg>, rx : spmc::Receiver<WorkerMsg>) {
        loop {
            match rx.recv().unwrap() {
                WorkerMsg::Quit => break,
                WorkerMsg::Trace(accum) => {
                }
            }
        }
    }

    fn new(width: usize, height: usize, num_workers: usize) -> Result<Game<'a>, io::Error> {
        print!("Setting up game...\n");
        let scene = try!(Scene::default_scene());
        let accumulator = Accumulator::new(width, height);
        let camera = Camera::new(width, height, scene);

        let mut workers = Vec::new();

        let (game_tx, worker_rx) = spmc::channel();

        let (worker_tx, game_rx) = mpsc::channel();

        for c in 0..num_workers {
            let rx = worker_rx.clone();
            let tx = worker_tx.clone();
            workers.push(Some(thread::spawn(move || {
                loop {
                }
                Game::worker(tx, rx)
            })));
        }
        let game = Game {
            accumulator: accumulator,
            camera: camera,
            workers: workers,
            rx: game_rx,
            tx: game_tx,
        };
        print!("Finished setting up game...\n");
        Ok(game)
    }

    fn tick(&mut self, key_presses : &HashSet<Keycode>) {
        if self.camera.handle_input(&key_presses) {
            self.accumulator.clear();

        }

        self.accumulator.spp += 1;

        for y in 0..self.accumulator.height {
            for x in 0..self.accumulator.width {
                let mut ray = self.camera.generate(x,y);
                let idx = x + y * self.accumulator.width;
                self.accumulator.buf[idx] += self.camera.sample(&mut ray, 20);
            }
        }




    }
    fn render(&self, texture : &mut sdl2::render::Texture) {
        let width = self.accumulator.width;
        let height = self.accumulator.height;
        let scale = 1.0 / (self.accumulator.spp as f32);
        texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
            for y in 0..height {
                for x in 0..width {
                    let offset: usize = y*pitch + x*3;
                    let rgb = vec_to_rgb(scale*self.accumulator.buf[x+y*width]);
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


    let mut game = Game::new(WIDTH, HEIGHT, 0).unwrap();




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
        renderer.copy(&texture, None, Some(Rect::new(0, 0, WIDTH as u32, HEIGHT as u32))).unwrap();
        renderer.present();
    }
}

