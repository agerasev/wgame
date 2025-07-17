struct VertexInput {
    @location(0) position: vec4<f32>,
    @location(1) local_coord: vec2<f32>,
};

struct InstanceInput {
    @location(2) xform_0: vec4<f32>,
    @location(3) xform_1: vec4<f32>,
    @location(4) xform_2: vec4<f32>,
    @location(5) xform_3: vec4<f32>,
    @location(6) tex_xform_m: vec4<f32>,
    @location(7) tex_xform_v: vec2<f32>,
    
    {% for (i, a) in instances|enumerate %}
    @location({{ i|add(8) }}) {{ a.name }}: {{ a.ty }},
    {% endfor %}
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) local_coord: vec2<f32>,
    @location(1) tex_coord: vec2<f32>,

    {% for (i, a) in instances|enumerate %}
    @location({{ i|add(2) }}) {{ a.name }}: {{ a.ty }},
    {% endfor %}
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
    let tex_xform = mat3x2<f32>(
        instance.tex_xform_m.xy,
        instance.tex_xform_m.zw,
        instance.tex_xform_v,
    );

    var output: VertexOutput;
    output.position = xform * vertex.position;
    output.local_coord = vertex.local_coord;
    output.tex_coord = tex_xform * vec3(vertex.local_coord, 1.0);

    {% for (i, a) in instances|enumerate %}
    output.{{ a.name }} = instance.{{ a.name }};
    {% endfor %}

    return output;
}

@group(0)
@binding(0)
var texture: texture_2d<f32>;

@group(0)
@binding(1)
var sampler_: sampler;

{% for (i, a) in uniforms|enumerate %}
@group(1)
@binding({{ i }})
var<uniform> {{ a.name }}: {{ a.ty }};
{% endfor %}

@fragment
fn fragment_main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(texture, sampler_, vertex.tex_coord);
    let coord = vertex.local_coord;

    {{ fragment_modifier }}

    return color;
}
