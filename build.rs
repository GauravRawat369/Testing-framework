use tonic_build;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        // .out_dir("src/generated_code")
        .compile_protos(&[
            "proto/health_check.proto",
            "proto/success_rate.proto"
        ], &["proto"])
        .expect("Failed to compile proto file");

    Ok(())
}