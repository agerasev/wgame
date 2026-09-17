@group(0) @binding(0) var<uniform> color: vec4<f32>;

@vertex
fn vertex_main(@builtin(vertex_index) index: u32) -> @builtin(position) vec4<f32> {
    let positions = array<vec2<f32>, 3>(vec2(-1.0, -1.0), vec2(3.0, -1.0), vec2(-1.0, 3.0));
    return vec4(positions[index], 1.0, 1.0);
}

@fragment
fn color_float() -> @location(0) vec4<f32> { return color; }

@fragment
fn color_uint() -> @location(0) vec4<u32> { return vec4<u32>(color); }

@fragment
fn color_sint() -> @location(0) vec4<i32> { return vec4<i32>(color); }
