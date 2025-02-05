use iocore::Path;
use regex::Regex;
use toml::{Table, Value};

pub fn extend_table(from: &Table, into: &Table) -> Table {
    let mut extended = into.clone();
    for (k, v) in from.iter() {
        extended.insert(k.clone(), v.clone().into());
    }
    extended
}

pub fn slug(text: &str, sep: Option<&str>) -> String {
    let re = Regex::new("[^a-zA-Z0-9_-]").unwrap();
    re.replace_all(text, sep.unwrap_or("-").to_string())
        .to_string()
}
pub fn acceptable_crate_name(val: &str) -> ::std::result::Result<String, String> {
    let re = Regex::new(r"^[a-z]+([-][a-z0-9]+|[a-z0-9]+)+$").unwrap();
    if re.is_match(val) {
        Ok(val.to_string())
    } else {
        Err(format!(
            "{:#?} does not appear to be a valid crate name",
            val
        ))
    }
}
pub fn valid_package_name(val: &str) -> ::std::result::Result<String, String> {
    let re = Regex::new(r"^[a-z]+([_][a-z0-9]+|[a-z0-9]+)+$").unwrap();
    if re.is_match(val) {
        Ok(val.to_string())
    } else {
        Err(format!("{:#?} is not a valid package name", val))
    }
}
pub fn path_to_entry_path(entry: Option<Table>) -> Option<Path> {
    match entry?.get("path")? {
        Value::String(path) => Some(Path::new(path)),
        _ => None,
    }
}

pub fn valid_manifest_path(val: &str) -> ::std::result::Result<Path, String> {
    let val = acceptable_crate_name(val)?;
    let path = Path::new(val);
    let path = if path.name() == "Cargo.toml" && !path.is_dir() {
        path.parent().expect(&format!("parent of {}", &path))
    } else {
        path.clone()
    };
    let manifest_path = path.join("Cargo.toml");
    if manifest_path.try_canonicalize().exists() && !manifest_path.is_dir() {
        return Err(format!("{} already exists", &manifest_path));
    }
    Ok(path)
}

pub fn crate_name_from_path(path: impl Into<Path>) -> ::std::result::Result<String, String> {
    let name = path.into().without_extension().name();
    let crate_name = into_acceptable_crate_name(&name);
    Ok(crate_name)
}
pub fn package_name_from_string_or_path(
    name: Option<String>,
    path: impl Into<Path>,
) -> ::std::result::Result<String, String> {
    let name = match name {
        Some(name) => name,
        None => crate_name_from_path(path)?,
    };
    let package_name = into_acceptable_package_name(&name);
    Ok(package_name)
}

pub fn into_acceptable_crate_name(val: &str) -> String {
    into_acceptable_name(val, '-')
}

pub fn into_acceptable_package_name(val: &str) -> String {
    into_acceptable_name(val, '_')
}

pub fn into_acceptable_name(val: &str, sep: char) -> String {
    let val = val.to_lowercase();
    let re = Regex::new(r"^[^a-z]+").unwrap();
    let val = re.replace_all(&val, String::new()).to_string();
    let re = Regex::new(r"[^a-z0-9]$").unwrap();
    let val = re.replace_all(&val, String::new()).to_string();
    let re = Regex::new(&format!(r"[{}]+", sep)).unwrap();
    let val = re.replace_all(&val, String::from(sep)).to_string();
    let re = Regex::new(&format!(r"[^a-zA-Z0-9{}]", sep)).unwrap();
    let val = re.replace_all(&val, String::from(sep)).to_string();
    val
}
