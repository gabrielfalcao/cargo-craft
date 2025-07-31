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

/// `slug` slugifies strings
///
/// Example:
/// ```
/// use cargo_craft::slug;
/// assert_eq!(slug(" crate name ", None), "crate-name");
/// ```
pub fn slug(text: &str, sep: Option<&str>) -> String {
    let re = Regex::new("[^a-zA-Z0-9_-]").unwrap();
    let sep = sep.unwrap_or("-").to_string();
    strip_ends(&re.replace_all(text, sep.as_str()), sep.as_str())
}
/// `strip_ends` strips suffix and prefix from string
///
/// Example:
/// ```
/// use cargo_craft::strip_ends;
/// assert_eq!(strip_ends("-crate-name-", "-"), "crate-name");
/// ```
pub fn strip_ends(text: &str, sep: &str) -> String {
    let text = text.to_string();
    let text = text
        .strip_prefix(sep)
        .map(|text| text.to_string())
        .unwrap_or_else(|| text.to_string());
    let text = text
        .strip_suffix(sep)
        .map(|text| text.to_string())
        .unwrap_or_else(|| text.to_string());
    text
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

pub fn package_name_from_string_or_path<T: std::fmt::Display>(
    name: Option<T>,
    path: impl Into<Path>,
) -> ::std::result::Result<String, String> {
    let name = match name {
        Some(name) => name.to_string(),
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

/// `capitalize_string`
///
/// Example
/// ```
/// use cargo_craft::capitalize_string;
/// assert_eq!(capitalize_string(""), "");
/// assert_eq!(capitalize_string("g"), "G");
/// assert_eq!(capitalize_string("string"), "String");
/// ```
pub fn capitalize_string(val: impl std::fmt::Display) -> String {
    let val = val.to_string();
    match val.len() {
        0 => val,
        1 => val.to_uppercase(),
        _ => format!("{}{}", val[0..1].to_uppercase(), val[1..].to_string()),
    }
}
pub fn struct_name_from_package_name(val: impl std::fmt::Display) -> String {
    let val = val.to_string();
    let package_name = into_acceptable_package_name(&val);
    package_name
        .split("_")
        .map(|part| capitalize_string(part))
        .collect::<String>()
}

pub fn valid_crate_name(val: &str) -> ::std::result::Result<String, String> {
    Ok(
        acceptable_crate_name(into_acceptable_name(val, '-').as_str())
            .map_err(|_| format!("{:#?} is not a valid crate name", val))?,
    )
}
pub fn to_pascal_case(val: impl std::fmt::Display) -> String {
    let pattern = regex::Regex::new(r"\W+").unwrap();
    pattern
        .split(&val.to_string())
        .map(|h| capitalize_string(h.to_string()))
        .collect()
}
pub fn words(val: impl std::fmt::Display) -> Vec<String> {
    let pattern = regex::Regex::new(r"\b\W+\b").unwrap();
    pattern.find_iter(val.to_string().as_str()).map(|h|h.as_str().to_string()).collect()
}
pub fn into_acceptable_error_type_name(val: &str) -> String {
    let pattern = regex::Regex::new(r"(?i)^(?<name>.*)(?:Error)?$").unwrap();
    words(pattern.replace_all(val, "$name")).iter().map(|h|capitalize_string(h)).collect::<Vec<String>>().join("")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_acceptable_crate_name() {
        assert_eq!(
            acceptable_crate_name("package_name"),
            Err(String::from(
                "\"package_name\" does not appear to be a valid crate name"
            ))
        );
    }
    #[test]
    fn test_valid_package_name() {
        assert_eq!(
            valid_package_name("crate-name"),
            Err(String::from("\"crate-name\" is not a valid package name"))
        );
    }
    #[test]
    fn test_package_name_from_string_or_path() -> Result<(), String> {
        assert_eq!(
            package_name_from_string_or_path(Some("package-name"), "path-to-package-name")?,
            "package_name"
        );
        assert_eq!(
            package_name_from_string_or_path(None::<String>, "package-name")?,
            "package_name"
        );
        Ok(())
    }
    #[test]
    fn test_struct_name_from_package_name() -> Result<(), String> {
        let struct_name = struct_name_from_package_name("package_name");
        assert_eq!(struct_name, "PackageName");
        Ok(())
    }
}
