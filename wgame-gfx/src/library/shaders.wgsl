struct VertexOutput {
    @location(0) tex_coord: vec2<f32>,
    @builtin(position) position: vec4<f32>,
};

@group(0)
@binding(0)
var<uniform> xform: mat4x4<f32>;
@group(0)
@binding(1)
var<uniform> tex_xform: mat3x2<f32>;

@vertex
fn vs_main(
    @location(0) position: vec4<f32>,
    @location(1) tex_coord: vec2<f32>,
) -> VertexOutput {
    var result: VertexOutput;
    result.tex_coord = tex_xform * vec3(tex_coord, 1.0);
    result.position = xform * position;
    return result;
}

@group(1)
@binding(0)
var texture: texture_2d<f32>;
@group(1)
@binding(1)
var sampler_: sampler;

@fragment
fn fs_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(texture, sampler_, vertex.tex_coord);
}
