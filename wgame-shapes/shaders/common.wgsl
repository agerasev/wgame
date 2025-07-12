struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) local_coord: vec2<f32>,
    @location(1) tex_coord: vec2<f32>,
};
