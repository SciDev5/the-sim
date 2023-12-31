@vertex
fn vert(
    @location(0) position: vec2<f32>,
) -> @builtin(position) vec4<f32> {
    return vec4<f32>(position, 0.0, 1.0);
}

@fragment
fn frag() -> @location(0) vec4<f32> {
    return vec4<f32>(0.0, 1.0, 1.0, 1.0);
}