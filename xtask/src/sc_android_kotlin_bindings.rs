use anyhow::{Result, bail};
use camino::Utf8Path;
use uniffi_bindgen::{
    BindgenPaths, BindgenPathsLayer,
    bindings::{GenerateOptions, TargetLanguage, generate_with_bindgen_paths},
    cargo_metadata::CrateConfigSupplier,
};

pub(crate) fn generate_android_bindings(
    library_path: &Utf8Path,
    output_dir: &Utf8Path,
) -> Result<()> {
    generate_with_bindgen_paths(
        GenerateOptions {
            languages: vec![TargetLanguage::Kotlin],
            source: library_path.to_path_buf(),
            out_dir: output_dir.to_path_buf(),
            ..GenerateOptions::default()
        },
        android_bindgen_paths()?,
    )
}

fn android_bindgen_paths() -> Result<BindgenPaths> {
    let mut paths = BindgenPaths::default();
    paths.add_layer(AndroidConfigLayer(CrateConfigSupplier::from_cargo_metadata_command(false)?));
    Ok(paths)
}

struct AndroidConfigLayer(CrateConfigSupplier);

impl BindgenPathsLayer for AndroidConfigLayer {
    fn get_config(&self, crate_name: &str) -> Result<Option<toml::Table>> {
        let Some(mut config) = self.0.get_config(crate_name)? else {
            return Ok(None);
        };

        let Some(bindings) = config.get_mut("bindings").and_then(toml::Value::as_table_mut) else {
            bail!("UniFFI config for {crate_name} has no bindings table");
        };
        let Some(kotlin) = bindings.get_mut("kotlin").and_then(toml::Value::as_table_mut) else {
            bail!("UniFFI config for {crate_name} has no Kotlin bindings table");
        };
        kotlin.insert("android_cleaner".to_owned(), true.into());

        Ok(Some(config))
    }
}
