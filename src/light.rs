#![allow(dead_code)]
use math::{Vec3f, Zero, EPS_COSINE};
use math::vector_traits::*;
use geometry::{Frame};
use brdf;
use std::f32;

#[derive(Debug, Clone)]
pub struct Illumination {
    pub dir_to_light: Vec3f,
    pub dist_to_light: f32,
    pub dir_pdf_w: f32,
    pub intensity: Vec3f,
}

#[derive(Debug, Clone)]
pub struct Radiance {
    pub intensity: Vec3f,
    pub dir_pdf_a: f32,
}

pub trait Light {
    fn illuminate(&self, receiving_pnt: Vec3f, rands: (f32, f32)) -> Illumination;
    fn get_radiance(&self, dir: &Vec3f, hit: Vec3f) -> Radiance;
    fn is_delta(&self) -> bool;
}

#[derive(Debug, Clone)]
pub struct AreaLight {
    p0: Vec3f,
    e1: Vec3f,
    e2: Vec3f,
    frame: Frame,
    intensity: Vec3f,
    inv_area: f32,
}

#[derive(Debug, Clone)]
pub struct BackgroundLight {
    pub intensity: Vec3f,
    pub scale: f32
}

impl AreaLight {
    pub fn new(p0: Vec3f, p1: Vec3f, p2: Vec3f, intensity: Vec3f) -> AreaLight {
        let e1 = p1 - p0;
        let e2 = p2 - p0;
        let normal = e1.cross(&e2);
        AreaLight {
            p0: p0,
            e1: e1,
            e2: e2,
            frame: Frame::from_z(normal),
            inv_area: 2.0 / normal.norm(),
            intensity: intensity
        }
    }
}

impl Light for AreaLight {
    fn illuminate(&self, receiving_pnt: Vec3f, rands: (f32, f32)) -> Illumination {
        let uv = brdf::uniform_triangle_sample(rands);
        let light_pnt = self.p0 + self.e1 * uv.x + self.e2 * uv.y;
        let dir_to_light = light_pnt - receiving_pnt;
        let dist_sqr = dir_to_light.sqnorm();
        let dir_to_light = dir_to_light.normalize();
        let cos_normal_dir = self.frame.normal().dot(&-dir_to_light);
        if cos_normal_dir <= EPS_COSINE {
            Illumination {
                dir_to_light: dir_to_light,
                dist_to_light: dist_sqr.sqrt(),
                dir_pdf_w: f32::NEG_INFINITY, // positive/n, (n < 0) => -inf
                intensity: Zero::zero()
            }
        } else {
            Illumination {
                dir_to_light: dir_to_light,
                dist_to_light: dist_sqr.sqrt(),
                dir_pdf_w: self.inv_area * dist_sqr / cos_normal_dir,
                intensity: self.intensity
            }
        }
    }

    fn get_radiance(&self, dir: &Vec3f, _hit: Vec3f) -> Radiance {
        let cos_out_l = self.frame.normal().dot(&-dir.clone()).max(0.0);
        if cos_out_l == 0.0 {
            Radiance {
                intensity: Zero::zero(),
                dir_pdf_a: f32::NEG_INFINITY
            }
        } else {
            Radiance {
                intensity: self.intensity,
                dir_pdf_a: self.inv_area
            }
        }
    }

    fn is_delta(&self) -> bool {
        false
    }
}

impl Light for BackgroundLight {
    fn illuminate(&self, _receiving_pnt: Vec3f, rands: (f32, f32)) -> Illumination {
        let (dir, dir_pdf_w) = brdf::uniform_sphere_sample_w(rands);
        Illumination {
            dir_to_light: dir,
            dir_pdf_w: dir_pdf_w,
            dist_to_light: 1.0e35,
            intensity: self.intensity * self.scale
        }
    }

    fn get_radiance(&self, _dir: &Vec3f, _hit: Vec3f) -> Radiance {
        let dir_pdf_w = brdf::uniform_sphere_pdf_w();
        Radiance {
            intensity: self.intensity * self.scale,
            dir_pdf_a: dir_pdf_w, // it's ok only for background light
        }
    }

    fn is_delta(&self) -> bool {
        false
    }
}
