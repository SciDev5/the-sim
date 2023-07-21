struct ImageBuffer {
    data: array<u32>,
};


@group(0) @binding(0) var<storage, read_write> pixels: ImageBuffer;

@compute
@workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    // Get the current pixel position in the 2D array
    let pixelX = global_id.x;
    let pixelY = global_id.y;

    // Check bounds to avoid accessing out-of-range pixels
    if (pixelX < 512u && pixelY < 512u) {
        pixels.data[pixelX + 512u * pixelY] += pixelX + 512u * pixelY;
    }
}