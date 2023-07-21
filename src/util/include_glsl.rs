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
    let src_transformed = src.split("\n").map(|ln| {
        let trimmed = ln.trim();
        let injected = trimmed.trim_start_matches("//INJECT//");
        if injected.len() < trimmed.len() {
            return injected;
        }
        if let Some((_, after)) = trimmed.split_once("//REPLACE//") {
            return after;
        }
        return trimmed;
    }).collect::<Vec<_>>().join("\n");
    let module = match parser.parse(&options, &src_transformed) {
        Ok(v) => v,
        Err(err) => {
            println!("Err {}", err.len());
            for err in err {
                let loc = err.meta.location(src);
                let s = String::from_utf8(src.as_bytes()[loc.offset as usize ..][..loc.length as usize].to_vec()).unwrap();
                println!("GLSL Compilation Error [{}:{}]: {}\n{}",loc.line_number,loc.line_position, err.kind.to_string(), s);
                log::warn!("GLSL Compilation Error [{}:{}]: {}\n{}",loc.line_number,loc.line_position, err.kind.to_string(), s);
            }
            panic!("shader compilation failed.");
        }
    };

    ShaderModuleDescriptor { label, source: wgpu::ShaderSource::Naga(Cow::Owned(module)) }
}

#[macro_export]
macro_rules! include_glsl {
    ($name: tt, $stage: expr) => {
        $crate::util::include_glsl::include_glsl_fn(Some($name), include_str!($name), $stage)
    };
}