use scene::Scene;
use ray::Ray;
use cgmath::Vector3;
use sdl2::render::Texture;

struct PathTracer<'a> {
    width: usize,
    height: usize,
    screen: & 'a Texture,
    accumulator: & 'a [Vector3<f32>],
    scene: Scene,
}

impl <'a> PathTracer<'a> {
    fn sample(ray: Ray, depth: usize) -> Vector3<f32> {
        panic!("not implemented")
    }
}
