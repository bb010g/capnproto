use anyhow::anyhow;
use kj_build::BuildExt;
use std::{env, fs, path::Path};

static CAPNP_JSON_SOURCES: &[&str] = &["compat/json.c++", "compat/json.capnp.c++"];
static CAPNP_JSON_EXTRAS: &[&str] = &["compat/json.capnp.h"];
static CAPNP_JSON_HEADERS: &[&str] = &["compat/json.h", "compat/json.capnp.h"];
static CAPNP_JSON_PRIVATE_HEADERS: &[&str] = &[];
static CAPNP_JSON_BRIDGES: &[&str] = &["src/lib.rs"];

fn main() -> anyhow::Result<()> {
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = env::var_os("OUT_DIR").ok_or_else(|| anyhow!("OUT_DIR not set"))?;
    let headers = Path::new(&out_dir).join("headers");
    cxx_build::CFG.exported_header_dirs.push(&headers);
    let private_headers = Path::new(&out_dir).join("private-headers");
    let sources = Path::new(&out_dir).join("sources");

    let source_dir = Path::new("../..");

    let capnp_json_header_dir = headers.join("capnp");
    fs::create_dir_all(capnp_json_header_dir.join("compat"))?;
    for capnp_json_header in CAPNP_JSON_HEADERS.into_iter() {
        let capnp_json_header_file = source_dir.join(capnp_json_header);
        println!(
            "cargo:rerun-if-changed={}",
            capnp_json_header_file
                .to_str()
                .ok_or_else(|| anyhow!("non–UTF-8 path: {:?}", capnp_json_header_file))?
        );
        fs::copy(
            &*capnp_json_header_file,
            &*capnp_json_header_dir.join(capnp_json_header),
        )?;
    }

    let capnp_json_private_header_dir = private_headers.join("capnp");
    fs::create_dir_all(capnp_json_private_header_dir.join("compat"))?;
    fs::create_dir_all(private_headers.join("kj"))?;
    for capnp_json_private_header in CAPNP_JSON_PRIVATE_HEADERS {
        let capnp_json_private_header_file = source_dir.join(capnp_json_private_header);
        println!(
            "cargo:rerun-if-changed={}",
            capnp_json_private_header_file
                .to_str()
                .ok_or_else(|| anyhow!("non–UTF-8 path: {:?}", capnp_json_private_header_file))?
        );
        fs::copy(
            &*capnp_json_private_header_file,
            &*capnp_json_private_header_dir.join(capnp_json_private_header),
        )?;
    }

    cxx_build::CFG.include_prefix = "capnp";
    let mut build = cxx_build::bridges(CAPNP_JSON_BRIDGES);
    let mut kj_cfg = kj_build::Cfg::default();
    kj_cfg.import_propagated_definitions()?;

    let use_libdl: Option<&str> = kj_cfg
        .get_propagated_definition("KJ_HAS_LIBDL")
        .last()
        .and_then(|(_dep, val)| *val);

    let capnp_json_source_dir = sources.join("capnp");
    fs::create_dir_all(capnp_json_source_dir.join("compat"))?;
    for capnp_json_source in CAPNP_JSON_SOURCES {
        let capnp_json_source_file = source_dir.join(capnp_json_source);
        println!(
            "cargo:rerun-if-changed={}",
            capnp_json_source_file
                .to_str()
                .ok_or_else(|| anyhow!("non–UTF-8 path: {:?}", capnp_json_source_file))?
        );
        let hermetic_capnp_json_source = capnp_json_source_dir.join(capnp_json_source);
        fs::copy(&*capnp_json_source_file, &*hermetic_capnp_json_source)?;
        build.file(hermetic_capnp_json_source);
    }
    for capnp_json_extra in CAPNP_JSON_EXTRAS {
        let capnp_json_extra_file = source_dir.join(capnp_json_extra);
        println!(
            "cargo:rerun-if-changed={}",
            capnp_json_extra_file
                .to_str()
                .ok_or_else(|| anyhow!("non–UTF-8 path: {:?}", capnp_json_extra_file))?
        );
        let hermetic_capnp_json_extra = capnp_json_source_dir.join(capnp_json_extra);
        fs::copy(&*capnp_json_extra_file, &*hermetic_capnp_json_extra)?;
    }
    build.include(private_headers);
    build.flag("-std=c++14");
    println!("cargo:rustc-link-lib=pthread");
    if use_libdl.is_some() {
        println!("cargo:rustc-link-lib=dl");
    }
    build
        .propagate_definitions(&mut kj_cfg)?
        .compile("capnp-json");

    Ok(())
}
