@group(1)
@binding(0)
var texture: texture_2d<f32>;
@group(1)
@binding(1)
var sampler_: sampler;

@fragment
fn main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    let x = vertex.local_coord.x;
    let y = vertex.local_coord.y;
    if (!({{mask_expr}})) {
        discard;
    }
    return textureSample(texture, sampler_, vertex.tex_coord);
}
