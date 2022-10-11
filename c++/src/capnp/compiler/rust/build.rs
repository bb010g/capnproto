use anyhow::anyhow;
use kj_build::BuildExt;
use std::{env, fs, path::Path};

static CAPNPC_SOURCES: &[&str] = &[
    "compiler/type-id.c++",
    "compiler/error-reporter.c++",
    "compiler/lexer.capnp.c++",
    "compiler/lexer.c++",
    "compiler/grammar.capnp.c++",
    "compiler/parser.c++",
    "compiler/generics.c++",
    "compiler/node-translator.c++",
    "compiler/compiler.c++",
    "compiler/capnp.c++",
    "schema-parser.c++",
    "serialize-text.c++",
];
static CAPNPC_EXTRAS: &[&str] = &["compiler/lexer.capnp.h", "compiler/grammar.capnp.h"];
static CAPNPC_HEADERS: &[&str] = &[
    "compiler/rust/cxx.h",
    "compiler/type-id.h",
    "compiler/error-reporter.h",
    "compiler/lexer.capnp.h",
    "compiler/lexer.h",
    "compiler/grammar.capnp.h",
    "compiler/parser.h",
    "compiler/generics.h",
    "compiler/node-translator.h",
    "compiler/compiler.h",
    "compiler/module-loader.h",
    "compiler/resolver.h",
];
static CAPNPC_PRIVATE_HEADERS: &[&str] = &["../kj/miniposix.h"];
static CAPNPC_BRIDGES: &[&str] = &["src/lib.rs"];

fn main() -> anyhow::Result<()> {
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = env::var_os("OUT_DIR").ok_or_else(|| anyhow!("OUT_DIR not set"))?;
    let headers = Path::new(&out_dir).join("headers");
    cxx_build::CFG.exported_header_dirs.push(&headers);
    let private_headers = Path::new(&out_dir).join("private-headers");
    let sources = Path::new(&out_dir).join("sources");

    let source_dir = Path::new("../..");

    let capnpc_header_dir = headers.join("capnp");
    fs::create_dir_all(capnpc_header_dir.join("compiler").join("rust"))?;
    for capnpc_header in CAPNPC_HEADERS.into_iter() {
        let capnpc_header_file = source_dir.join(capnpc_header);
        println!(
            "cargo:rerun-if-changed={}",
            capnpc_header_file
                .to_str()
                .ok_or_else(|| anyhow!("non–UTF-8 path: {:?}", capnpc_header_file))?
        );
        fs::copy(
            &*capnpc_header_file,
            &*capnpc_header_dir.join(capnpc_header),
        )?;
    }

    let capnpc_private_header_dir = private_headers.join("capnp");
    fs::create_dir_all(capnpc_private_header_dir.join("compiler"))?;
    fs::create_dir_all(private_headers.join("kj"))?;
    for capnpc_private_header in CAPNPC_PRIVATE_HEADERS {
        let capnpc_private_header_file = source_dir.join(capnpc_private_header);
        println!(
            "cargo:rerun-if-changed={}",
            capnpc_private_header_file
                .to_str()
                .ok_or_else(|| anyhow!("non–UTF-8 path: {:?}", capnpc_private_header_file))?
        );
        fs::copy(
            &*capnpc_private_header_file,
            &*capnpc_private_header_dir.join(capnpc_private_header),
        )?;
    }

    cxx_build::CFG.include_prefix = "capnp";
    let mut build = cxx_build::bridges(CAPNPC_BRIDGES);
    let mut kj_cfg = kj_build::Cfg::default();
    kj_cfg.import_propagated_definitions()?;

    let use_libdl: Option<&str> = kj_cfg
        .get_propagated_definition("KJ_HAS_LIBDL")
        .last()
        .and_then(|(_dep, val)| *val);

    let capnpc_source_dir = sources.join("capnp");
    fs::create_dir_all(capnpc_source_dir.join("compiler"))?;
    for capnpc_source in CAPNPC_SOURCES {
        let capnpc_source_file = source_dir.join(capnpc_source);
        println!(
            "cargo:rerun-if-changed={}",
            capnpc_source_file
                .to_str()
                .ok_or_else(|| anyhow!("non–UTF-8 path: {:?}", capnpc_source_file))?
        );
        let hermetic_capnpc_source = capnpc_source_dir.join(capnpc_source);
        fs::copy(&*capnpc_source_file, &*hermetic_capnpc_source)?;
        build.file(hermetic_capnpc_source);
    }
    for capnpc_extra in CAPNPC_EXTRAS {
        let capnpc_extra_file = source_dir.join(capnpc_extra);
        println!(
            "cargo:rerun-if-changed={}",
            capnpc_extra_file
                .to_str()
                .ok_or_else(|| anyhow!("non–UTF-8 path: {:?}", capnpc_extra_file))?
        );
        let hermetic_capnpc_extra = capnpc_source_dir.join(capnpc_extra);
        fs::copy(&*capnpc_extra_file, &*hermetic_capnpc_extra)?;
    }
    build.include(private_headers);
    build.flag("-std=c++14");
    println!("cargo:rustc-link-lib=pthread");
    if use_libdl.is_some() {
        println!("cargo:rustc-link-lib=dl");
    }
    build
        .propagate_definitions(&mut kj_cfg)?
        .define("CAPNP_COMPILER_NAMESPACED_MAIN", "1")
        .compile("capnpc");

    Ok(())
}
