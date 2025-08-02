struct VertexInput {
    @location(0) position: vec4<f32>,
};

struct InstanceInput {
    @location(1) xform_0: vec4<f32>,
    @location(2) xform_1: vec4<f32>,
    @location(3) xform_2: vec4<f32>,
    @location(4) xform_3: vec4<f32>,
    @location(5) tex_rect: vec4<f32>,
    @location(6) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
    @location(1) color: vec4<f32>,
};

@vertex
fn vertex_main(
    vertex: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let xform = mat4x4<f32>(
        instance.xform_0,
        instance.xform_1,
        instance.xform_2,
        instance.xform_3,
    );
    let tex_offset = instance.tex_rect.xy;
    let tex_size = instance.tex_rect.zw;

    let local_coord = vec2<f32>(vertex.position.x, -vertex.position.y);
    var output: VertexOutput;
    output.position = xform * vertex.position;
    output.tex_coord = tex_offset + local_coord * tex_size;
    output.color = instance.color;

    return output;
}

@group(0)
@binding(0)
var texture: texture_2d<u32>;

@fragment
fn fragment_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    let tex_coord = vec2<u32>(vertex.tex_coord);
    let int_value = textureLoad(texture, tex_coord, 0).x;
    let value = f32(int_value) / 255.0;

    // var color = vec4<f32>(1.0, 1.0, 1.0, value);
    var color = vec4<f32>(value, value, value, 1.0);
    color *= vertex.color;

    return color;
}
