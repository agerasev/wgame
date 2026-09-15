struct LightingData {
    ambient: vec4<f32>,
    direction: vec4<f32>,
    color: vec4<f32>,
    eye: vec4<f32>,
};
@group(2) @binding(0) var normal_texture: texture_2d<f32>;
@group(2) @binding(1) var normal_sampler: sampler;
@group(3) @binding(0) var<uniform> lighting: LightingData;

fn safe_unit(v: vec3<f32>, fallback: vec3<f32>) -> vec3<f32> {
    let squared = dot(v,v);
    if squared > 1e-20 { return v * inverseSqrt(squared); }
    return fallback;
}
fn srgb_to_linear(rgb: vec3<f32>) -> vec3<f32> {
    let c = max(rgb, vec3(0.0));
    return select(pow((c + vec3(0.055)) / 1.055, vec3(2.4)), c / 12.92, c <= vec3(0.04045));
}
fn linear_to_srgb(rgb: vec3<f32>) -> vec3<f32> {
    let c = max(rgb, vec3(0.0));
    return select(1.055 * pow(c, vec3(1.0 / 2.4)) - vec3(0.055), 12.92 * c, c <= vec3(0.0031308));
}
