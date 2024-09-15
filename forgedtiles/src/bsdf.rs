use crate::prelude::*;

pub struct BSDFLight {
    pub position: Vec3f,
    pub emission: Vec3f,
    pub u: Vec3f,
    pub v: Vec3f,
    pub radius: f32,
    pub area: f32,
    pub type_: f32,
}

pub struct BSDFState {
    pub depth: i32,
    pub eta: f32,
    pub hit_dist: f32,

    pub fhp: Vec3f,
    pub normal: Vec3f,
    pub ffnormal: Vec3f,
    pub tangent: Vec3f,
    pub bitangent: Vec3f,

    pub is_emitter: bool,

    pub tex_coord: Vec2f,
    pub mat: BSDFMaterial,
    pub medium: BSDFMedium,
}

impl Default for BSDFState {
    fn default() -> Self {
        Self::new()
    }
}

impl BSDFState {
    pub fn new() -> Self {
        Self {
            depth: 0,
            eta: 0.0,
            hit_dist: 0.0,
            fhp: Vec3f::zero(),
            normal: Vec3f::zero(),
            ffnormal: Vec3f::zero(),
            tangent: Vec3f::zero(),
            bitangent: Vec3f::zero(),
            is_emitter: false,
            tex_coord: Vec2f::zero(),
            mat: BSDFMaterial::new(),
            medium: BSDFMedium::new(),
        }
    }
}

pub struct BSDFScatterSampleRec {
    pub l: Vec3f,
    pub f: Vec3f,
    pub pdf: f32,
}

impl Default for BSDFScatterSampleRec {
    fn default() -> Self {
        Self::new()
    }
}

impl BSDFScatterSampleRec {
    pub fn new() -> Self {
        Self {
            l: Vec3f::zero(),
            f: Vec3f::zero(),
            pdf: 0.0,
        }
    }
}

pub struct BSDFLightSampleRec {
    pub normal: Vec3f,
    pub emission: Vec3f,
    pub direction: Vec3f,
    pub dist: f32,
    pub pdf: f32,
}

impl Default for BSDFLightSampleRec {
    fn default() -> Self {
        Self::new()
    }
}

impl BSDFLightSampleRec {
    pub fn new() -> Self {
        Self {
            normal: Vec3f::zero(),
            emission: Vec3f::zero(),
            direction: Vec3f::zero(),
            dist: 0.0,
            pdf: 0.0,
        }
    }
}

pub fn face_forward(a: Vec3f, b: Vec3f) -> Vec3f {
    if dot(a, b) < 0.0 {
        -b
    } else {
        b
    }
}

pub fn luminance(c: Vec3f) -> f32 {
    0.212671 * c.x + 0.715160 * c.y + 0.072169 * c.z
}

pub fn gtr1(ndoth: f32, a: f32) -> f32 {
    if a >= 1.0 {
        return f32::inv_pi();
    }
    let a2 = a * a;
    let t = 1.0 + (a2 - 1.0) * ndoth * ndoth;
    (a2 - 1.0) / (f32::pi() * a2.ln() * t)
}

pub fn sample_gtr1(rgh: f32, r1: f32, r2: f32) -> Vec3f {
    let a = rgh.max(0.001);
    let a2 = a * a;

    let phi = r1 * f32::two_pi();

    let cos_theta = ((1.0 - a2.powf(1.0 - r2)) / (1.0 - a2)).sqrt();
    let sin_theta = (1.0 - (cos_theta * cos_theta)).sqrt().clamp(0.0, 1.0);
    let sin_phi = phi.sin();
    let cos_phi = phi.cos();

    Vec3f::new(sin_theta * cos_phi, sin_theta * sin_phi, cos_theta)
}

pub fn gtr2(ndoth: f32, a: f32) -> f32 {
    let a2 = a * a;
    let t = 1.0 + (a2 - 1.0) * ndoth * ndoth;
    a2 / (f32::pi() * t * t)
}

