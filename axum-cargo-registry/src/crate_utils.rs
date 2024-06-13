pub fn crate_name_to_index(crate_name: &str) -> String {
    match crate_name.len() {
        1 => format!("1/{}", crate_name),
        2 => format!("2/{}", crate_name),
        3 => format!("3/{}/{}", &crate_name[..1], crate_name),
        _ => format!("{}/{}/{}", &crate_name[..2], &crate_name[2..4], crate_name),
    }
}
