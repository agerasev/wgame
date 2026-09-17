// Derivatives/samples stay outside varying control flow for WebGL2 uniformity.
let dp_x = dpdx(input.world_position);
let dp_y = dpdy(input.world_position);
let uv_x = dpdx(input.normal_coord);
let uv_y = dpdy(input.normal_coord);
let normal_rgba = textureSample(normal_texture, normal_sampler, input.normal_coord);
let normal_sample = normal_rgba.rgb / select(1.0, max(normal_rgba.a, 1e-20), normal_texture_info.x != 0u);
var normal = safe_unit(input.world_normal, vec3(0.0, 0.0, 1.0));
let uv_det = uv_x.x * uv_y.y - uv_x.y * uv_y.x;
let uv_scale = sqrt(dot(uv_x, uv_x) * dot(uv_y, uv_y));
if input.settings.x > 0.0 && abs(uv_det) > 1e-6 * uv_scale {
    let tangent_raw = (dp_x * uv_y.y - dp_y * uv_x.y) / uv_det;
    let bitangent_raw = (dp_y * uv_x.x - dp_x * uv_y.x) / uv_det;
    let tangent_plane = tangent_raw - normal * dot(normal, tangent_raw);
    if dot(tangent_plane, tangent_plane) > 1e-20 {
        let tangent = normalize(tangent_plane);
        let handedness = select(-1.0, 1.0, dot(cross(normal, tangent), bitangent_raw) >= 0.0);
        let bitangent = handedness * cross(normal, tangent);
        let decoded = 2.0 * normal_sample - vec3(1.0);
        let mapped = safe_unit(vec3(decoded.x * input.settings.x, decoded.y * input.settings.x * input.settings.y, decoded.z), vec3(0.0,0.0,1.0));
        normal = safe_unit(mat3x3<f32>(tangent, bitangent, normal) * mapped, normal);
    }
}
var albedo = sampled_color.rgb;
if input.settings.z > 0.5 { albedo = srgb_to_linear(albedo); }
albedo *= input.color.rgb;
let diffuse = max(dot(normal, lighting.direction.xyz), 0.0);
let view_direction = safe_unit(lighting.eye.xyz - input.world_position, normal);
let halfway = safe_unit(view_direction + lighting.direction.xyz, normal);
let highlight = select(0.0, input.surface.x * pow(max(dot(normal, halfway), 0.0), input.surface.y), diffuse > 0.0);
let lit_color = albedo * (lighting.ambient.xyz + diffuse * lighting.color.xyz) + highlight * lighting.color.xyz;