pub fn sample_gtr2(rgh: f32, r1: f32, r2: f32) -> Vec3f {
    let a = rgh.max(0.001);

    let phi = r1 * f32::two_pi();

    let cos_theta = ((1.0 - r2) / (1.0 + (a * a - 1.0) * r2)).sqrt();
    let sin_theta = (1.0 - (cos_theta * cos_theta)).sqrt().clamp(0.0, 1.0);
    let sin_phi = phi.sin();
    let cos_phi = phi.cos();

    Vec3f::new(sin_theta * cos_phi, sin_theta * sin_phi, cos_theta)
}

pub fn sample_ggx_vndf(v: Vec3f, ax: f32, ay: f32, r1: f32, r2: f32) -> Vec3f {
    let vh = normalize(Vec3f::new(ax * v.x, ay * v.y, v.z));

    let lensq = vh.x * vh.x + vh.y * vh.y;
    let tt1 = if lensq > 0.0 {
        Vec3f::new(-vh.y, vh.x, 0.0) * rsqrt(lensq)
    } else {
        Vec3f::new(1.0, 0.0, 0.0)
    };
    let tt2 = cross(vh, tt1);

    let r = r1.sqrt();
    let phi = 2.0 * f32::pi() * r2;
    let t1 = r * phi.cos();
    let t2 = r * phi.sin();
    let s = 0.5 * (1.0 + vh.z);
    let t2 = (1.0 - s) * (1.0 - t1 * t1).sqrt() + s * t2;

    let nh = t1 * tt1 + t2 * tt2 + (1.0 - t1 * t1 - t2 * t2).max(0.0).sqrt() * vh;

    normalize(Vec3f::new(ax * nh.x, ay * nh.y, nh.z.max(0.0)))
}

pub fn gtr2_aniso(ndoth: f32, hdox: f32, hdoy: f32, ax: f32, ay: f32) -> f32 {
    let a = hdox / ax;
    let b = hdoy / ay;
    let c = a * a + b * b + ndoth * ndoth;
    1.0 / (f32::pi() * ax * ay * c * c)
}

pub fn sample_gtr2_aniso(ax: f32, ay: f32, r1: f32, r2: f32) -> Vec3f {
    let phi = r1 * f32::two_pi();

    let sin_phi = ay * phi.sin();
    let cos_phi = ax * phi.cos();
    let tan_theta = (r2 / (1.0 - r2)).sqrt();

    Vec3f::new(tan_theta * cos_phi, tan_theta * sin_phi, 1.0)
}

pub fn smith_g(ndotv: f32, alpha_g: f32) -> f32 {
    let a = alpha_g * alpha_g;
    let b = ndotv * ndotv;
    (2.0 * ndotv) / (ndotv + (a + b - a * b).sqrt())
}

pub fn smith_g_aniso(ndotv: f32, vdotx: f32, vdoty: f32, ax: f32, ay: f32) -> f32 {
    let a = vdotx * ax;
    let b = vdoty * ay;
    let c = ndotv;
    (2.0 * ndotv) / (ndotv + (a * a + b * b + c * c).sqrt())
}

pub fn schlick_weight(u: f32) -> f32 {
    let m = (1.0 - u).clamp(0.0, 1.0);
    let m2 = m * m;
    m2 * m2 * m
}

pub fn dielectric_fresnel(cos_theta_i: f32, eta: f32) -> f32 {
    let sin_theta_t_sq = eta * eta * (1.0 - cos_theta_i * cos_theta_i);

    if sin_theta_t_sq > 1.0 {
        return 1.0;
    }

    let cos_theta_t = (1.0 - sin_theta_t_sq).max(0.0).sqrt();

    let rs = (eta * cos_theta_t - cos_theta_i) / (eta * cos_theta_t + cos_theta_i);
    let rp = (eta * cos_theta_i - cos_theta_t) / (eta * cos_theta_i + cos_theta_t);

    0.5 * (rs * rs + rp * rp)
}

