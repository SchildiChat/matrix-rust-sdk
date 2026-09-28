use anyhow::Result;
use camino::Utf8Path;
use uniffi_bindgen::bindings::{GenerateOptions, TargetLanguage, generate};

pub(crate) fn generate_android_bindings(
    library_path: &Utf8Path,
    output_dir: &Utf8Path,
) -> Result<()> {
    let config_override = Utf8Path::new(env!("CARGO_MANIFEST_DIR")).join("sc-android-uniffi.toml");

    generate(GenerateOptions {
        languages: vec![TargetLanguage::Kotlin],
        source: library_path.to_path_buf(),
        out_dir: output_dir.to_path_buf(),
        config_override: Some(config_override),
        ..GenerateOptions::default()
    })
}
