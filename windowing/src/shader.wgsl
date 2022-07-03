struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>
};
struct Uniform {
    transform: mat4x4<f32>
};
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>
};


@group(0) @binding(0) var<uniform> u: Uniform;
@group(0) @binding(1) var transform_texture : texture_1d<f32>;

@vertex
fn vs_main(
        model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.color = model.color;
    var t1 = textureLoad(transform_texture,0,0);
    var t2 = textureLoad(transform_texture,1,0);
    var t3 = textureLoad(transform_texture,2,0);
    var t4 = textureLoad(transform_texture,3,0);
    var s1 = textureLoad(transform_texture,4,0);
    var s2 = textureLoad(transform_texture,5,0);
    var s3 = textureLoad(transform_texture,6,0);
    var s4 = textureLoad(transform_texture,7,0);
    var texture_transform = mat4x4<f32>(t1,t2,t3,t4);
    var sexture_transform = mat4x4<f32>(s1,s2,s3,s4);
    out.clip_position =texture_transform*sexture_transform*vec4<f32>(model.position, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