pub fn cosine_sample_hemisphere(r1: f32, r2: f32) -> Vec3f {
    let r = r1.sqrt();
    let phi = f32::two_pi() * r2;
    let x = r * phi.cos();
    let y = r * phi.sin();
    let z = (1.0 - x * x - y * y).max(0.0).sqrt();
    Vec3f::new(x, y, z)
}

pub fn uniform_sample_hemisphere(r1: f32, r2: f32) -> Vec3f {
    let r = (1.0 - r1 * r1).max(0.0).sqrt();
    let phi = f32::two_pi() * r2;
    Vec3f::new(r * phi.cos(), r * phi.sin(), r1)
}

pub fn uniform_sample_sphere(r1: f32, r2: f32) -> Vec3f {
    let z = 1.0 - 2.0 * r1;
    let r = (1.0 - z * z).max(0.0).sqrt();
    let phi = f32::two_pi() * r2;
    Vec3f::new(r * phi.cos(), r * phi.sin(), z)
}

pub fn power_heuristic(a: f32, b: f32) -> f32 {
    let t = a * a;
    t / (b * b + t)
}

pub fn onb(n: Vec3f, t: &mut Vec3f, b: &mut Vec3f) {
    let up = if n.z.abs() < 0.9999999 {
        Vec3f::new(0.0, 0.0, 1.0)
    } else {
        Vec3f::new(1.0, 0.0, 0.0)
    };
    *t = normalize(cross(n, up));
    *b = cross(n, *t);
}

pub fn sample_sphere_light(
    light: &BSDFLight,
    scatter_pos: Vec3f,
    light_sample: &mut BSDFLightSampleRec,
    num_of_lights: i32,
    rng: &mut ThreadRng,
    max_distance: f32,
) {
    let r1 = rng.gen();
    let r2 = rng.gen();

    let mut sphere_center_to_surface = scatter_pos - light.position;
    let dist_to_sphere_center = length(sphere_center_to_surface);

    sphere_center_to_surface /= dist_to_sphere_center;
    let sampled_dir = uniform_sample_hemisphere(r1, r2);
    let mut t = Vec3f::zero();
    let mut b = Vec3f::zero();
    onb(sphere_center_to_surface, &mut t, &mut b);
    let sampled_dir =
        t * sampled_dir.x + b * sampled_dir.y + sphere_center_to_surface * sampled_dir.z;

    let light_surface_pos = light.position + sampled_dir * light.radius;

    light_sample.direction = light_surface_pos - scatter_pos;
    light_sample.dist = length(light_sample.direction);
    let dist_sq = light_sample.dist * light_sample.dist;

    light_sample.direction /= light_sample.dist;
    light_sample.normal = normalize(light_surface_pos - light.position);
    light_sample.emission = light.emission * num_of_lights as f32;
    light_sample.pdf = dist_sq
        / (light.area * /*0.5*/ max_distance * dot(light_sample.normal, light_sample.direction).abs());
}

pub fn sample_rect_light(
    light: &BSDFLight,
    scatter_pos: Vec3f,
    light_sample: &mut BSDFLightSampleRec,
    num_of_lights: i32,
    rng: &mut ThreadRng,
) {
    let r1: f32 = rng.gen();
    let r2: f32 = rng.gen();

    let light_surface_pos = light.position + light.u * r1 + light.v * r2;
    light_sample.direction = light_surface_pos - scatter_pos;
    light_sample.dist = length(light_sample.direction);
    let dist_sq = light_sample.dist * light_sample.dist;
    light_sample.direction /= light_sample.dist;
    light_sample.normal = normalize(cross(light.u, light.v));
    light_sample.emission = light.emission * num_of_lights as f32;
    light_sample.pdf =
        dist_sq / (light.area * dot(light_sample.normal, light_sample.direction).abs());
}

