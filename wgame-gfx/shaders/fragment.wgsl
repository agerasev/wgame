@group(1)
@binding(0)
var texture: texture_2d<f32>;
@group(1)
@binding(1)
var sampler_: sampler;

@fragment
fn main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(texture, sampler_, vertex.tex_coord);
}
