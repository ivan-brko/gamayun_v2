fn main() -> Result<(), Box<dyn std::error::Error>> {
    handle_protos()
}

fn handle_protos() -> Result<(), Box<dyn std::error::Error>> {
    let include_paths: [&str; 1] = ["proto/"];

    let protos = ["proto/result_reporting_service.proto"];

    let config = tonic_build::configure()
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .type_attribute(".", "#[serde(rename_all = \"camelCase\")]");

    config.clone().compile_protos(&protos, &include_paths)?;
    Ok(())
}