pub fn sample_distant_light(
    light: &BSDFLight,
    scatter_pos: Vec3f,
    light_sample: &mut BSDFLightSampleRec,
    num_of_lights: i32,
) {
    light_sample.direction = normalize(light.position - Vec3f::zero());
    light_sample.normal = normalize(scatter_pos - light.position);
    light_sample.emission = light.emission * num_of_lights as f32;
    light_sample.dist = f32::INFINITY;
    light_sample.pdf = 1.0;
}

// fn sample_one_light(light: &Light, scatter_pos: Vec3f, light_sample: &mut LightSampleRec) {
//     let light_type = light.type_ as i32;

//     if light_type == QUAD_LIGHT {
//         sample_rect_light(light, scatter_pos, light_sample);
//     } else if light_type == SPHERE_LIGHT {
//         sample_sphere_light(light, scatter_pos, light_sample);
//     } else {
//         sample_distant_light(light, scatter_pos, light_sample);
//     }
// }

pub fn sample_hg(v: Vec3f, g: f32, r1: f32, r2: f32) -> Vec3f {
    let cos_theta = if g.abs() < 0.001 {
        1.0 - 2.0 * r2
    } else {
        let sqr_term = (1.0 - g * g) / (1.0 + g - 2.0 * g * r2);
        -(1.0 + g * g - sqr_term * sqr_term) / (2.0 * g)
    };

    let phi = r1 * f32::two_pi();
    let sin_theta = (1.0 - (cos_theta * cos_theta)).sqrt().clamp(0.0, 1.0);
    let sin_phi = phi.sin();
    let cos_phi = phi.cos();

    let mut v1 = Vec3f::zero();
    let mut v2 = Vec3f::zero();
    onb(v, &mut v1, &mut v2);

    sin_theta * cos_phi * v1 + sin_theta * sin_phi * v2 + cos_theta * v
}

#[allow(clippy::excessive_precision)]
pub fn phase_hg(cos_theta: f32, g: f32) -> f32 {
    let denom = 1.0 + g * g + 2.0 * g * cos_theta;
    /*INV_4_PI*/
    0.07957747154594766 * (1.0 - g * g) / (denom * denom.sqrt())
}

pub fn to_world(x: Vec3f, y: Vec3f, z: Vec3f, v: Vec3f) -> Vec3f {
    v.x * x + v.y * y + v.z * z
}

pub fn to_local(x: Vec3f, y: Vec3f, z: Vec3f, v: Vec3f) -> Vec3f {
    Vec3f::new(dot(v, x), dot(v, y), dot(v, z))
}

pub fn tint_colors(
    mat: &BSDFMaterial,
    eta: f32,
    f0: &mut f32,
    csheen: &mut Vec3f,
    cspec0: &mut Vec3f,
) {
    let lum = luminance(mat.base_color);
    let ctint = if lum > 0.0 {
        mat.base_color / lum
    } else {
        Vec3f::new(1.0, 1.0, 1.0)
    };

    *f0 = (1.0 - eta) / (1.0 + eta);
    *f0 *= *f0;

    *cspec0 = *f0 * lerp(Vec3f::new(1.0, 1.0, 1.0), ctint, mat.specular_tint);
    *csheen = lerp(Vec3f::new(1.0, 1.0, 1.0), ctint, mat.sheen_tint);
}

pub fn eval_disney_diffuse(
    mat: &BSDFMaterial,
    csheen: Vec3f,
    v: Vec3f,
    l: Vec3f,
    h: Vec3f,
    pdf: &mut f32,
) -> Vec3f {
    *pdf = 0.0;
    if l.z <= 0.0 {
        return Vec3f::zero();
    }

    let l_dot_h = dot(l, h);

    let rr = 2.0 * mat.roughness * l_dot_h * l_dot_h;

    // Diffuse
    let fl = schlick_weight(l.z);
    let fv = schlick_weight(v.z);
    let fretro = rr * (fl + fv + fl * fv * (rr - 1.0));
    let fd = (1.0 - 0.5 * fl) * (1.0 - 0.5 * fv);

    // Fake subsurface
    let fss90 = 0.5 * rr;
    let fss = lerp(1.0, fss90, fl) * lerp(1.0, fss90, fv);
    let ss = 1.25 * (fss * (1.0 / (l.z + v.z) - 0.5) + 0.5);

    // Sheen
    let fh = schlick_weight(l_dot_h);
    let fsheen = fh * mat.sheen * csheen;

    *pdf = l.z * f32::inv_pi();
    f32::inv_pi() * mat.base_color * lerp(fd + fretro, ss, mat.subsurface) + fsheen
}

