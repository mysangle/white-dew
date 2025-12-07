
mod helpers;
mod i18n;

use crate::helpers::env_opt;

#[derive(Clone, Copy, Eq, PartialEq)]
enum TargetOs {
    //Windows,
    MacOS,
    Unix,
}

fn main() {
    stdext::arena::init(128 * 1024 * 1024).unwrap();

    let target_os = match env_opt("CARGO_CFG_TARGET_OS").as_str() {
        "windows" => panic!("`windows` is not supported"),
        "macos" | "ios" => TargetOs::MacOS,
        _ => TargetOs::Unix,
    };

    compile_i18n();
    configure_icu(target_os);
}

fn compile_i18n() {
    let i18n_path = "../../i18n/edit.toml";

    let i18n = std::fs::read_to_string(i18n_path).unwrap();
    let contents = i18n::generate(&i18n);
    let out_dir = env_opt("OUT_DIR");
    let path = format!("{out_dir}/i18n_edit.rs");
    std::fs::write(&path, contents).unwrap();

    // 환경 변수 EDIT_CFG_LANGUAGES 값이 바뀌면 build.rs를 다시 실행해라.
    println!("cargo::rerun-if-env-changed=EDIT_CFG_LANGUAGES");
    // i18n_path 경로의 파일이 변경되면 build.rs를 다시 실행해라.
    println!("cargo::rerun-if-changed={i18n_path}");
}

fn configure_icu(target_os: TargetOs) {
    let icuuc_soname = env_opt("EDIT_CFG_ICUUC_SONAME");
    let icui18n_soname = env_opt("EDIT_CFG_ICUI18N_SONAME");
    let cpp_exports = env_opt("EDIT_CFG_ICU_CPP_EXPORTS");
    let renaming_version = env_opt("EDIT_CFG_ICU_RENAMING_VERSION");
    let renaming_auto_detect = env_opt("EDIT_CFG_ICU_RENAMING_AUTO_DETECT");

    // If none of the `EDIT_CFG_ICU*` environment variables are set,
    // we default to enabling `EDIT_CFG_ICU_RENAMING_AUTO_DETECT` on UNIX.
    // This slightly improves portability at least in the cases where the SONAMEs match our defaults.
    let renaming_auto_detect = if !renaming_auto_detect.is_empty() {
        renaming_auto_detect.parse::<bool>().unwrap()
    } else {
        target_os == TargetOs::Unix
            && icuuc_soname.is_empty()
            && icui18n_soname.is_empty()
            && cpp_exports.is_empty()
            && renaming_version.is_empty()
    };
    if renaming_auto_detect && !renaming_version.is_empty() {
        // It makes no sense to specify an explicit version and also ask for auto-detection.
        panic!(
            "Either `EDIT_CFG_ICU_RENAMING_AUTO_DETECT` or `EDIT_CFG_ICU_RENAMING_VERSION` must be set, but not both"
        );
    }

    let icuuc_soname = if !icuuc_soname.is_empty() {
        &icuuc_soname
    } else {
        match target_os {
            //TargetOs::Windows => "icuuc.dll",
            TargetOs::MacOS => "libicucore.dylib",
            TargetOs::Unix => "libicuuc.so",
        }
    };
    let icui18n_soname = if !icui18n_soname.is_empty() {
        &icui18n_soname
    } else {
        match target_os {
            //TargetOs::Windows => "icuin.dll",
            TargetOs::MacOS => "libicucore.dylib",
            TargetOs::Unix => "libicui18n.so",
        }
    };
    let icu_export_prefix =
        if !cpp_exports.is_empty() && cpp_exports.parse::<bool>().unwrap() { "_" } else { "" };
    let icu_export_suffix =
        if !renaming_version.is_empty() { format!("_{renaming_version}") } else { String::new() };

    println!("cargo::rerun-if-env-changed=EDIT_CFG_ICUUC_SONAME");
    println!("cargo::rustc-env=EDIT_CFG_ICUUC_SONAME={icuuc_soname}");
    println!("cargo::rerun-if-env-changed=EDIT_CFG_ICUI18N_SONAME");
    println!("cargo::rustc-env=EDIT_CFG_ICUI18N_SONAME={icui18n_soname}");
    println!("cargo::rerun-if-env-changed=EDIT_CFG_ICU_EXPORT_PREFIX");
    println!("cargo::rustc-env=EDIT_CFG_ICU_EXPORT_PREFIX={icu_export_prefix}");
    println!("cargo::rerun-if-env-changed=EDIT_CFG_ICU_EXPORT_SUFFIX");
    println!("cargo::rustc-env=EDIT_CFG_ICU_EXPORT_SUFFIX={icu_export_suffix}");
    println!("cargo::rerun-if-env-changed=EDIT_CFG_ICU_RENAMING_AUTO_DETECT");
    println!("cargo::rustc-check-cfg=cfg(edit_icu_renaming_auto_detect)");
    if renaming_auto_detect {
        println!("cargo::rustc-cfg=edit_icu_renaming_auto_detect");
    }
}
