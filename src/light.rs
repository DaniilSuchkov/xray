#![allow(dead_code)]
use math::{Vec3f, Zero, EPS_COSINE};
use math::vector_traits::*;
use geometry::{Frame, Geometry, Ray};
use brdf;
// use std::f32;

#[derive(Debug, Clone)]
pub struct BackgroundLight {
    pub intensity: Vec3f,
    pub scale: f32
}

#[derive(Debug, Clone)]
pub struct PointLight {
    pub intensity: Vec3f,
    pub position: Vec3f,
}

pub struct Illumination {
    pub radiance: Vec3f,
    pub dir_to_light: Vec3f,
    pub dist_to_light: f32,
    // pub dir_pdf_w: f32,
}

pub struct Radiation {
    pub radiance: Vec3f,
    // pub dir_pdf_w: f32,
}

pub trait Light {
    // out_ray - "out" in physical meaning, in trace from eye to light it's "incoming"
    fn radiate(&self, out_ray: &Ray) -> Option<Radiation>;
}

impl Light for BackgroundLight {
    fn radiate(&self, _out_ray: &Ray) -> Option<Radiation> {
        Some(Radiation {
            radiance: self.intensity * self.scale
        })
    }
}

impl Light for PointLight {
    fn radiate(&self, _out_ray: &Ray) -> Option<Radiation> {
        Some(Radiation {
            radiance: self.intensity
        })
    }
}