pub fn eval_microfacet_reflection(
    mat: &BSDFMaterial,
    v: Vec3f,
    l: Vec3f,
    h: Vec3f,
    f: Vec3f,
    pdf: &mut f32,
) -> Vec3f {
    *pdf = 0.0;
    if l.z <= 0.0 {
        return Vec3f::zero();
    }

    let d = gtr2_aniso(h.z, h.x, h.y, mat.ax, mat.ay);
    let g1 = smith_g_aniso(v.z.abs(), v.x, v.y, mat.ax, mat.ay);
    let g2 = g1 * smith_g_aniso(l.z.abs(), l.x, l.y, mat.ax, mat.ay);

    *pdf = g1 * d / (4.0 * v.z);
    f * d * g2 / (4.0 * l.z * v.z)
}

pub fn eval_microfacet_refraction(
    mat: &BSDFMaterial,
    eta: f32,
    v: Vec3f,
    l: Vec3f,
    h: Vec3f,
    f: Vec3f,
    pdf: &mut f32,
) -> Vec3f {
    *pdf = 0.0;
    if l.z >= 0.0 {
        return Vec3f::zero();
    }

    let l_dot_h = dot(l, h);
    let v_dot_h = dot(v, h);

    let d = gtr2_aniso(h.z, h.x, h.y, mat.ax, mat.ay);
    let g1 = smith_g_aniso(v.z.abs(), v.x, v.y, mat.ax, mat.ay);
    let g2 = g1 * smith_g_aniso(l.z.abs(), l.x, l.y, mat.ax, mat.ay);
    let denom = l_dot_h + v_dot_h * eta;
    let denom = denom * denom;
    let eta2 = eta * eta;
    let jacobian = l_dot_h.abs() / denom;

    *pdf = g1 * v_dot_h.max(0.0) * d * jacobian / v.z;
    powf(mat.base_color, 0.5) * (1.0 - f) * d * g2 * v_dot_h.abs() * jacobian * eta2
        / (l.z * v.z).abs()
}

pub fn eval_clearcoat(mat: &BSDFMaterial, v: Vec3f, l: Vec3f, h: Vec3f, pdf: &mut f32) -> Vec3f {
    *pdf = 0.0;
    if l.z <= 0.0 {
        return Vec3f::zero();
    }

    let v_dot_h = dot(v, h);

    let f = lerp(0.04, 1.0, schlick_weight(v_dot_h));
    let d = gtr1(h.z, mat.clearcoat_roughness);
    let g = smith_g(l.z, 0.25) * smith_g(v.z, 0.25);
    let jacobian = 1.0 / (4.0 * v_dot_h);

    *pdf = d * h.z * jacobian;
    Vec3f::new(f, f, f) * d * g
}

