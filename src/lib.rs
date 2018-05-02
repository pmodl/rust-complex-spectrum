extern crate image;
extern crate num_complex;
extern crate rayon;
//use rayon::prelude::*;
use image::*;
use num_complex::{Complex64};
use std::fs::File;
use std::f64::consts::PI;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vectors() {
        let zero = Complex64::new(0.0, 0.0);
        let one = Complex64::new(1.0, 0.0);
        let z = Complex64::new(2.0, 1.0);

        // f(z) = 0
        let f1 = C64Terms(Vec::new());
        // f(z) = 2 z^2 + z + 1
        let f2 = C64Terms(vec![one, one, 2.0 * one]);
        assert_eq!(f1.eval_at(z), zero);
        assert_eq!(f2.eval_at(z), Complex64::new(9.0, 9.0));
    }
}

pub struct ImageDesc {
    pub width: u32,
    pub height: u32,
    pub xres: f64,
    pub yres: f64,
}

pub enum LightnessAlg {
    Exp,
    Exp2,
    LogFrac,
    ModSq,
    ModFrac,
    No,
}
use LightnessAlg::*;

fn angle_to_hue(theta: f64) -> f64{
    return 3.0 + theta * 3.0 / PI;
}

pub trait PixelGenerator {
    fn rgb_complex(&self, z: Complex64, repeat: &Option<&Fn(Complex64) -> f64>) -> Rgb<u8>;
}

pub trait ComplexFunction {
    fn eval_at(&self, z: Complex64) -> Complex64;
}

// Treat as a list of terms of a polynomial
pub struct C64Terms(pub Vec<Complex64>);
impl ComplexFunction for C64Terms {
    fn eval_at(&self, z: Complex64) -> Complex64 {
        let mut sum = Complex64::new(0.0, 0.0);
        let mut zn = Complex64::new(1.0, 0.0);
        for val in &self.0 {
            sum += zn * val;
            zn *= z;
        }
        return sum;
    }
}

// Treat as a list of roots of a polynomial
pub struct C64Roots(pub Vec<Complex64>);
impl ComplexFunction for C64Roots {
    fn eval_at(&self, z: Complex64) -> Complex64 {
        let mut prod = Complex64::new(1.0, 0.0);
        for val in &self.0 {
            prod *= z - val;
        }
        return prod;
    }
}

// Roots of unity
pub struct C64Unity(pub usize);
impl ComplexFunction for C64Unity {
    fn eval_at(&self, z:Complex64) -> Complex64 {
        z.powf(self.0 as f64) - 1.0
    }
}


impl ComplexFunction for Fn(Complex64) -> Complex64 {
    fn eval_at(&self, z: Complex64) -> Complex64 { self(z) }
}

impl PixelGenerator for LightnessAlg {
    fn rgb_complex(&self, z: Complex64, repeat: &Option<&Fn(Complex64) -> f64>) -> Rgb<u8>{
        let r2 = z.norm_sqr();
        let l1 = match *self {
            Exp     => 1.0 - (-r2.sqrt()).exp(),
            Exp2    => 1.0 - (-r2.sqrt()).exp2(),
            ModSq   => r2 / (r2 + 1.0),
            No      => 0.5,
            _       => 0.0
        };
        let l = match *repeat {
            None    => l1,
            Some(f) => {
                let l2 = f(z).fract();
                if l2 > 0.0 { 0.8 * l1 + 0.2 * l2 }
                else {0.8 * l1 + 0.2 + 0.2 * l2}
            }
        };
        return hsl_to_rgb(angle_to_hue(z.arg()), 1.0, l);
    }
}

fn hsl_to_rgb(h: f64, s: f64, l: f64) -> Rgb<u8> {
    let l_capped = if l > 0.999 { 0.999 } else { l };
    let mut c0 = (1.0 - f64::abs(2.0 * l_capped - 1.0)) * s;
    let m = l_capped - 0.5 * c0;
    let scale = f64::abs(h % 2.0 - 1.0);
    let mut c1 = c0 * scale;
    c0 += m;
    c1 += m;
    let c2 = m;

    if c0 >= 1.0 { c0 = 0.999; }
    if c1 >= 1.0 { c1 = 0.999; }

    let vals: [f64;3];
    match h as u8 % 6 {
        3 => vals = [c0, c1, c2],
        4 => vals = [c1, c0, c2],
        5 => vals = [c2, c0, c1],
        0 => vals = [c2, c1, c0],
        1 => vals = [c1, c2, c0],
        2 => vals = [c0, c2, c1],
        _ => vals = [0.0, 0.0, 0.0],
    };
    let mut ret: [u8;3] = [0, 0, 0];
    for i in 0..3 {
        ret[i] = (256.0 * vals[i]) as u8;
    }
    return Rgb(ret);
}

pub fn domain_color<T: PixelGenerator>(
    i: &ImageDesc,
    f: &Fn(Complex64) -> Complex64,
    imgname: &str,
    method: T,
    repeat: &Option<&Fn(Complex64) -> f64>
){
    let yoffset = i.height as f64 * i.yres / 2.0;
    let xoffset = i.width as f64 * i.xres / 2.0;

    // Create a new ImgBuf with width: imgx and height: imgy
    let mut imgbuf = image::ImageBuffer::new(i.width, i.height);

    let draw_pixel = |(x,y,pixel): (u32, u32, &mut Rgb<u8>)| {
        let cy = yoffset - y as f64 * i.yres as f64;
        let cx = x as f64 * i.xres as f64 - xoffset;
        let z = f(Complex64::new(cx, cy));

        *pixel = method.rgb_complex(z, repeat);
    };

    // Iterate over the coordinates and pixels of the image
    imgbuf.enumerate_pixels_mut().for_each(draw_pixel);

    // We must indicate the image's color type and what format to save as
    image::ImageRgb8(imgbuf).save(imgname).unwrap();
}

pub fn domain_color_simple(
    i: &ImageDesc,
    f: &Fn(Complex64) -> Complex64,
    imgname: &str,
) {
    domain_color(i, f, imgname, Exp2, &None);
}
