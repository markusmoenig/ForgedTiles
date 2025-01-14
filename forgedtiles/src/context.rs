pub use crate::bsdf::*;
pub use crate::camera::*;
use crate::prelude::*;
pub use crate::ray::Ray;
use rayon::prelude::*;

use crate::NodeRole::*;
use crate::NodeSubRole::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FTContext {
    pub nodes: Vec<Node>,
    pub shapes: Vec<u8>,
    pub patterns: Vec<u8>,
    pub faces: Vec<u8>,
    pub materials: Vec<u8>,

    pub variables: FxHashMap<String, usize>,

    pub output: Option<usize>,

    pub meta_delete: Vec<i32>,
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
            materials: vec![],

            variables: FxHashMap::default(),

            output: None,

            meta_delete: vec![],
        }
    }

    /// Get the distance to a face.
    pub fn distance_to_face(
        &self,
        p: Vec3f,
        face_index: usize,
        tile_id: Vec2f,
        use_bbox: bool,
    ) -> FTHitStruct {
        let mut hit = FTHitStruct {
            tile_id,
            ..Default::default()
        };

        if
        /*face_index >= self.faces.len() - 1 ||*/
        self.faces.is_empty() {
            hit.distance = f32::MAX;
            return hit;
        }

        let face_index = self.faces[face_index];
        let indices = &self.nodes[face_index as usize].links;

        let face_length = self.nodes[face_index as usize]
            .values
            .get(FTValueRole::Length, vec![1.0])[0];

        let face_height = self.nodes[face_index as usize]
            .values
            .get(FTValueRole::Height, vec![1.0])[0];

        let mut face_thickness = self.nodes[face_index as usize]
            .values
            .get(FTValueRole::Thickness, vec![0.1])[0];

        fn op_extrusion_x(p: Vec3f, d: f32, h: f32) -> f32 {
            let w = Vec2f::new(d, abs(p.x) - h);
            min(max(w.x, w.y), 0.0) + length(max(w, Vec2f::zero()))
        }

        fn op_extrusion_y(p: Vec3f, d: f32, h: f32) -> f32 {
            let w = Vec2f::new(d, abs(p.y) - h);
            min(max(w.x, w.y), 0.0) + length(max(w, Vec2f::zero()))
        }

        fn op_extrusion_z(p: Vec3f, d: f32, h: f32) -> f32 {
            let w = Vec2f::new(d, abs(p.z) - h);
            min(max(w.x, w.y), 0.0) + length(max(w, Vec2f::zero()))
        }

        // Get the 2D positiion and the 2D offset based on the wall type.
        let half_length = face_length / 2.0;
        let (p2d, pos) = match &self.nodes[face_index as usize].sub_role {
            NodeSubRole::MiddleX | NodeSubRole::Back | NodeSubRole::Front => {
                hit.face = vec3f(face_length, face_height, face_thickness);
                (
                    vec2f(p.x, p.y),
                    vec2f(tile_id.x + half_length, face_height / 2.0),
                )
            }
            NodeSubRole::MiddleY | NodeSubRole::Left | NodeSubRole::Right => {
                hit.face = vec3f(face_length, face_height, face_thickness);
                hit.tile_id = vec2f(hit.tile_id.y, hit.tile_id.x);
                (
                    vec2f(p.z, p.y),
                    vec2f(tile_id.y + half_length, face_height / 2.0),
                )
            }
            NodeSubRole::Floor => {
                hit.face = vec3f(face_length, face_height, face_thickness);
                (
                    vec2f(p.x, face_height - p.z),
                    vec2f(tile_id.x + half_length, face_height / 2.0),
                )
            }
            _ => (Vec2f::zero(), Vec2f::zero()),
        };

        let bbox_dist = crate::sdf::sdf_box2d(
            p2d,
            vec2f(pos.x, hit.face.y / 2.0),
            face_length / 2.0,
            hit.face.y / 2.0,
            0.0,
        );

        if !use_bbox {
            for index in indices {
                if self.nodes[*index as usize].role == NodeRole::Pattern {
                    self.distance(*index as usize, p2d, Vec2f::zero(), &mut hit)
                } else {
                    self.distance(*index as usize, p2d, pos, &mut hit)
                };
            }
        }

        // Clip 2D output to the face
        let dist_2d_min = if use_bbox {
            bbox_dist
        } else {
            max(hit.min_distance, bbox_dist)
        };

        if let Some(node) = hit.node {
            face_thickness = self.nodes[node]
                .expressions
                .eval(
                    FTExpressionRole::Extrusion,
                    vec![
                        (FTExpressionParam::Hash, hit.pattern_hash),
                        (FTExpressionParam::Thickness, face_thickness),
                    ],
                    face_thickness,
                )
                .clamp(0.0, 10.0);
        }

        face_thickness /= 2.0;

        // Extrude in the direction of the face
        hit.distance = match &self.nodes[face_index as usize].sub_role {
            NodeSubRole::Left => op_extrusion_x(
                p - vec3f(tile_id.x + face_thickness / 2.0, 0.0, 0.0),
                dist_2d_min,
                face_thickness,
            ),
            NodeSubRole::MiddleY => op_extrusion_x(
                p - vec3f(tile_id.x + 0.5, 0.0, 0.0),
                dist_2d_min,
                face_thickness,
            ),
            NodeSubRole::Right => op_extrusion_x(
                p - vec3f(tile_id.x + 1.0 - face_thickness / 2.0, 0.0, 0.0),
                dist_2d_min,
                face_thickness,
            ),
            NodeSubRole::Back => op_extrusion_z(
                p - vec3f(0.0, 0.0, tile_id.y + face_thickness / 2.0),
                dist_2d_min,
                face_thickness,
            ),
            NodeSubRole::MiddleX => op_extrusion_z(
                p - vec3f(0.0, 0.0, tile_id.y + 0.5),
                dist_2d_min,
                face_thickness,
            ),
            NodeSubRole::Front => op_extrusion_z(
                p - vec3f(0.0, 0.0, tile_id.y + 1.0 - face_thickness / 2.0),
                dist_2d_min,
                face_thickness,
            ),
            NodeSubRole::Floor => {
                let face_offset = self.nodes[face_index as usize]
                    .values
                    .get(FTValueRole::Offset, vec![0.0])[0];
                max(
                    op_extrusion_y(
                        p - vec3f(0.0, tile_id.y + face_thickness / 2.0 + face_offset, 0.0),
                        dist_2d_min,
                        face_thickness,
                    ),
                    -p.y,
                )
            }
            _ => f32::MAX,
        };
        hit
    }

    /// Get the face normal at the given position.
    pub fn face_normal(&self, p: Vec3f, face_index: usize, tile_id: Vec2f) -> Vec3f {
        let scale = 0.5773 * 0.0005;
        let e = vec2f(1.0 * scale, -1.0 * scale);

        // IQs normal function

        let e1 = vec3f(e.x, e.y, e.y);
        let e2 = vec3f(e.y, e.y, e.x);
        let e3 = vec3f(e.y, e.x, e.y);
        let e4 = vec3f(e.x, e.x, e.x);

        let n = e1
            * self
                .distance_to_face(p + e1, face_index, tile_id, false)
                .distance
            + e2 * self
                .distance_to_face(p + e2, face_index, tile_id, false)
                .distance
            + e3 * self
                .distance_to_face(p + e3, face_index, tile_id, false)
                .distance
            + e4 * self
                .distance_to_face(p + e4, face_index, tile_id, false)
                .distance;
        normalize(n)
    }

    /// Get the meta data at the given position of a face.
    pub fn meta_data_at(&self, x: i32, y: i32, width: usize, height: usize) -> Option<FTHitStruct> {
        if self.nodes.is_empty() {
            return None;
        }
        let output = self.output.unwrap_or(self.nodes.len() - 1);
        let indices = &self.nodes[output].links;

        let w = width as f32;
        let h = height as f32;

        //let length = self.get_value_default(output, FTValueRole::Length, vec![1.0])[0];
        let height = self.get_value_default(output, FTValueRole::Height, vec![1.0])[0];
        let p = vec2f(x as f32 / w, height - y as f32 / h);

        let mut hit = FTHitStruct {
            face: vec3f(100.0, 100.0, 100.0),
            ..Default::default()
        };

        //let mut dist = FTHitStruct::default();
        //let mut hit_index: Option<usize> = None;
        for index in indices {
            self.distance(*index as usize, p, Vec2f::zero(), &mut hit);
            if hit.distance < 0.0 {
                //&& hit.distance < dist.distance {
                //dist.clone_from(&hit);
                //hit_index = Some(*index as usize);
                break;
            }
        }

        if hit.distance <= 0.0 {
            Some(hit)
        } else {
            None
        }
    }

    /// Computes the pixel at the given position.
    pub fn face_pixel_at(&self, mut p: Vec2f) -> Option<[u8; 4]> {
        if self.nodes.is_empty() {
            return None;
        }

        let output = self.output.unwrap_or(self.nodes.len() - 1);
        let indices = if self.nodes[output].role != Face {
            vec![output as i32]
        } else {
            self.nodes[output].links.clone()
        };

        let face: Vec3f = if self.nodes[output].role == NodeRole::Face {
            let face_length = self.nodes[output]
                .values
                .get(FTValueRole::Length, vec![1.0])[0];

            let face_height = self.nodes[output]
                .values
                .get(FTValueRole::Height, vec![1.0])[0];

            let face_thickness = self.nodes[output]
                .values
                .get(FTValueRole::Thickness, vec![0.2])[0];

            vec3f(face_length, face_height, face_thickness)
        } else {
            vec3f(1.0, 1.0, 1.0)
        };

        let mut color = Vec3f::zero();

        let mut hit = FTHitStruct {
            face,
            ..Default::default()
        };

        // To hide ugly diffs in 2D mode
        hit.shape_adder = 0.025;

        // Scale down to one tile unit
        p.y *= face.y;

        for index in &indices {
            let mut pos = vec2f(0.0, 0.0);
            if self.nodes[*index as usize].role == NodeRole::Shape {
                pos = vec2f(0.5, 0.5);
            }
            self.distance(*index as usize, p, pos, &mut hit);
        }

        if hit.is_cut_out {
            return None;
        }

        if hit.distance <= 0.0 {
            if let Some(node) = hit.node {
                if let Some(material) = self.nodes[node].material {
                    let spec_trans = self.nodes[material as usize].expressions.eval(
                        FTExpressionRole::Transmission,
                        vec![(FTExpressionParam::Hash, hit.pattern_hash)],
                        0.0,
                    );

                    // Return transparent for transmissive materials
                    if spec_trans > 0.5 {
                        return None;
                    }

                    let col = self.nodes[material as usize]
                        .values
                        .get(FTValueRole::Color, vec![0.5, 0.5, 0.5]);
                    color[0] = col[0];
                    color[1] = col[1];
                    color[2] = col[2];

                    color[0] = col[0] + ((hit.pattern_hash) - 0.5) * 0.5;
                    color[1] = col[1] + ((hit.pattern_hash) - 0.5) * 0.5;
                    color[2] = col[2] + ((hit.pattern_hash) - 0.5) * 0.5;

                    return Some([
                        (color.x * 255.0) as u8,
                        (color.y * 255.0) as u8,
                        (color.z * 255.0) as u8,
                        255,
                    ]);
                } else {
                    return Some([127, 127, 127, 255]);
                }
            } else {
                return Some([127, 127, 127, 255]);
            }
        }

        Some([0, 0, 0, 255])
    }

    /// Render the output node into as 2D
    pub fn render(&self, width: usize, height: usize, buffer: &mut [u8]) {
        let w = width as f32;
        let h = height as f32;

        if self.nodes.is_empty() {
            return;
        }
        let output = self.output.unwrap_or(self.nodes.len() - 1);
        let indices = if self.nodes[output].role != Face {
            vec![output as i32]
        } else {
            self.nodes[output].links.clone()
        };

        let face: Vec3f = if self.nodes[output].role == NodeRole::Face {
            let face_length = self.nodes[output]
                .values
                .get(FTValueRole::Length, vec![1.0])[0];

            let face_height = self.nodes[output]
                .values
                .get(FTValueRole::Height, vec![1.0])[0];

            let face_thickness = self.nodes[output]
                .values
                .get(FTValueRole::Thickness, vec![0.2])[0];

            vec3f(face_length, face_height, face_thickness)
        } else {
            vec3f(1.0, 1.0, 1.0)
        };

        buffer
            .par_rchunks_exact_mut(width * 4)
            .enumerate()
            .for_each(|(j, line)| {
                for (i, pixel) in line.chunks_exact_mut(4).enumerate() {
                    let i = j * width + i;
                    let x = (i % width) as f32;
                    let y = (i / width) as f32;

                    let xx = x / w;
                    let yy = y / h;

                    let mut p = vec2f(xx, yy);
                    p.x *= face.x;
                    p.y *= face.y;

                    let mut color = vec3f(0.0, 0.0, 0.0);

                    //let p = (2.0 * vec2f(x, y) - vec2f(w, h)) / h;

                    let mut hit = FTHitStruct {
                        face,
                        ..Default::default()
                    };

                    //let mut hit_index: Option<usize> = None;
                    for index in &indices {
                        // let mut local_hit = FTHitStruct {
                        //     face,
                        //     ..Default::default()
                        // };

                        let pos = vec2f(0.0, 0.0);
                        // if self.nodes[*index as usize].role == NodeRole::Shape {
                        //     pos = vec2f(0.5, 0.5);
                        // }
                        _ = self.distance(*index as usize, p, pos, &mut hit);
                        //if distance < 0.0 {
                        //&& hit.distance < dist.distance {
                        //hit.clone_from(&local_hit);
                        //hit_index = Some(*index as usize);
                        //}
                    }

                    if hit.distance < 0.0 {
                        let material = BSDFMaterial::from_hit(self, &hit);

                        color[0] = material.base_color[0];
                        color[1] = material.base_color[1];
                        color[2] = material.base_color[2];
                        //
                        // if let Some(node) = hit.node {
                        //     if let Some(material) = self.nodes[node].material {
                        //         let col = self.nodes[material as usize]
                        //             .values
                        //             .get(FTValueRole::Color, vec![0.5, 0.5, 0.5]);
                        //         color[0] = col[0];
                        //         color[1] = col[1];
                        //         color[2] = col[2];

                        //         color[0] = col[0] + ((hit.pattern_hash) - 0.5) * 0.5;
                        //         color[1] = col[1] + ((hit.pattern_hash) - 0.5) * 0.5;
                        //         color[2] = col[2] + ((hit.pattern_hash) - 0.5) * 0.5;
                        //     } else {
                        //         color = vec3f(0.5, 0.5, 0.5);
                        //     }
                        // } else {
                        //     color = vec3f(0.5, 0.5, 0.5);
                        // }
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
        samples: i32,
    ) {
        let w = width as f32;
        let h = height as f32;

        if self.nodes.is_empty() {
            return;
        }

        let camera = Camera::new(vec3f(4., 4., 4.), vec3f(0.0, 0.0, 0.0), 5.0);

        if self.faces.is_empty() {
            println!("render_bsdf_sample:: Need a face to render.");
            return;
        }

        const EPS: f32 = 0.001;

        buffer
            .par_rchunks_exact_mut(width * 4)
            .enumerate()
            .for_each(|(j, line)| {
                for (i, pixel) in line.chunks_exact_mut(4).enumerate() {
                    let mut rng = rand::thread_rng();

                    let i = j * width + i;
                    let x = (i % width) as f32;
                    let y = (i / width) as f32;

                    let xx = x / w;
                    let yy = y / h;

                    for sample in 0..samples {
                        let mut ray = camera.create_ortho_ray(
                            vec2f(xx, yy),
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
                            let mut t = 0.0;

                            for _ in 0..30 {
                                let p = ray.at(t);

                                let mut ft_hit = self.distance_to_face(p, 0, Vec2f::zero(), false);
                                for face in 1..self.faces.len() {
                                    let ft = self.distance_to_face(p, face, Vec2f::zero(), false);
                                    if ft.distance < ft_hit.distance {
                                        ft_hit.clone_from(&ft);
                                    }
                                }

                                if t > 12.0 {
                                    break;
                                }
                                //println!("aa {}", ft_hit.distance);

                                if ft_hit.distance < 0.0001 {
                                    has_hit = true;
                                    hit_point = p;
                                    hit_distance = t;
                                    hit_normal = self.face_normal(p, 0, Vec2f::zero());
                                    state.mat = BSDFMaterial::from_hit(self, &ft_hit);
                                    break;
                                }
                                t += ft_hit.distance;
                            }

                            if has_hit {
                                state.depth = depth;

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

                                let light_pos = vec3f(1.0, 2.0, 3.0);

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
                                        ld += (mis_weight * li * scatter_sample.f
                                            / light_sample.pdf)
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
                                ray.o = state.fhp + ray.d * 0.001;

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

                        fn vec4f_u8(col: Vec3f) -> [u8; 4] {
                            [
                                (col.x * 255.0) as u8,
                                (col.y * 255.0) as u8,
                                (col.z * 255.0) as u8,
                                255,
                            ]
                        }

                        if sample == 0 {
                            pixel.copy_from_slice(&vec4f_u8(color));
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
                            pixel.copy_from_slice(&vec4f_u8(accumulated_color));
                        }
                    }
                }
            });
    }

    /// Returns the distance and other meta data for the given node.
    pub fn distance(&self, index: usize, p: Vec2f, mut pos: Vec2f, hit: &mut FTHitStruct) -> f32 {
        let mut distance = f32::MAX;

        /// Adjust the distances for shapes
        #[inline(always)]
        fn adjust_distances(index: usize, distance: f32, hit: &mut FTHitStruct) {
            if distance <= 0.0 {
                hit.distance = distance;
                hit.is_cut_out = false;
            }
            if distance < hit.min_distance {
                hit.min_distance = distance;
                hit.node = Some(index);

                hit.seed = hit.working_seed;
                hit.seed_id = hit.working_seed_id;

                hit.pattern_hash = hit.working_pattern_hash;
                hit.pattern_id = hit.working_pattern_id;
            }

            if hit.node.is_none() {
                hit.node = Some(index);

                hit.seed = hit.working_seed;
                hit.seed_id = hit.working_seed_id;

                hit.pattern_hash = hit.working_pattern_hash;
                hit.pattern_id = hit.working_pattern_id;
            }
        }

        // #[allow(clippy::single_match)]
        match &self.nodes[index].role {
            Shape => match &self.nodes[index].sub_role {
                Disc => {
                    if self.meta_delete.contains(&hit.working_seed_id)
                        || self.meta_delete.contains(&hit.working_pattern_id)
                    {
                        return hit.distance;
                    }
                    let radius = self.get_value_default(index, FTValueRole::Radius, vec![0.5])[0];
                    distance = length(p - pos) - radius;
                    adjust_distances(index, distance, hit);
                }
                Box => {
                    if self.meta_delete.contains(&hit.working_seed_id)
                        || self.meta_delete.contains(&hit.working_pattern_id)
                    {
                        hit.is_cut_out = true;
                        return hit.distance;
                    }
                    let mut length =
                        self.get_value_default(index, FTValueRole::Length, vec![1.0])[0];
                    let mut height =
                        self.get_value_default(index, FTValueRole::Height, vec![1.0])[0];

                    length += hit.shape_adder;
                    height += hit.shape_adder;
                    // let rounding =
                    //     self.get_value_default(index, FTValueRole::Rounding, vec![0.0])[0];
                    let rounding = self.nodes[index].expressions.eval(
                        FTExpressionRole::Rounding,
                        vec![(FTExpressionParam::Hash, hit.working_pattern_hash)],
                        0.0,
                    );
                    let hole = self.nodes[index].expressions.eval(
                        FTExpressionRole::Annular,
                        vec![(FTExpressionParam::Hash, hit.working_pattern_hash)],
                        0.0,
                    );
                    let rotation = self.nodes[index].expressions.eval(
                        FTExpressionRole::Rotation,
                        vec![(FTExpressionParam::Hash, hit.working_pattern_hash)],
                        0.0,
                    );
                    distance = crate::sdf::sdf_box2d(
                        crate::sdf::rot((hit.working_pattern_hash - 0.5) * rotation) * p,
                        pos,
                        length / 2.0 - hole,
                        height / 2.0 - hole,
                        rounding,
                    );

                    if hole > 0.0 {
                        distance = distance.abs() - hole;
                    }

                    hit.last_size = self.get_dim_default(index);
                    adjust_distances(index, distance, hit);
                }
                _ => {}
            },
            Pattern => match &self.nodes[index].sub_role {
                Repeat => {
                    if !self.nodes[index].links.is_empty() {
                        fn op_rep(p: Vec2f, s: f32) -> Vec2f {
                            vec2f(p.x - s * round(p.x / s), p.y)
                        }
                        let content = self.nodes[index].links[0] as usize;
                        let dim = self.get_dim_default(content);
                        let spacing =
                            self.get_value_default(index, FTValueRole::Spacing, vec![0.0])[0];
                        let offset =
                            self.get_value_default(index, FTValueRole::Offset, vec![0.0])[0];

                        hit.last_size = dim;
                        pos += vec2f(0.0, dim.y / 2.0);

                        let r = op_rep(
                            p - vec2f(dim.x / 2.0 - offset * dim.x, 0.0),
                            dim.x + spacing,
                        );

                        let u = (p
                            - vec2f(offset * dim.x, pos.y - dim.y / 2.0)
                            - vec2f(hit.tile_id.x, 0.0))
                            / vec2f(dim.x + spacing, dim.y + pos.y);
                        hit.working_pattern_hash = crate::sdf::hash21(floor(u) + hit.working_seed);
                        hit.working_pattern_id =
                            ((hit.working_pattern_hash * 10000.0).floor() as i32) % 10000;

                        distance = self.distance(content, r, pos, hit);
                    }
                }
                Stack => {
                    if !self.nodes[index].links.is_empty() {
                        let spacing =
                            self.get_value_default(index, FTValueRole::Spacing, vec![0.0])[0];

                        pos = hit.origin;
                        let mut counter = 0;
                        // let mut rng = rand::thread_rng();
                        // hit.seed = rng.gen();

                        let content = self.nodes[index].links[0] as usize;

                        let mut top_end = hit.face.y;
                        if self.nodes[content].role == Shape {
                            let dim = self.get_dim_default(content);
                            pos += vec2f(dim.x / 2.0, dim.y / 2.0);
                            top_end += dim.y / 2.0;
                        }

                        loop {
                            hit.working_seed = crate::sdf::hash21(pos);
                            hit.working_seed_id =
                                ((hit.working_seed * 10000.0).floor() as i32) % 10000;
                            let content = self.nodes[index].links
                                [counter % self.nodes[index].links.len()]
                                as usize;

                            distance = self.distance(content, p, pos, hit);
                            pos.y += hit.last_size.y + spacing;

                            //println!("{} {}", pos.y + hit.last_size.y, top_end);
                            if pos.y + hit.last_size.y > top_end {
                                break;
                            }

                            counter += 1;
                        }
                    }
                }
                Group => {
                    let mut group_pos = Vec2f::zero();
                    group_pos.x =
                        hit.tile_id.x + self.get_value_default(index, FTValueRole::X, vec![0.0])[0];
                    group_pos.y = self.get_value_default(index, FTValueRole::Y, vec![0.0])[0];
                    let old_origin = hit.origin;
                    hit.origin = group_pos;

                    if let Some(cut_out_indices) = self.get_value(index, FTValueRole::Cutout) {
                        let cut_out_index = cut_out_indices[0] as usize;

                        let cut_out_dim = self.get_dim_default(cut_out_index);
                        let mut cut_out_hit = FTHitStruct::default();
                        let cut_out_distance = self.distance(
                            cut_out_index,
                            p,
                            group_pos + vec2f(cut_out_dim.x / 2.0, cut_out_dim.y / 2.0),
                            &mut cut_out_hit,
                        );

                        distance = max(hit.distance, -cut_out_distance);
                        hit.distance = distance;

                        if cut_out_distance < 0.0 {
                            hit.group_uv = vec2f(
                                (p.x - group_pos.x) / cut_out_dim.x,
                                1.0 - (p.y - group_pos.y) / cut_out_dim.y,
                            );
                            hit.min_distance = 0.001;
                            hit.node = None;
                            hit.pattern_hash = 0.0;
                            hit.pattern_id = 0;
                            hit.seed = 0.0;
                            hit.seed_id = 0;
                            hit.is_cut_out = true;
                        } else {
                            hit.is_cut_out = false;
                        }
                    }

                    for content in &self.nodes[index].links {
                        let index = *content as usize;
                        let old_group_pos = group_pos;
                        if self.nodes[index].role == Shape {
                            let dim = self.get_dim_default(index);
                            group_pos.x += dim.x / 2.0;
                            group_pos.y += dim.y / 2.0;
                        }
                        distance = self.distance(*content as usize, p, group_pos, hit);
                        group_pos = old_group_pos;
                    }

                    hit.origin = old_origin;
                }
                _ => {}
            },
            _ => {}
        }

        distance
    }

    /// Get a value from a node.
    fn get_value(&self, index: usize, role: FTValueRole) -> Option<Vec<f32>> {
        self.nodes[index].values.get_option(role)
    }

    /// Get a value from a node.
    fn get_value_default(&self, index: usize, role: FTValueRole, default: Vec<f32>) -> Vec<f32> {
        self.nodes[index].values.get(role, default)
    }

    /// Get the dimension of a node.
    fn get_dim_default(&self, index: usize) -> Vec2f {
        self.nodes[index].get_shape_dim()
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
