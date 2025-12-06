
mod helpers;
mod i18n;

use crate::helpers::env_opt;

fn main() {
    compile_i18n();
}

fn compile_i18n() {
    let i18n_path = "../../../i18n/edit.toml";

    let i18n = std::fs::read_to_string(i18n_path).unwrap();
    let contents = i18n::generate(&i18n);
    let out_dir = env_opt("OUT_DIR");
    let path = format!("{out_dir}/i18n_edit.rs");
    std::fs::write(&path, contents).unwrap();

    println!("cargo::rerun-if-env-changed=EDIT_CFG_LANGUAGES");
    println!("cargo::rerun-if-changed={i18n_path}");
}
