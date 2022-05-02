struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>
};
struct Uniform {
    transform: mat4x4<f32>
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>
};


@group(0) @binding(0)
var<uniform> u: Uniform;

@stage(vertex)
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.color = model.color;
    out.clip_position = vec4<f32>(model.position, 1.0)*u.transform;
    return out;
}

@stage(fragment)
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color.x, in.color.y, in.color.z, 1.0);
}