pub fn disney_sample(
    state: &BSDFState,
    v: Vec3f,
    n: Vec3f,
    ll: &mut Vec3f,
    pdf: &mut f32,
    rng: &mut ThreadRng,
) -> Vec3f {
    *pdf = 0.0;

    let r1 = rng.gen();
    let r2 = rng.gen();

    // TODO: Tangent and bitangent should be calculated from mesh (provided, the mesh has proper uvs)
    let mut t = Vec3f::zero();
    let mut b = Vec3f::zero();
    onb(n, &mut t, &mut b);

    // Transform to shading space to simplify operations (NDotL = L.z; NDotV = V.z; NDotH = H.z)
    let v = to_local(t, b, n, v);

    // Tint colors
    let mut csheen = Vec3f::zero();
    let mut cspec0 = Vec3f::zero();
    let mut f0 = 0.0;
    tint_colors(&state.mat, state.eta, &mut f0, &mut csheen, &mut cspec0);

    // Model weights
    let dielectric_wt = (1.0 - state.mat.metallic) * (1.0 - state.mat.spec_trans);
    let metal_wt = state.mat.metallic;
    let glass_wt = (1.0 - state.mat.metallic) * state.mat.spec_trans;

    // Lobe probabilities
    let schlick_wt = schlick_weight(v.z);

    let diff_pr = dielectric_wt * luminance(state.mat.base_color);
    let dielectric_pr =
        dielectric_wt * luminance(lerp(cspec0, Vec3f::new(1.0, 1.0, 1.0), schlick_wt));
    let metal_pr = metal_wt
        * luminance(lerp(
            state.mat.base_color,
            Vec3f::new(1.0, 1.0, 1.0),
            schlick_wt,
        ));
    let glass_pr = glass_wt;
    let clear_ct_pr = 0.25 * state.mat.clearcoat;

    // Normalize probabilities
    let inv_total_wt = 1.0 / (diff_pr + dielectric_pr + metal_pr + glass_pr + clear_ct_pr);
    let diff_pr = diff_pr * inv_total_wt;
    let dielectric_pr = dielectric_pr * inv_total_wt;
    let metal_pr = metal_pr * inv_total_wt;
    let glass_pr = glass_pr * inv_total_wt;
    let clear_ct_pr = clear_ct_pr * inv_total_wt;

    // CDF of the sampling probabilities
    let cdf = [
        diff_pr,
        diff_pr + dielectric_pr,
        diff_pr + dielectric_pr + metal_pr,
        diff_pr + dielectric_pr + metal_pr + glass_pr,
        diff_pr + dielectric_pr + metal_pr + glass_pr + clear_ct_pr,
    ];

    // Sample a lobe based on its importance
    let r3: f32 = rng.gen();

    let l = if r3 < cdf[0] {
        cosine_sample_hemisphere(r1, r2)
    } else if r3 < cdf[2] {
        let mut h = sample_ggx_vndf(v, state.mat.ax, state.mat.ay, r1, r2);

        if h.z < 0.0 {
            h = -h;
        }

        normalize(Vec3f::reflect(-v, h))
    } else if r3 < cdf[3] {
        let mut h = sample_ggx_vndf(v, state.mat.ax, state.mat.ay, r1, r2);
        let f = dielectric_fresnel(dot(v, h).abs(), state.eta);

        if h.z < 0.0 {
            h = -h;
        }

        let r3 = (r3 - cdf[2]) / (cdf[3] - cdf[2]);

        if r3 < f {
            normalize(Vec3f::reflect(-v, h))
        } else {
            normalize(Vec3f::refract(-v, h, state.eta))
        }
    } else {
        let mut h = sample_gtr1(state.mat.clearcoat_roughness, r1, r2);

        if h.z < 0.0 {
            h = -h;
        }

        normalize(Vec3f::reflect(-v, h))
    };

    let l = to_world(t, b, n, l);
    let v = to_world(t, b, n, v);

    *ll = l;

    disney_eval(state, v, n, l, pdf)
}

