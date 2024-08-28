use crate::prelude::*;
use rayon::prelude::*;

pub struct FTContext {
    pub nodes: Vec<Node>,
    pub shapes: Vec<usize>,
    pub patterns: Vec<usize>,

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

            variables: FxHashMap::default(),

            output: None,
        }
    }

    pub fn render(&self, width: usize, height: usize, buffer: &mut Vec<u8>) {
        const LINES: usize = 20;

        buffer
            .par_rchunks_exact_mut(width * LINES * 3)
            .enumerate()
            .for_each(|(j, line)| {
                for (i, pixel) in line.chunks_exact_mut(3).enumerate() {
                    let i = (LINES - j - 1) * width * LINES + i;
                    let x = (i % width) as f32;
                    let y = (i / width) as f32;

                    let xx = x / width as f32;
                    let yy = 1.0 - y / height as f32;

                    // vec2 p = (2.0*fragCoord-iResolution.xy)/iResolution.y;

                    //let ratio = width as f32 / height as f32;

                    let p = //vec2f(x, height as f32 - y);
                    (2.0 * vec2f(x, y) - vec2f(width as f32, height as f32)) / height as f32;
                    //let p = vec2f(x - width as f32 / 2.0, y - height as f32 / 2.0);
                    let mut color = vec3f(0.0, 0.0, 0.0);

                    let d = sdf_box2d(p, 0.5, 0.5); // - 0.2;
                    if d < 0.0 {
                        color.x = 1.0;
                    }

                    let out = [
                        (color.x * 255.0) as u8,
                        (color.y * 255.0) as u8,
                        (color.z * 255.0) as u8,
                    ];

                    pixel.copy_from_slice(&out);
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
