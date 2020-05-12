//! Shader handling

/// Safety: Malformed shaders or shaders with incorrect type may be unsafe.
///
/// This isn't actually unsafe, what's unsafe is using bad shaders, but since bad shaders come
/// from here, we mark it as unsafe nonetheless.
pub(crate) unsafe fn compile_glsl_to_spirv<S: AsRef<str>>(
    source: S,
    file_name: S,
    shader_type: ShaderType,
) -> Shader {
    warn_incorrect_shader_type(&file_name, shader_type);

    let mut compiler = shaderc::Compiler::new().unwrap();
    let options = shaderc::CompileOptions::new().unwrap();

    let shader_kind = shader_type.get_shaderc_shader_kind();

    let binary_result = compiler
        .compile_into_spirv(
            source.as_ref(),
            shader_kind,
            file_name.as_ref(),
            "main",
            Some(&options),
        )
        .unwrap();

    Shader {
        shader_type,
        binary: binary_result.as_binary_u8().to_vec(),
    }
}

/// Debugging utility function
#[allow(dead_code)]
pub(crate) fn compile_glsl_to_spirv_asm<S: AsRef<str>>(
    source: S,
    file_name: S,
    shader_type: ShaderType,
) -> String {
    warn_incorrect_shader_type(&file_name, shader_type);

    let mut compiler = shaderc::Compiler::new().unwrap();
    let options = shaderc::CompileOptions::new().unwrap();

    let shader_kind = shader_type.get_shaderc_shader_kind();

    let asm_result = compiler
        .compile_into_spirv_assembly(
            source.as_ref(),
            shader_kind,
            file_name.as_ref(),
            "main",
            Some(&options),
        )
        .unwrap();

    asm_result.as_text()
}

/// Warns if shader file name extension doesn't match the shader type. Should help prevent
/// bugs involving incorrect/malformed shaders.
fn warn_incorrect_shader_type<S: AsRef<str>>(file_name: &S, shader_type: ShaderType) {
    let file_ext = std::path::Path::new(file_name.as_ref())
        .extension()
        .and_then(std::ffi::OsStr::to_str);

    match file_ext {
        None => log::warn!(
            "Shader file name (\"{}\") doesn't have extension.\
             Make sure you are using the correct ShaderType ( You are using {:?})",
            file_name.as_ref(),
            shader_type
        ),
        Some(ext) => {
            if ext != expected_extension(shader_type) {
                log::warn!(
                    "Shader file name (\"{}\") has extension \"{}\", but ShaderType is {:?} \
                     (expects extension \"{}\"). Make sure you are using the correct ShaderType.",
                    file_name.as_ref(),
                    ext,
                    shader_type,
                    expected_extension(shader_type)
                );
            };
        }
    };
}

/// Internal function used for debugging purposes
fn expected_extension(shader_type: ShaderType) -> &'static str {
    match shader_type {
        ShaderType::Vertex => "vert",
        ShaderType::Fragment => "frag",
    }
}

/// Represents a type of shader (e.g Vertex, Fragment)
#[derive(Copy, Clone, Debug)]
pub enum ShaderType {
    Vertex,
    Fragment,
}

impl ShaderType {
    fn get_shaderc_shader_kind(&self) -> shaderc::ShaderKind {
        match self {
            ShaderType::Vertex => shaderc::ShaderKind::Vertex,
            ShaderType::Fragment => shaderc::ShaderKind::Fragment,
        }
    }
}

pub struct Shader {
    shader_type: ShaderType,
    binary: Vec<u8>,
}

impl Shader {
    #[allow(dead_code)]
    pub(crate) fn shader_type(&self) -> ShaderType {
        self.shader_type
    }

    pub(crate) fn as_ref(&self) -> &[u8] {
        self.binary.as_ref()
    }
}
