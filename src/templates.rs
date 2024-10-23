use tera::{Tera, Context};
use crate::cli::Craft;


pub fn tera(craft: &Craft) -> (Tera, Context) {
    let mut tera = Tera::default();
    tera.add_raw_template("errors", include_str!("./templates/errors.rs.tera")).unwrap();
    tera.add_raw_template("lib", include_str!("./templates/lib.rs.tera")).unwrap();
    tera.add_raw_template("cli", include_str!("./templates/cli.rs.tera")).unwrap();

    let mut context = Context::new();
    context.insert("package_name", &craft.name());
    context.insert("craft_lib", &craft.lib);
    context.insert("craft_cli", &craft.cli);
    context.insert("craft_binaries", &craft.bin_entries());
    context.insert("craft_binaries", &craft.bin_entries());
    context.insert("manifest", &craft.cargo_manifest());
    (tera, context)
}

pub fn render_errors(craft: &Craft) -> Option<String> {
    if craft.lib {
        let (tera, context) = tera(craft);
        let rendered = tera.render("errors", &context).unwrap();
        Some(rendered)
    } else {
        None
    }
}
pub fn render_lib(craft: &Craft) -> Option<String> {
    if craft.lib {
        let (tera, context) = tera(craft);
        let rendered = tera.render("lib", &context).unwrap();
        Some(rendered)
    } else {
        None
    }
}

pub fn render_cli(craft: &Craft) -> Option<String> {
    if craft.cli {
        let (tera, context) = tera(craft);
        let rendered = tera.render("cli", &context).unwrap();
        Some(rendered)
    } else {
        None
    }
}
