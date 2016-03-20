#![allow(dead_code)]
use math::{Vec3f, Vec2f, Zero, EPS_COSINE, EPS_PHONG};
use math::vector_traits::*;
use utility::{cos_hemisphere_sample_w, luminance, pow_cos_hemisphere_sample_w};
use std::f32::consts::{FRAC_1_PI, PI};
use geometry::{Frame, Ray};
use std::ops::Add;

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct Material {
    pub diffuse: Vec3f,
    pub specular: Vec3f,
    pub phong_exp: f32
}

#[derive(Debug, Clone)]
pub struct Brdf {
    material: Material,
    own_basis: Frame,
    wo_local: Vec3f, // "out" in physical meaning, in fact - incoming
    probs: Probabilities
}

pub struct BrdfSample {
    pub wi: Vec3f, // "in" in physical meaning, i.e. from light to eye
    pub cos_theta_in: f32,
    pub radiance: Vec3f,
    pub pdf: f32,
}

pub struct BrdfEval {
    pub radiance: Vec3f,
    pub pdf: f32,
}

#[derive(Debug, Clone)]
struct Probabilities {
    diffuse: f32,
    phong: f32,
    continuation: f32,
}

impl Brdf {
    pub fn new(out_dir_world: &Vec3f, hit_normal: &Vec3f, material: &Material) -> Option<Brdf> {
        let own_basis = Frame::from_z(hit_normal);
        let wo_local = own_basis.to_local(&-*out_dir_world);
        if wo_local.z < EPS_COSINE {
            None
        } else {
            Some(Brdf {
                material: *material,
                own_basis: own_basis,
                wo_local: wo_local,
                probs: Probabilities::new(material)
            })
        }
    }

    pub fn sample(&self, rnd: (f32, f32, f32)) -> Option<BrdfSample> {
        let sample_rnds = (rnd.1, rnd.2);
        if rnd.0 <= self.probs.diffuse {
            self.lambert_sample(sample_rnds)
        } else {
            self.phong_sample(sample_rnds)
        }
    }

    pub fn eval(&self, wi: &Vec3f) -> Option<BrdfEval> {
        let wi_local = self.own_basis.to_local(wi).normalize();
        if wi_local.z < EPS_COSINE {
            None
        } else {
            let lambert = self.lambert_eval(&wi_local);
            let phong = self.phong_eval(&wi_local);
            Some(BrdfEval {
                radiance: lambert.radiance * self.probs.diffuse + phong.radiance * self.probs.phong,
                pdf: lambert.pdf * self.probs.diffuse + phong.pdf * self.probs.phong
            })
        }
    }

    fn lambert_sample(&self, rnd: (f32, f32)) -> Option<BrdfSample> {
        let (wi_local, pdf) = cos_hemisphere_sample_w(rnd);
        let cos_theta_in = wi_local.z;
        if cos_theta_in < EPS_COSINE {
            None
        } else {
            let wi = self.own_basis.to_world(&wi_local);
            Some(BrdfSample {
                wi: wi,
                cos_theta_in: cos_theta_in,
                radiance: self.material.diffuse,
                pdf: pdf
            })
        }
    }

    fn phong_sample(&self, rnd: (f32, f32)) -> Option<BrdfSample> {
        // get dir around refl. dir, move it to normals basis and then move it to world coords
        let (wi_local_reflect, pdf) = pow_cos_hemisphere_sample_w(self.material.phong_exp, rnd);
        let reflect_dir = self.wo_local.reflect_local();
        let reflect_basis = Frame::from_z(&reflect_dir);
        let wi_local = reflect_basis.to_world(&wi_local_reflect);
        let wi = self.own_basis.to_world(&wi_local);
        let cos_theta = wi_local.z;
        if wi_local.z < EPS_COSINE {
            None
        } else {
            Some(BrdfSample {
                wi: wi,
                cos_theta_in: cos_theta,
                radiance: self.material.specular,
                pdf: pdf
            })
        }
    }

    fn lambert_eval(&self, wi_local: &Vec3f) -> BrdfEval {
        let cos_theta = wi_local.z.max(0.0);
        BrdfEval {
            radiance: self.material.diffuse * cos_theta * FRAC_1_PI,
            pdf: cos_theta * FRAC_1_PI,
        }
    }

    fn phong_eval(&self, wi_local: &Vec3f) -> BrdfEval {
        let refl_local = self.wo_local.reflect_local();
        let cos_theta = refl_local.dot(&wi_local).max(0.0).min(1.0);
        // let cos_theta = if cos_theta < EPS_PHONG { 0.0 } else { cos_theta };
        let n = self.material.phong_exp;
        let refl_brightness = cos_theta.powf(n) * (n + 1.0) * 0.5 * FRAC_1_PI;
        BrdfEval {
            radiance: self.material.specular * refl_brightness,
            pdf: refl_brightness
        }
    }
}

impl Material {
    pub fn new_identity() -> Material {
        Material {
            diffuse: Zero::zero(),
            specular: Zero::zero(),
            phong_exp: 0.0
        }
    }

    fn albedo_diffuse(&self) -> f32 {
        luminance(&self.diffuse)
    }

    fn albedo_specular(&self) -> f32 {
        luminance(&self.specular)
    }

    fn total_albedo(&self) -> f32 {
        self.albedo_specular() + self.albedo_diffuse()
    }
}

impl Probabilities {
    fn new(mat: &Material) -> Probabilities {
        let albedo_diffuse = mat.albedo_diffuse();
        let albedo_specular = mat.albedo_specular();
        let total_albedo = mat.total_albedo();
        if total_albedo < 1.0e-9 {
            Probabilities {
                diffuse: 0.0,
                phong: 0.0,
                continuation: 0.0
            }
        } else {
            Probabilities {
                diffuse: albedo_diffuse / total_albedo,
                phong: albedo_specular / total_albedo,
                continuation: total_albedo
            }
        }
    }
}
