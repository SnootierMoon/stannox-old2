use std::io::Write;

const IN_DIRNAME: &str = "assets/shaders";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed={}", IN_DIRNAME);
    let out_dirname = std::env::var("OUT_DIR")?;

    std::fs::create_dir_all(&out_dirname)?;

    let mut compiler = shaderc::Compiler::new().expect("Failed to create shaderc compiler");

    for entry in std::fs::read_dir(IN_DIRNAME)? {
        let entry = entry?;
        let in_path = entry.path();
        let in_filename = in_path.file_name().unwrap().to_string_lossy();
        let kind = match in_path.extension() {
            Some(ext) => match ext.to_string_lossy().as_ref() {
                "vert" => shaderc::ShaderKind::Vertex,
                "frag" => shaderc::ShaderKind::Fragment,
                _ => continue,
            },
            None => continue,
        };
        let out_path = format!("{}/{}.spv", out_dirname, in_filename);
        println!(
            "Compiling {} into {}",
            in_path.to_str().expect("Invalid path"),
            &out_path
        );

        let source = std::fs::read_to_string(&in_path)?;

        let spirv = compiler.compile_into_spirv(&source, kind, &in_filename, "main", None)?;

        std::fs::File::create(&out_path)?.write(spirv.as_binary_u8())?;
    }
    Ok(())
}
