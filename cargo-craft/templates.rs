use crate::cli::Craft;
use crate::Result;
use tera::{Context, Tera};

pub fn tera(craft: &Craft) -> Result<(Tera, Context)> {
    let mut tera = Tera::default();
    tera.add_raw_template("errors.rs", include_str!("./templates/errors.rs.tera"))?;
    tera.add_raw_template("lib.rs", include_str!("./templates/lib.rs.tera"))?;
    tera.add_raw_template("cli.rs", include_str!("./templates/lib_cli.rs.tera"))?;
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
    context.insert("crate_version", &craft.version());
    context.insert("package_name", &craft.package_name());
    context.insert("struct_name", &craft.struct_name());
    context.insert("craft_lib", &true);
    context.insert("craft_cli", &craft.cli);
    context.insert("crate_binaries", &craft.bin_entries());
    context.insert("crate_lib", &craft.lib_entry("lib.rs"));
    context.insert("craft_value_enum", &(craft.cli && craft.value_enum));
    context.insert("craft_subcommands", &(craft.cli && craft.subcommands));
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
    let rendered = tera
        .render(template_name, &context)
        .expect(&format!("render {}", template_name));
    Ok(Some(rendered))
}
pub fn render_cli(craft: &Craft) -> Result<Option<String>> {
    if craft.cli {
        Ok(render(craft, "cli")?)
    } else {
        Ok(None)
    }
}
