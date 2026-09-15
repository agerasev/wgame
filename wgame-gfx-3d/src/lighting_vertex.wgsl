let world = model_matrix * vertex.position;
output.world_position = world.xyz;
// Cofactors give the inverse transpose up to |determinant|. Keep its sign
// for reflected instances; avoid division so planar 2D transforms also work.
let cofactors = mat3x3<f32>(
    cross(model_matrix[1].xyz, model_matrix[2].xyz),
    cross(model_matrix[2].xyz, model_matrix[0].xyz),
    cross(model_matrix[0].xyz, model_matrix[1].xyz),
);
let orientation = select(-1.0, 1.0, dot(model_matrix[0].xyz, cofactors[0]) >= 0.0);
output.world_normal = orientation * (cofactors * vertex.normal);
output.normal_coord = mat3x2<f32>(
    instance.normal_tex_xform_m.xy,
    instance.normal_tex_xform_m.zw,
    instance.normal_tex_xform_v,
) * vertex.local_coord;
output.settings = instance.settings;
output.surface = instance.surface;
