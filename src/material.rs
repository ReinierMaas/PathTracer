extern crate cgmath;
use self::cgmath::Vector3;

#[derive(Debug)]
pub enum Material {
    CheckerBoard,
    Realistic {
        emissive: Emissive,
        diffuse: Vector3<f32>,
    }
}

#[derive(Debug)]
pub enum Emissive {
    Emissive,
    NonEmissive {
        refl: f32,
        // This is the refraction index
        refr: f32,
    }
}

const LIGHT_SCALE: f32 = 1.0;

pub const LIGHT_COLOR: Vector3<f32> =
    Vector3 {
        x: 8.5 * LIGHT_SCALE,
        y: 8.5 * LIGHT_SCALE,
        z: 7.0 * LIGHT_SCALE,
    };
