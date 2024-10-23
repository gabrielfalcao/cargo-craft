use toml::{Table, Value};

pub struct CargoManifest {
    manifest: Table,
}

impl CargoManifest {
    pub fn from_str(manifest: &str) -> Result<CargoManifest, String> {
        let manifest: Table = toml::from_str(manifest).map_err(|err| err.to_string())?;
        Ok(CargoManifest { manifest })
    }
    pub fn set_package_key_value(&mut self, key: impl Into<String>, value: impl Into<Value>) {
        let package_entry = self
            .manifest
            .get_mut("package")
            .expect("manifest to contain the [package] entry")
            .as_table_mut()
            .expect("package entry to be a toml::Table");
        package_entry.insert(key.into(), value.into());
    }

    pub fn set_lib_entry(&mut self, entry: Option<Table>) {
        let package_entry = self
            .manifest
            .get_mut("package")
            .expect("manifest to contain the [package] entry")
            .as_table_mut()
            .expect("package entry to be a toml::Table");
        if let Some(entry) = entry {
            package_entry.insert("lib".to_string(), Value::Table(entry.clone()));
        }
    }
    pub fn set_bin_entries(&mut self, bin_entries: Vec<Table>) {
        let package_entry = self
            .manifest
            .get_mut("package")
            .expect("manifest to contain the [package] entry")
            .as_table_mut()
            .expect("package entry to be a toml::Table");
        if bin_entries.len() > 0 {
            package_entry.insert(
                "bin".to_string(),
                Value::Array(
                    bin_entries
                        .iter()
                        .map(|entry| Value::Table(entry.clone()))
                        .collect::<Vec<_>>(),
                ),
            );
        }
    }
    pub fn to_string_pretty(&self) -> Result<String, String> {
        Ok(toml::to_string_pretty(&self.manifest).map_err(|err| err.to_string())?)
    }
}

impl Default for CargoManifest {
    fn default() -> CargoManifest {
        CargoManifest::from_str(
            r#"cargo-features = ["edition2024"]

[package]
name = ""
version = "0.1.0"
edition = "2021"
autobins = false
autoexamples = false
autobenches = false

[package.metadata]
cargo-args = ["-Zmtime-on-use", "-Zavoid-dev-deps"]

[dependencies]
"#,
        )
        .unwrap()
    }
}
