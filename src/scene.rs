#![allow(dead_code)]

use geometry::{GeometryManager, Ray, SurfaceIntersection, Geometry, BSphere, Surface};
use math::vector_traits::*;
use brdf::Material;
// use light::Light;

pub type MaterialID = i32;
pub type LightID = i32;

#[derive(Debug, Clone, Copy)]
pub enum SurfaceProperties {
    Material(MaterialID),
    Light(LightID),
}

#[derive(Debug, Clone)]
pub struct DefaultScene<T> where T: GeometryManager {
    geo: T,
    materials: Vec<Material>,
    // lights: Vec<LightID>
}

pub trait Scene {
    fn new() -> Self;
    fn nearest_intersection(&self, ray: &Ray) -> Option<SurfaceIntersection>;
    fn add_object<G>(&mut self, geo: G, material: Material) where G: Geometry + 'static;
    fn bounding_sphere(&self) -> BSphere;
}

impl<T> Scene for DefaultScene<T> where T: GeometryManager {
    fn new() -> DefaultScene<T> {
        DefaultScene {
            geo: T::new(),
            materials: Vec::new(),
            // lights: Vec::new()
        }
    }

    fn nearest_intersection(&self, ray: &Ray) -> Option<SurfaceIntersection> {
        self.geo.nearest_intersection(ray)
    }

    fn add_object<G>(&mut self, geo: G, material: Material)
        where G: Geometry + 'static {
        self.materials.push(material);
        let material_id = self.materials.len() as i32 - 1;
        self.geo.add_geometry(Surface {
            geometry: geo,
            properties: SurfaceProperties::Material(material_id)
        });
    }

    fn bounding_sphere(&self) -> BSphere {
        let aabb = self.geo.build_aabbox();
        let radius2 = (aabb.max - aabb.min).sqnorm();
        BSphere {
            center: (aabb.min + aabb.max) * 0.5,
            radius: radius2.sqrt(),
            inv_radius_sqr: 1.0 / radius2
        }
    }
}
