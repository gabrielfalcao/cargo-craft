use crate::helpers::{
    into_acceptable_package_name, to_pascal_case, valid_crate_name,
};
use clap::Parser;
use std::cmp::PartialEq;
use std::fmt::{Display, Formatter};
use toml::{Table, Value};


#[derive(Parser, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Dependency {
    #[arg(value_parser = valid_crate_name)]
    pub name: String,

    #[arg(short = 'F', long)]
    pub features: Option<String>,

    #[arg(long, conflicts_with = "build")]
    pub dev: bool,

    #[arg(long)]
    pub build: bool,

    #[arg(long)]
    pub optional: bool,
}
impl Display for Dependency {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let mut args = vec![self.name.to_string()];
        if self.dev {
            args.push("--dev".to_string());
        }
        if self.build {
            args.push("--build".to_string());
        }
        if self.optional {
            args.push("--optional".to_string());
        }
        let features = self.features();
        if features.len() > 0 {
            args.push(format!("-F{}", features.join(",")));
        }
        write!(f, "{}", args.join(" "))
    }
}

impl Dependency {
    pub fn features(&self) -> Vec<String> {
        let mut features = Vec::<String>::new();
        for h in self
            .features
            .clone()
            .unwrap_or_default()
            .split(",")
            .filter(|h| h.trim().len() > 0)
        {
            features.push(h.to_string());
        }
        features
    }
    pub fn to_tera(&self) -> Table {
        let mut dep = Table::new();
        dep.insert("name".to_string(), Value::String(self.name.to_string()));
        dep.insert(
            "package_name".to_string(),
            Value::String(into_acceptable_package_name(self.name.as_str())),
        );
        dep.insert(
            "features".to_string(),
            Value::Array(
                self.features()
                    .iter()
                    .map(|h| Value::String(h.to_string()))
                    .collect(),
            ),
        );
        dep.insert("pascal_case".to_string(), Value::String(self.pascal_name()));
        dep.insert("dev".to_string(), Value::Boolean(self.dev));
        dep.insert("build".to_string(), Value::Boolean(self.build));
        dep
    }
    pub fn pascal_name(&self) -> String {
        to_pascal_case(self.name.as_str())
    }
}
