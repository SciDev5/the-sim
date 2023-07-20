use std::borrow::Cow;

use naga::ShaderStage;
use wgpu::ShaderModuleDescriptor;


#[doc(hidden)]
pub fn include_glsl_fn<'a>(
    label: Option<&'a str>,
    src: &'a str,
    stage: ShaderStage,
) -> ShaderModuleDescriptor<'a> {
    let options = stage.into();
    let mut parser = naga::front::glsl::Frontend::default();
    let module = parser.parse(&options, src).unwrap();

    ShaderModuleDescriptor { label, source: wgpu::ShaderSource::Naga(Cow::Owned(module)) }
}

#[macro_export]
macro_rules! include_glsl {
    ($name: tt, $stage: expr) => {
        $crate::util::include_glsl::include_glsl_fn(Some($name), include_str!($name), $stage)
    };
}