pub fn disney_eval(state: &BSDFState, vv: Vec3f, nn: Vec3f, ll: Vec3f, pdf: &mut f32) -> Vec3f {
    *pdf = 0.0;
    let mut f = Vec3f::zero();

    // TODO: Tangent and bitangent should be calculated from mesh (provided, the mesh has proper uvs)
    let mut t = Vec3f::zero();
    let mut b = Vec3f::zero();
    onb(nn, &mut t, &mut b);

    // Transform to shading space to simplify operations (NDotL = L.z; NDotV = V.z; NDotH = H.z)
    let v = to_local(t, b, nn, vv);
    let l = to_local(t, b, nn, ll);

    let mut h = if l.z > 0.0 {
        normalize(l + v)
    } else {
        normalize(l + v * state.eta)
    };

    if h.z < 0.0 {
        h = -h;
    }

    // Tint colors
    let mut csheen = Vec3f::zero();
    let mut cspec0 = Vec3f::zero();
    let mut f0 = 0.0;
    tint_colors(&state.mat, state.eta, &mut f0, &mut csheen, &mut cspec0);

    // Model weights
    let dielectric_wt = (1.0 - state.mat.metallic) * (1.0 - state.mat.spec_trans);
    let metal_wt = state.mat.metallic;
    let glass_wt = (1.0 - state.mat.metallic) * state.mat.spec_trans;

    // Lobe probabilities
    let schlick_wt = schlick_weight(v.z);

    let diff_pr = dielectric_wt * luminance(state.mat.base_color);
    let dielectric_pr =
        dielectric_wt * luminance(lerp(cspec0, Vec3f::new(1.0, 1.0, 1.0), schlick_wt));
    let metal_pr = metal_wt
        * luminance(lerp(
            state.mat.base_color,
            Vec3f::new(1.0, 1.0, 1.0),
            schlick_wt,
        ));
    let glass_pr = glass_wt;
    let clear_ct_pr = 0.25 * state.mat.clearcoat;

    // Normalize probabilities
    let inv_total_wt = 1.0 / (diff_pr + dielectric_pr + metal_pr + glass_pr + clear_ct_pr);
    let diff_pr = diff_pr * inv_total_wt;
    let dielectric_pr = dielectric_pr * inv_total_wt;
    let metal_pr = metal_pr * inv_total_wt;
    let glass_pr = glass_pr * inv_total_wt;
    let clear_ct_pr = clear_ct_pr * inv_total_wt;

    let reflect = l.z * v.z > 0.0;

    let mut tmp_pdf = 0.0;
    let v_dot_h = dot(v, h).abs();

    if diff_pr > 0.0 && reflect {
        f += eval_disney_diffuse(&state.mat, csheen, v, l, h, &mut tmp_pdf) * dielectric_wt;
        *pdf += tmp_pdf * diff_pr;
    }

    if dielectric_pr > 0.0 && reflect {
        let ff = (dielectric_fresnel(v_dot_h, 1.0 / state.mat.ior) - f0) / (1.0 - f0);

        f += eval_microfacet_reflection(
            &state.mat,
            v,
            l,
            h,
            lerp(cspec0, Vec3f::new(1.0, 1.0, 1.0), ff),
            &mut tmp_pdf,
        ) * dielectric_wt;
        *pdf += tmp_pdf * dielectric_pr;
    }

    if metal_pr > 0.0 && reflect {
        let ff = lerp(
            state.mat.base_color,
            Vec3f::new(1.0, 1.0, 1.0),
            schlick_weight(v_dot_h),
        );

        f += eval_microfacet_reflection(&state.mat, v, l, h, ff, &mut tmp_pdf) * metal_wt;
        *pdf += tmp_pdf * metal_pr;
    }

    if glass_pr > 0.0 {
        let ff = dielectric_fresnel(v_dot_h, state.eta);

        if reflect {
            f += eval_microfacet_reflection(
                &state.mat,
                v,
                l,
                h,
                Vec3f::new(ff, ff, ff),
                &mut tmp_pdf,
            ) * glass_wt;
            *pdf += tmp_pdf * glass_pr * ff;
        } else {
            f += eval_microfacet_refraction(
                &state.mat,
                state.eta,
                v,
                l,
                h,
                Vec3f::new(ff, ff, ff),
                &mut tmp_pdf,
            ) * glass_wt;
            *pdf += tmp_pdf * glass_pr * (1.0 - ff);
        }
    }

    if clear_ct_pr > 0.0 && reflect {
        f += eval_clearcoat(&state.mat, v, l, h, &mut tmp_pdf) * 0.25 * state.mat.clearcoat;
        *pdf += tmp_pdf * clear_ct_pr;
    }

    f * l.z.abs()
}
