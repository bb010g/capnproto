use anyhow::anyhow;
use kj_build::BuildExt;
use std::{env, fs, path::Path};

static CAPNP_SOURCES_LITE: &[&str] = &[
    "c++.capnp.c++",
    "blob.c++",
    "arena.c++",
    "layout.c++",
    "list.c++",
    "any.c++",
    "message.c++",
    "schema.capnp.c++",
    "stream.capnp.c++",
    "serialize.c++",
    "serialize-packed.c++",
];
static CAPNP_SOURCES_HEAVY: &[&str] = &[
    "schema.c++",
    "schema-loader.c++",
    "dynamic.c++",
    "stringify.c++",
];
static CAPNP_EXTRAS: &[&str] = &["c++.capnp.h", "schema.capnp.h", "stream.capnp.h"];
static CAPNP_HEADERS: &[&str] = &[
    "c++.capnp.h",
    "common.h",
    "blob.h",
    "endian.h",
    "layout.h",
    "orphan.h",
    "list.h",
    "any.h",
    "message.h",
    "capability.h",
    "membrane.h",
    "dynamic.h",
    "schema.h",
    "schema.capnp.h",
    "stream.capnp.h",
    "schema-lite.h",
    "schema-loader.h",
    "schema-parser.h",
    "pretty-print.h",
    "serialize.h",
    "serialize-async.h",
    "serialize-packed.h",
    "serialize-text.h",
    "pointer-helpers.h",
    "generated-header-support.h",
    "raw-schema.h",
];
static CAPNP_PRIVATE_HEADERS: &[&str] = &["arena.h"];
static CAPNP_COMPAT_HEADERS: &[&str] = &["compat/std-iterator.h"];
static CAPNP_BRIDGES: &[&str] = &["src/lib.rs"];

fn main() -> anyhow::Result<()> {
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = env::var_os("OUT_DIR").ok_or_else(|| anyhow!("OUT_DIR not set"))?;
    let headers = Path::new(&out_dir).join("headers");
    cxx_build::CFG.exported_header_dirs.push(&headers);
    let private_headers = Path::new(&out_dir).join("private-headers");
    let sources = Path::new(&out_dir).join("sources");

    let source_dir = Path::new("..");

    let capnp_header_dir = headers.join("capnp");
    fs::create_dir_all(&capnp_header_dir)?;
    fs::create_dir_all(capnp_header_dir.join("compat"))?;
    for capnp_header in CAPNP_HEADERS.into_iter().chain(CAPNP_COMPAT_HEADERS) {
        let capnp_header_file = source_dir.join(capnp_header);
        println!(
            "cargo:rerun-if-changed={}",
            capnp_header_file
                .to_str()
                .ok_or_else(|| anyhow!("non–UTF-8 path: {:?}", capnp_header_file))?
        );
        fs::copy(&*capnp_header_file, &*capnp_header_dir.join(capnp_header))?;
    }

    let capnp_private_header_dir = private_headers.join("capnp");
    fs::create_dir_all(&capnp_private_header_dir)?;
    for capnp_private_header in CAPNP_PRIVATE_HEADERS {
        let capnp_private_header_file = source_dir.join(capnp_private_header);
        println!(
            "cargo:rerun-if-changed={}",
            capnp_private_header_file
                .to_str()
                .ok_or_else(|| anyhow!("non–UTF-8 path: {:?}", capnp_private_header_file))?
        );
        fs::copy(
            &*capnp_private_header_file,
            &*capnp_private_header_dir.join(capnp_private_header),
        )?;
    }

    cxx_build::CFG.include_prefix = "capnp";
    let mut build = cxx_build::bridges(CAPNP_BRIDGES);
    let mut kj_cfg = kj_build::Cfg::default();
    kj_cfg.import_propagated_definitions()?;

    let use_libdl: Option<&str> = kj_cfg
        .get_propagated_definition("KJ_HAS_LIBDL")
        .last()
        .and_then(|(_dep, val)| *val);
    let capnp_lite: Option<&str> = kj_cfg
        .get_propagated_definition("CAPNP_LITE")
        .last()
        .and_then(|(_dep, val)| *val);
    let capnp_heavy: bool = !capnp_lite.is_none();

    let capnp_source_dir = sources.join("capnp");
    fs::create_dir_all(&capnp_source_dir)?;
    fs::create_dir_all(capnp_source_dir.join("parse"))?;
    fs::create_dir_all(capnp_source_dir.join("std"))?;
    for capnp_source in CAPNP_SOURCES_LITE {
        let capnp_source_file = source_dir.join(capnp_source);
        println!(
            "cargo:rerun-if-changed={}",
            capnp_source_file
                .to_str()
                .ok_or_else(|| anyhow!("non–UTF-8 path: {:?}", capnp_source_file))?
        );
        let hermetic_capnp_source = capnp_source_dir.join(capnp_source);
        fs::copy(&*capnp_source_file, &*hermetic_capnp_source)?;
        build.file(hermetic_capnp_source);
    }
    if capnp_heavy {
        for capnp_source in CAPNP_SOURCES_HEAVY {
            let capnp_source_file = source_dir.join(capnp_source);
            println!(
                "cargo:rerun-if-changed={}",
                capnp_source_file
                    .to_str()
                    .ok_or_else(|| anyhow!("non–UTF-8 path: {:?}", capnp_source_file))?
            );
            let hermetic_capnp_source = capnp_source_dir.join(capnp_source);
            fs::copy(&*capnp_source_file, &*hermetic_capnp_source)?;
            build.file(hermetic_capnp_source);
        }
    }
    for capnp_extra in CAPNP_EXTRAS {
        let capnp_extra_file = source_dir.join(capnp_extra);
        println!(
            "cargo:rerun-if-changed={}",
            capnp_extra_file
                .to_str()
                .ok_or_else(|| anyhow!("non–UTF-8 path: {:?}", capnp_extra_file))?
        );
        let hermetic_capnp_extra = capnp_source_dir.join(capnp_extra);
        fs::copy(&*capnp_extra_file, &*hermetic_capnp_extra)?;
    }
    build.include(private_headers);
    build.flag("-std=c++14");
    println!("cargo:rustc-link-lib=pthread");
    if use_libdl.is_some() {
        println!("cargo:rustc-link-lib=dl");
    }
    build.propagate_definitions(&mut kj_cfg)?.compile("capnp");

    // Err(anyhow!("woah"))
    Ok(())
}
