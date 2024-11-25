use crate::cli::Craft;
use tera::{Context, Tera};

pub fn tera(craft: &Craft) -> (Tera, Context) {
    let mut tera = Tera::default();
    tera.add_raw_template("errors.rs", include_str!("./templates/errors.rs.tera"))
        .unwrap();
    tera.add_raw_template("lib.rs", include_str!("./templates/lib.rs.tera"))
        .unwrap();
    tera.add_raw_template("cli", include_str!("./templates/cli.rs.tera"))
        .unwrap();
    tera.add_raw_template("Cargo.toml", include_str!("./templates/Cargo.toml.tera"))
        .unwrap();
    tera.add_raw_template(".gitignore", include_str!("./templates/gitignore.tera"))
        .unwrap();

    let mut context = Context::new();
    context.insert("crate_name", &craft.crate_name());
    context.insert("crate_version", &craft.version());
    context.insert("package_name", &craft.package_name());
    context.insert("craft_lib", &true);
    context.insert("craft_cli", &craft.cli);
    context.insert("crate_binaries", &craft.bin_entries());
    context.insert("crate_lib", &craft.lib_entry());
    context.insert("craft_value_enum", &craft.value_enum);
    (tera, context)
}
pub fn render(craft: &Craft, template_name: &str) -> Option<String> {
    let (tera, context) = tera(craft);
    let rendered = tera
        .render(template_name, &context)
        .expect(&format!("render {}", template_name));
    Some(rendered)
}
pub fn render_cli(craft: &Craft) -> Option<String> {
    if craft.cli {
        render(craft, "cli")
    } else {
        None
    }
}
