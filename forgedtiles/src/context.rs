use core::f32;

pub use crate::bsdf::*;
pub use crate::camera::*;
use crate::prelude::*;
pub use crate::ray::Ray;
use rayon::prelude::*;

#[derive(Clone, Debug)]
pub struct FTContext {
    pub nodes: Vec<Node>,
    pub shapes: Vec<usize>,
    pub patterns: Vec<usize>,
    pub faces: Vec<usize>,

    pub variables: FxHashMap<String, usize>,

    pub output: Option<usize>,
}

impl Default for FTContext {
    fn default() -> Self {
        Self::new()
    }
}

impl FTContext {
    pub fn new() -> Self {
        Self {
            nodes: vec![],
            shapes: vec![],
            patterns: vec![],
            faces: vec![],

            variables: FxHashMap::default(),

            output: None,
        }
    }

    pub fn distance_to_face(&self, p: Vec3f, face_index: usize, tile_id: Vec2f) -> f32 {
        let face_index = self.faces[face_index];
        let indices = &self.nodes[face_index].links;

        fn op_extrusion_x(p: Vec3f, d: f32, h: f32) -> f32 {
            let w = Vec2f::new(d, abs(p.x) - h);
            min(max(w.x, w.y), 0.0) + length(max(w, Vec2f::zero()))
        }

        fn op_extrusion_z(p: Vec3f, d: f32, h: f32) -> f32 {
            let w = Vec2f::new(d, abs(p.z) - h);
            min(max(w.x, w.y), 0.0) + length(max(w, Vec2f::zero()))
        }

        //let mut dist = f32::MAX;
        let mut dist_2d_min = f32::MAX;

        for index in indices {
            let half_length = self.nodes[face_index]
                .values
                .get(FTValueRole::Length, vec![1.0])[0]
                / 2.0;

            let (p, pos) = match &self.nodes[face_index].sub_role {
                NodeSubRole::MiddleX => (vec2f(p.x, p.y), vec2f(tile_id.x + half_length, 0.5)),
                _ => (Vec2f::zero(), Vec2f::zero()),
            };
            let d = self.nodes[*index as usize].distance(p, pos);
            dist_2d_min = min(dist_2d_min, d.0);
        }

        op_extrusion_z(p - vec3f(0.0, 0.0, tile_id.y + 0.5), dist_2d_min, 0.2)
    }

    pub fn meta_data_at(
        &self,
        x: i32,
        y: i32,
        width: usize,
        height: usize,
    ) -> Option<(f32, usize, usize)> {
        if self.nodes.is_empty() {
            return None;
        }
        let output = self.output.unwrap_or(self.nodes.len() - 1);
        let indices = &self.nodes[output].links;

        let w = width as f32;
        let h = height as f32;

        let p = (2.0 * vec2f(x as f32, h - y as f32) - vec2f(w, h)) / h;

        let mut dist = (f32::MAX, -1);
        let mut hit_index: Option<usize> = None;
        for index in indices {
            let d = self.nodes[*index as usize].distance(p, Vec2f::zero());
            if d.0 < 0.0 && d.0 < dist.0 {
                dist = d;
                hit_index = Some(*index as usize);
            }
        }

        if let Some(hit_index) = hit_index {
            Some((dist.0, hit_index, dist.1 as usize))
        } else {
            None
        }
    }

    pub fn render(&self, width: usize, height: usize, buffer: &mut [u8]) {
        let w = width as f32;
        let h = height as f32;

        if self.nodes.is_empty() {
            return;
        }
        let output = self.output.unwrap_or(self.nodes.len() - 1);
        let indices = &self.nodes[output].links;

        let wd = self.nodes[output]
            .values
            .get(FTValueRole::Length, vec![1.0])[0];
        let hd = self.nodes[output]
            .values
            .get(FTValueRole::Height, vec![1.0])[0];

        let pc_x = w / wd;
        let pc_y = h / hd;
        println!("wd {}", wd);

        buffer
            .par_rchunks_exact_mut(width * 4)
            .enumerate()
            .for_each(|(j, line)| {
                for (i, pixel) in line.chunks_exact_mut(4).enumerate() {
                    let i = j * width + i;
                    let x = (i % width) as f32;
                    let y = (i / width) as f32;

                    // let xx = x / w;
                    // let yy = y / h;

                    // let p = vec2f(xx, yy);
                    let mut color = vec3f(0.0, 0.0, 0.0);

                    let mut p = (2.0 * vec2f(x, y) - vec2f(w, h)) / h;

                    let mut dist = f32::MAX;
                    let mut hit_index: Option<usize> = None;
                    for index in indices {
                        let d = self.nodes[*index as usize].distance(p, Vec2f::zero()).0;
                        if d < 0.0 && d < dist {
                            dist = d;
                            hit_index = Some(*index as usize);
                            color.x = 1.0;
                        }
                    }

                    let out = [
                        (color.x * 255.0) as u8,
                        (color.y * 255.0) as u8,
                        (color.z * 255.0) as u8,
                        255,
                    ];

                    pixel.copy_from_slice(&out);
                }
            });
    }

