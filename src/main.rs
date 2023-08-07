mod engine_base;
mod test_gpu {
    pub mod core;
}
mod wave {
    pub mod core;
}
mod blackhole_gtx {
    pub mod core;
}
mod util;
mod texture;
mod bind_group;
mod buffer;

mod new_abstractions;

fn main() {
    // engine_base::run::<wave::core::Wave>();
    engine_base::run::<blackhole_gtx::core::BlackholeGtx>();
}
