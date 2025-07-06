@group(1)
@binding(0)
var texture: texture_2d<f32>;

@group(1)
@binding(1)
var sampler_: sampler;

{% for (i, u) in uniforms|enumerate %}
@group(1)
@binding({{ i|add(2) }})
var<uniform> {{ u.name }}: {{ u.ty }};
{% endfor %}

@fragment
fn main(vertex: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(texture, sampler_, vertex.tex_coord);

    let coord = vertex.local_coord;
    var mask = true;
    {{ mask_stmt }}
    if (!mask) {
        discard;
    }

    return color;
}
