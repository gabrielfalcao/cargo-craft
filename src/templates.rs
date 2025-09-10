use crate::cli::Craft;
use crate::Result;
use crate::helpers::to_pascal_case;
use tera::{Context, Tera};
use toml::{Table, Value};

pub fn tera_info(craft: &Craft) -> Result<(Tera, Context)> {
    let mut tera = Tera::default();
    tera.add_raw_template("errors.rs", include_str!("./templates/errors.rs.tera"))?;
    tera.add_raw_template(
        "bare.main.rs",
        include_str!("./templates/bare.main.rs.tera"),
    )?;
    tera.add_raw_template(
        "bare.mod.cli.rs",
        include_str!("./templates/bare.mod.cli.rs.tera"),
    )?;
    tera.add_raw_template("lib.rs", include_str!("./templates/lib.rs.tera"))?;
    tera.add_raw_template("dispatch.rs", include_str!("./templates/lib_dispatch.rs.tera"))?;
    tera.add_raw_template(
        "{{package_name}}.rs",
        include_str!("./templates/{{package_name}}.rs.tera"),
    )?;
    tera.add_raw_template("cli", include_str!("./templates/cli.rs.tera"))?;
    tera.add_raw_template("Cargo.toml", include_str!("./templates/Cargo.toml.tera"))?;
    tera.add_raw_template(".gitignore", include_str!("./templates/gitignore.tera"))?;
    tera.add_raw_template(
        ".rustfmt.toml",
        include_str!("./templates/rustfmt.toml.tera"),
    )?;
    tera.add_raw_template(
        "rust-toolchain.toml",
        include_str!("./templates/rust-toolchain.toml.tera"),
    )?;
    tera.add_raw_template("README.md", include_str!("./templates/README.md.tera"))?;

    let mut context = Context::new();
    context.insert("crate_name", &craft.crate_name());
    context.insert("is_cargo_command", &craft.crate_name().starts_with("cargo-"));
    context.insert("crate_version", &craft.version());
    context.insert("package_name", &craft.package_name());
    context.insert(
        "package_description",
        &craft.description.clone().unwrap_or_default(),
    );
    context.insert("struct_name", &craft.struct_name());
    context.insert("craft_lib", &true);
    context.insert("craft_cli", &craft.is_cli());
    context.insert("crate_path", &craft.path());
    context.insert("lib_path", &craft.lib_path());
    Ok((tera, context))
}

pub fn tera(craft: &Craft) -> Result<(Tera, Context)> {
    let (tera, mut context) = tera_info(craft)?;
    let subcommands = craft
        .subcommand_names()
        .iter()
        .map(|name| {
            let mut case_variants = Table::new();
            case_variants.insert("name".to_string(), Value::String(name.to_string()));
            case_variants.insert("lowercase".to_string(), Value::String(name.to_lowercase().to_string()));
            case_variants.insert("uppercase".to_string(), Value::String(name.to_uppercase().to_string()));
            case_variants.insert("pascalcase".to_string(), Value::String(to_pascal_case(&name)));
            case_variants
        })
        .collect::<Vec<Table>>();
    context.insert("crate_binaries", &craft.bin_entries());
    context.insert("crate_lib", &craft.lib_entry("lib.rs"));
    context.insert("craft_value_enum", &(craft.is_cli() && craft.value_enum));
    context.insert("craft_subcommands", &(subcommands.len() > 0));
    context.insert("subcommands", &subcommands);
    context.insert(
        "craft_dependencies",
        &craft
            .deps()?
            .iter()
            .map(|dep| dep.to_tera())
            .collect::<Vec<toml::Table>>(),
    );
    context.insert("craft_errors", &craft.error_types());
    Ok((tera, context))
}
pub fn render(craft: &Craft, template_name: &str) -> Result<Option<String>> {
    let (tera, context) = tera(craft)?;
    let rendered = tera.render(template_name, &context)?;
    // .expect(&format!("render {}", template_name));
    Ok(Some(rendered))
}
pub fn render_info_string(craft: &Craft, template: &str) -> Result<String> {
    let (mut tera, context) = tera_info(craft)?;
    let rendered = tera.render_str(template, &context)?;
    Ok(rendered)
}
pub fn render_cli(craft: &Craft) -> Result<Option<String>> {
    if craft.is_cli() {
        Ok(render(craft, "cli")?)
    } else {
        Ok(None)
    }
}
