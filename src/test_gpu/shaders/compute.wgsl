struct ImageBuffer {
    data: array<u32>,
};


@group(0) @binding(0) var<storage, read_write> pixels: ImageBuffer;
@group(0) @binding(1) var out_tex: texture_storage_2d<rgba8unorm, write>;

@compute
@workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    // Get the current pixel position in the 2D array
    let pixelX = global_id.x;
    let pixelY = global_id.y;
    let coord = vec2<i32>(global_id.xy);

    // Check bounds to avoid accessing out-of-range pixels
    if (pixelX < 512u && pixelY < 512u) {
        pixels.data[pixelX + 512u * pixelY] += pixels.data[((pixelX+1u)%256u) + 512u * ((pixelY+1u)%256u)] / 64u;
        // pixels.data[pixelX + 512u * pixelY] -= pixels.data[((pixelX+1u)%512u) + 512u * ((pixelY+1u)%512u)] / 64u;
        // pixels.data[pixelX + 512u * pixelY] += pixels.data[((pixelX+4294967168u)%256u) + 2047u * ((pixelY+1u)%128u) / 4u] / 64u;
        // pixels.data[pixelX + 512u * pixelY] += pixels.data[((pixelX+128u)%512u) + 512u * ((pixelY+15u)%512u)] / 64u;

        let d = pixels.data[pixelX + 512u * pixelY];
        
        textureStore(out_tex, coord, vec4<f32>(0.5,f32(d % 0x10000u)/0x10000.0,f32(d)/0x100000000.0,1.));
    }
}