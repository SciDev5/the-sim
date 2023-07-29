use wave::core::Wave;

mod engine_base;
mod test_gpu {
    pub mod core;
}
mod wave {
    pub mod core;
}
mod util;
mod texture;
mod bind_group;
mod buffer;

mod new_abstractions;

fn main() {
    engine_base::run::<Wave>();
}
