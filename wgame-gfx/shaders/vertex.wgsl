@group(0)
@binding(0)
var<uniform> xform: mat4x4<f32>;
@group(0)
@binding(1)
var<uniform> tex_xform: mat3x2<f32>;

@vertex
fn main(
    @location(0) position: vec4<f32>,
    @location(1) tex_coord: vec2<f32>,
) -> VertexOutput {
    var result: VertexOutput;
    result.tex_coord = tex_xform * vec3(tex_coord, 1.0);
    result.position = xform * position;
    return result;
}
