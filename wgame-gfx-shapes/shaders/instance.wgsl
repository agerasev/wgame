const PI: f32 = 3.141592653589793238462643;

struct VertexData {
    @location(0) position: vec4<f32>,
    @location(1) local_coord: vec3<f32>,
};

struct InstanceData {
    @location(2) xform_0: vec4<f32>,
    @location(3) xform_1: vec4<f32>,
    @location(4) xform_2: vec4<f32>,
    @location(5) xform_3: vec4<f32>,
    @location(6) tex_xform_m: vec4<f32>,
    @location(7) tex_xform_v: vec2<f32>,
    @location(8) tex_color: vec4<f32>,

    {% for (i, a) in instance|enumerate %}
    @location({{ i|add(9) }}) {{ a.name }}: {{ a.ty }},
    {% endfor %}
};

struct VaryingData {
    @builtin(position) position: vec4<f32>,
    @location(0) local_coord: vec3<f32>,
    @location(1) tex_coord: vec2<f32>,
    @location(2) color: vec4<f32>,

    {% for (i, a) in varying|enumerate %}
    @location({{ i|add(3) }}) {{ a.name }}: {{ a.ty }},
    {% endfor %}
};

@group(0)
@binding(0)
var<uniform> view_matrix: mat4x4<f32>;
@group(0)
@binding(1)
var<uniform> view_color: vec4<f32>;

@vertex
fn vertex_main(
    vertex: VertexData,
    instance: InstanceData,
) -> VaryingData {
    let model_matrix = mat4x4<f32>(
        instance.xform_0,
        instance.xform_1,
        instance.xform_2,
        instance.xform_3,
    );
    let tex_xform = mat3x2<f32>(
        instance.tex_xform_m.xy,
        instance.tex_xform_m.zw,
        instance.tex_xform_v,
    );

    var output: VaryingData;
    output.position = view_matrix * model_matrix * vertex.position;
    output.local_coord = vertex.local_coord;
    output.tex_coord = tex_xform * vertex.local_coord;
    output.color = view_color * instance.tex_color;

    {{ vertex_source }}

    return output;
}

@group(1)
@binding(0)
var texture: texture_2d<f32>;

@group(1)
@binding(1)
var sampler_: sampler;

{% for (i, a) in fragment_uniforms|enumerate %}
@group(2)
@binding({{ i }})
var<uniform> {{ a.name }}: {{ a.ty }};
{% endfor %}

@fragment
fn fragment_main(input: VaryingData) -> @location(0) vec4<f32> {
    var tex_coord = input.tex_coord;

    {{ fragment_texcoord_source }}

    var color = textureSample(texture, sampler_, tex_coord);
    color *= input.color;

    {{ fragment_color_source }}

    return color;
}