    pub fn render_bsdf_sample(
        &self,
        width: usize,
        height: usize,
        buffer: &mut Vec<u8>,
        sample: i32,
    ) {
        let w = width as f32;
        let h = height as f32;

        if self.nodes.is_empty() {
            return;
        }

        let camera = Camera::new(vec3f(0., 0., 2.), Vec3f::zero(), 5.0);

        let output = self.output.unwrap_or(self.nodes.len() - 1);
        let indices = self.nodes[output].content_indices(output);

        const EPS: f32 = 0.001;

        buffer
            .par_rchunks_exact_mut(width * 3)
            .enumerate()
            .for_each(|(j, line)| {
                let mut rng = rand::thread_rng();
                for (i, pixel) in line.chunks_exact_mut(3).enumerate() {
                    let i = j * width + i;
                    let x = (i % width) as f32;
                    let y = (i / width) as f32;

                    let xx = x / w;
                    let yy = y / h;

                    let mut ray = camera.create_ortho_ray(
                        vec2f(xx, 1.0 - yy),
                        vec2f(w, h),
                        vec2f(rng.gen(), rng.gen()),
                    );

                    let mut radiance = Vec3f::zero();
                    let mut throughput = Vec3f::one();

                    let mut state = BSDFState::default();
                    //let mut light_sample = BSDFLightSampleRec::default();
                    //let mut scatter_sample = BSDFScatterSampleRec::default();

                    // For medium tracking
                    let mut _in_medium = false;
                    let mut _medium_sampled = false;
                    let mut _surface_scatter = false;

                    let mut color = Vec3f::zero();

                    for depth in 0..8 {
                        let mut has_hit = false;
                        let mut hit_point = Vec3f::zero();
                        let mut hit_distance = 0.0;
                        let mut hit_normal = Vec3f::zero();
                        let mut mat = BSDFMaterial::default();
                        let mut t = 0.0;

                        for _ in 0..20 {
                            let p = ray.at(t);

                            //let d = length(p) - 1.0;

                            // if t > 3.0 {
                            //     break;
                            // }

                            fn op_extrusion_x(p: Vec3f, d: f32, h: f32) -> f32 {
                                let w = Vec2f::new(d, abs(p.x) - h);
                                min(max(w.x, w.y), 0.0) + length(max(w, Vec2f::zero()))
                            }

                            fn op_extrusion_z(p: Vec3f, d: f32, h: f32) -> f32 {
                                let w = Vec2f::new(d, abs(p.z) - h);
                                min(max(w.x, w.y), 0.0) + length(max(w, Vec2f::zero()))
                            }

                            //let mut dist = f32::MAX;
                            let mut dist_2d_min = f32::MAX;

                            for index in &indices {
                                let d = self.nodes[*index].distance(vec2f(p.x, p.y), Vec2f::zero());
                                dist_2d_min = min(dist_2d_min, d.0);
                                // if d < 0.0 && d < dist_2d_min {
                                //     dist_2d_min = d;
                                // }
                            }

                            let dist = op_extrusion_z(p, dist_2d_min, 0.2);

                            if dist.abs() < 0.001 {
                                has_hit = true;
                                hit_point = p;
                                hit_distance = t;
                                hit_normal = normalize(p);
                                mat.base_color = Vec3f::one();
                                break;
                            }
                            t += dist;
                        }

                        if has_hit {
                            state.depth = depth;

                            state.mat.clone_from(&mat);
                            state.mat.roughness = max(state.mat.roughness, 0.001);
                            // Remapping from clearcoat gloss to roughness
                            state.mat.clearcoat_roughness =
                                lerp(0.1, 0.001, state.mat.clearcoat_roughness);

                            state.hit_dist = hit_distance;
                            state.fhp = hit_point;

                            state.normal = hit_normal;
                            state.ffnormal = if dot(state.normal, ray.d) <= 0.0 {
                                state.normal
                            } else {
                                -state.normal
                            };

                            state.eta = if dot(ray.d, state.normal) < 0.0 {
                                1.0 / state.mat.ior
                            } else {
                                state.mat.ior
                            };

                            onb(state.normal, &mut state.tangent, &mut state.bitangent);

                            let aspect = sqrt(1.0 - state.mat.anisotropic * 0.9);
                            state.mat.ax = max(0.001, state.mat.roughness / aspect);
                            state.mat.ay = max(0.001, state.mat.roughness * aspect);

                            // --- Sample light
                            //
                            let mut light_sample = BSDFLightSampleRec::default();
                            let mut scatter_sample = BSDFScatterSampleRec::default();

                            let scatter_pos = state.fhp + state.normal * EPS;

                            let light_pos = vec3f(1.0, 0.0, 3.0);

                            let radius = 0.2;

                            let l = BSDFLight {
                                position: light_pos,
                                emission: Vec3f::one() * 10.0,
                                radius,
                                type_: 1.0,
                                u: Vec3f::zero(),
                                v: Vec3f::zero(),
                                area: 4.0 * f32::pi() * radius * radius,
                            };

                            sample_sphere_light(
                                &l,
                                scatter_pos,
                                &mut light_sample,
                                1,
                                &mut rng,
                                5.0,
                            );

                            let li = light_sample.emission;

                            let sphere_ray = Ray::new(scatter_pos, light_sample.direction);

                            if sphere_ray.sphere(light_pos, radius).is_some() {
                                scatter_sample.f = disney_eval(
                                    &state,
                                    -ray.d,
                                    state.ffnormal,
                                    light_sample.direction,
                                    &mut scatter_sample.pdf,
                                );

                                let mut mis_weight = 1.0;
                                if l.area > 0.0 {
                                    // No MIS for distant light
                                    mis_weight =
                                        power_heuristic(light_sample.pdf, scatter_sample.pdf);
                                }

                                let mut ld = Vec3f::zero();

                                if scatter_sample.pdf > 0.0 {
                                    ld += (mis_weight * li * scatter_sample.f / light_sample.pdf)
                                        * throughput;
                                }

                                radiance += ld * throughput;
                            }
                            //

                            scatter_sample.f = disney_sample(
                                &state,
                                -ray.d,
                                state.ffnormal,
                                &mut scatter_sample.l,
                                &mut scatter_sample.pdf,
                                &mut rng,
                            );
                            if scatter_sample.pdf > 0.0 {
                                throughput *= scatter_sample.f / scatter_sample.pdf;
                            } else {
                                break;
                            }

                            ray.d = scatter_sample.l;
                            ray.o = state.fhp + ray.d * 0.01;

                            color.x = radiance.x;
                            color.y = radiance.y;
                            color.z = radiance.z;
                        } else {
                            // Env color
                            color.x += throughput.x * 0.5;
                            color.y += throughput.y * 0.5;
                            color.z += throughput.z * 0.5;
                            break;
                        }
                    }

                    fn vec3f_u8(col: Vec3f) -> [u8; 3] {
                        [
                            (col.x * 255.0) as u8,
                            (col.y * 255.0) as u8,
                            (col.z * 255.0) as u8,
                        ]
                    }

                    if sample == 0 {
                        pixel.copy_from_slice(&vec3f_u8(color));
                    } else {
                        let mut ex = Vec3f::zero();
                        ex.x = pixel[0] as f32 / 255.0;
                        ex.y = pixel[1] as f32 / 255.0;
                        ex.z = pixel[2] as f32 / 255.0;

                        //color = powf(color, 0.4545);
                        //color = clamp(color, Vec4f::zero(), vec4f(1.0, 1.0, 1.0, 1.0));

                        let s = 1.0 / (sample as f32 + 1.0);
                        let accumulated_color = lerp(ex, color, s);
                        // let accumulated_color =
                        //     (ex * (sample as f32) + color) / (sample as f32 + 1.0);
                        //
                        pixel.copy_from_slice(&vec3f_u8(accumulated_color));
                    }
                }
            });
    }
}

/*
float fillMask(float dist)
{
    return clamp(-dist, 0.0, 1.0);
}

float innerBorderMask(float dist, float width)
{
    //dist += 1.0;
    return clamp(dist + width, 0.0, 1.0) - clamp(dist, 0.0, 1.0);
}

float outerBorderMask(float dist, float width)
{
    //dist += 1.0;
    return clamp(dist, 0.0, 1.0) - clamp(dist - width, 0.0, 1.0);
}
*/
