use toml::Table;

pub fn extend_table(from: &Table, into: &Table) -> Table {
    let mut extended = into.clone();
    for (k, v) in from.iter() {
        extended.insert(k.clone(), v.clone().into());
    }
    extended
}
