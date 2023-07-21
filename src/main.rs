use test_gpu::core::TestGPU;

mod engine_base;
mod test_gpu {
    pub mod core;
}
mod util;
mod texture;

fn main() {
    engine_base::run::<TestGPU>();
}
