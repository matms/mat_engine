lazy_static::lazy_static! {
    static ref COMPILED_DEFAULT_VERT_SHADER: Shader = {
        unsafe{
        compile_glsl_to_spirv(
            include_str!("default_shaders/shader.vert"),
            "shader.vert",
            ShaderType::Vertex)
        }
    };

    static ref COMPILED_DEFAULT_FRAG_SHADER: Shader = {
        unsafe{
        compile_glsl_to_spirv(
            include_str!("default_shaders/shader.frag"),
            "shader.frag",
            ShaderType::Fragment)
        }
    };
}

pub(crate) fn default_vert_shader() -> &'static Shader {
    &*COMPILED_DEFAULT_VERT_SHADER
}

pub(crate) fn default_frag_shader() -> &'static Shader {
    &*COMPILED_DEFAULT_FRAG_SHADER
}

/// Safety: Malformed shaders or shaders with incorrect type may be unsafe.
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

    let shader_kind = match shader_type {
        ShaderType::Vertex => shaderc::ShaderKind::Vertex,
        ShaderType::Fragment => shaderc::ShaderKind::Fragment,
    };

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
pub(crate) fn compile_glsl_to_spirv_asm<S: AsRef<str>>(
    source: S,
    file_name: S,
    shader_type: ShaderType,
) -> String {
    warn_incorrect_shader_type(&file_name, shader_type);

    let mut compiler = shaderc::Compiler::new().unwrap();
    let options = shaderc::CompileOptions::new().unwrap();

    let shader_kind = match shader_type {
        ShaderType::Vertex => shaderc::ShaderKind::Vertex,
        ShaderType::Fragment => shaderc::ShaderKind::Fragment,
    };

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

#[derive(Copy, Clone, Debug)]
pub enum ShaderType {
    Vertex,
    Fragment,
}

pub struct Shader {
    shader_type: ShaderType,
    binary: Vec<u8>,
}

impl Shader {
    pub(crate) fn shader_type(&self) -> ShaderType {
        self.shader_type
    }

    pub(crate) fn as_ref(&self) -> &[u8] {
        self.binary.as_ref()
    }
}
