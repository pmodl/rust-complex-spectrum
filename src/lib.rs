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
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

pub struct ImageDesc {
    pub width: u32,
    pub height: u32,
    pub xres: f64,
    pub yres: f64,
}

fn complex_polar2_rgb(r2: f64, theta: f64) -> Rgb<u8> {
    let h = 3.0 + theta * 3.0 / PI;
    let s = 1.0;
    let l;
    if r2 > (2 as u64).pow(15) as f64{
        l = 255.875 / 256.0;
    }
    else {
        l = r2 / (r2 + 1.0);
    }
    return hsl_to_rgb(h, s, l);
}

fn complex_polar_rgb(r: f64, theta: f64) -> Rgb<u8> {
    return complex_polar2_rgb(r * r, theta);
}

fn hsl_to_rgb(h: f64, s: f64, l: f64) -> Rgb<u8> {
    let mut c0 = (1.0 - f64::abs(2.0 * l - 1.0)) * s;
    let m = l - 0.5 * c0;
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

fn complex_rgb(z: Complex64) -> Rgb<u8> {
    let theta = z.arg();
    let r2 = z.norm_sqr();
    return complex_polar_rgb(r2, theta);
}

pub fn complex_spectrum(i: &ImageDesc, f: &Fn(Complex64) -> Complex64, imgname: &str) {

    let yoffset = i.height as f64 * i.yres / 2.0;
    let xoffset = i.width as f64 * i.xres / 2.0;

    // Create a new ImgBuf with width: imgx and height: imgy
    let mut imgbuf = image::ImageBuffer::new(i.width, i.height);

    let draw_pixel = |(x, y, pixel): (u32, u32, &mut Rgb<u8>)| {
        let cy = yoffset - y as f64 * i.yres as f64;
        let cx = x as f64 * i.xres as f64 - xoffset;
        let z = Complex64::new(cx, cy);

        *pixel = complex_rgb(f(z));
    };

    // Iterate over the coordinates and pixels of the image
    imgbuf.enumerate_pixels_mut()
        .for_each(draw_pixel);

    // Save the image as "imgname"
    let ref mut fout = File::create(imgname).unwrap();

    // We must indicate the image's color type and what format to save as
    image::ImageRgb8(imgbuf).save(fout, image::PNG).unwrap();
}
