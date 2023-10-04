use spirv_builder::{SpirvBuilder, Capability};

fn main() {
    SpirvBuilder::new("kernel", "spirv-unknown-vulkan1.1")
        .capability(Capability::Int8)
        .build()
        .expect("Shader failed to compile");

    /*for shader in std::fs::read_dir("shaders").expect("Error finding shaders folder") {
        let path = shader.expect("Invalid path in ahders folder").path();
        SpirvBuilder::new(path, "spirv-unknown-vulkan1.1")
            .build()
            .expect("Shader failed to compile");
    }*/
}
