use cargo_craft::cli::{path_to_entry_path, Craft};
use cargo_craft::templates::{render_cli, render_errors, render_lib};
use clap::Parser;
use iocore::Path;
use toml::Table;

fn main() {
    let args = Craft::parse();

    let mut manifest = args.cargo_manifest();
    let manifest_string = manifest.to_string_pretty().unwrap();
    let manifest_path = args.manifest_path();
    manifest_path
        .write(&manifest_string.into_bytes())
        .unwrap();
    println!("wrote {}", manifest_path);

    manifest.set_lib_entry(args.lib_entry());
    manifest.set_bin_entries(args.bin_entries());

    for (template, target) in vec![
        (render_lib(&args), vec![args.lib_entry()]),
        (render_errors(&args), vec![args.errors_entry()]),
        (
            render_cli(&args),
            args.bin_entries()
                .iter()
                .map(|entry| Some(entry.clone()))
                .collect::<Vec<Option<Table>>>(),
        ),
    ] {
        for target in target
            .iter()
            .filter(|entry| entry.is_some())
            .map(|entry| path_to_entry_path(entry.clone()))
            .filter(|path| path.is_some())
            .map(|path| path.unwrap())
            .collect::<Vec<Path>>()
        {
            match args.path_to(target).write(&template.clone().unwrap().as_bytes()) {
                Ok(path) => {
                    println!("wrote {}", path);
                },
                Err(err) => panic!("{}", err),
            }
        }
    }
}
