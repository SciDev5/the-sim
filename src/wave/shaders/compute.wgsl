struct WavePoint {
    x: f32,
    v: f32,
};

const size: u32 = 256u;

const c: f32 = 0.1;
const dt: f32 = 0.005;
const du: f32 = 0.01;

@group(0) @binding(0) var<storage, read> d_in: array<WavePoint>;
@group(0) @binding(1) var<storage, read_write> d_out: array<WavePoint>;
@group(0) @binding(2) var out_tex: texture_storage_2d<rgba8unorm, write>;

@compute
@workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    // Get the current pixel position in the 2D array
    let pixelX = global_id.x;
    let pixelY = global_id.y;
    let coord = vec2<i32>(global_id.xy);

    // Check bounds to avoid accessing out-of-range pixels
    if (pixelX < size - 1u && pixelY < size - 1u && pixelX > 0u && pixelY > 0u) {
        let i = pixelX + pixelY * size;

        let d2u_dt2 = c * (
            (
                (d_in[i + 1u].x - d_in[i].x) / du -
                (d_in[i].x - d_in[i - 1u].x) / du
            ) / du +
            (
                (d_in[i + size].x - d_in[i].x) / du -
                (d_in[i].x - d_in[i - size].x) / du
            ) / du
        );

        var out = d_in[i];
        out.v += d2u_dt2 * dt;
        out.x += out.v * dt;

        d_out[i] = out;
        
        textureStore(out_tex, coord, vec4<f32>(-out.x, 0.0, out.x, 1.0));
    }
}