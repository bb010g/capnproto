use anyhow::anyhow;
use kj_build::BuildExt;
use std::{env, fs, path::Path};

const CAPNP_HEAVY: bool = cfg!(feature = "heavy");
const USE_LIBDL: bool = cfg!(feature = "libdl");
const USE_SAVE_ACQUIRED_LOCK_INFO: bool = cfg!(feature = "save_acquired_lock_info");
const USE_TRACK_LOCK_BLOCKING: bool = cfg!(feature = "track_lock_blocking");

static KJ_SOURCES_LITE: &[&str] = &[
    "array.c++",
    "list.c++",
    "common.c++",
    "debug.c++",
    "exception.c++",
    "io.c++",
    "memory.c++",
    "mutex.c++",
    "string.c++",
    "source-location.c++",
    "hash.c++",
    "table.c++",
    "thread.c++",
    "main.c++",
    "arena.c++",
    "test-helpers.c++",
    "units.c++",
    "encoding.c++",
];
static KJ_SOURCES_HEAVY: &[&str] = &[
    "refcount.c++",
    "string-tree.c++",
    "time.c++",
    "filesystem.c++",
    "filesystem-disk-unix.c++",
    "filesystem-disk-win32.c++",
    "parse/char.c++",
];
static KJ_HEADERS: &[&str] = &[
    "common.h",
    "units.h",
    "memory.h",
    "refcount.h",
    "array.h",
    "list.h",
    "vector.h",
    "string.h",
    "string-tree.h",
    "source-location.h",
    "hash.h",
    "table.h",
    "map.h",
    "encoding.h",
    "exception.h",
    "debug.h",
    "arena.h",
    "io.h",
    "tuple.h",
    "one-of.h",
    "function.h",
    "mutex.h",
    "thread.h",
    "threadlocal.h",
    "filesystem.h",
    "time.h",
    "main.h",
    "win32-api-version.h",
    "windows-sanity.h",
];
static KJ_PRIVATE_HEADERS: &[&str] = &["miniposix.h", "test.h"];
static KJ_PARSE_HEADERS: &[&str] = &["parse/common.h", "parse/char.h"];
static KJ_STD_HEADERS: &[&str] = &["std/iostream.h"];
static KJ_BRIDGES: &[&str] = &["src/lib.rs"];

fn bool_int_str(p: bool) -> &'static str {
    if p {
        "1"
    } else {
        "0"
    }
}

fn kj_configure<'a>(build: &'a mut cc::Build, kj_cfg: &mut kj_build::Cfg) -> &'a mut cc::Build {
    kj_cfg
        .define_propagated("KJ_CONTENTION_WARNING_THRESHOLD", "100")
        .define_propagated(
            "KJ_SAVE_ACQUIRED_LOCK_INFO",
            bool_int_str(USE_SAVE_ACQUIRED_LOCK_INFO),
        )
        .define_propagated(
            "KJ_TRACK_LOCK_BLOCKING",
            bool_int_str(USE_TRACK_LOCK_BLOCKING),
        );
    if USE_LIBDL {
        build.define("KJ_HAS_LIBDL", None);
    }
    build
}

fn capnp_configure<'a>(build: &'a mut cc::Build, kj_cfg: &mut kj_build::Cfg) -> &'a mut cc::Build {
    kj_configure(build, kj_cfg);
    if !CAPNP_HEAVY {
        kj_cfg.define_propagated("CAPNP_LITE", None);
    }
    build
}

fn main() -> anyhow::Result<()> {
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = env::var_os("OUT_DIR").ok_or_else(|| anyhow!("OUT_DIR not set"))?;
    let headers = Path::new(&out_dir).join("headers");
    cxx_build::CFG.exported_header_dirs.push(&headers);
    let sources = Path::new(&out_dir).join("sources");

    let source_dir = Path::new("..");

    let kj_header_dir = headers.join("kj");
    fs::create_dir_all(&kj_header_dir)?;
    fs::create_dir_all(kj_header_dir.join("parse"))?;
    fs::create_dir_all(kj_header_dir.join("std"))?;
    for kj_header in KJ_HEADERS
        .into_iter()
        .chain(KJ_PARSE_HEADERS)
        .chain(KJ_STD_HEADERS)
    {
        let kj_header_file = source_dir.join(kj_header);
        println!(
            "cargo:rerun-if-changed={}",
            kj_header_file
                .to_str()
                .ok_or_else(|| anyhow!("non–UTF-8 path: {:?}", kj_header_file))?
        );
        fs::copy(&*kj_header_file, &*kj_header_dir.join(kj_header))?;
    }

    let kj_source_dir = sources.join("kj");
    fs::create_dir_all(&kj_source_dir)?;

    for kj_private_header in KJ_PRIVATE_HEADERS {
        let kj_private_header_file = source_dir.join(kj_private_header);
        println!(
            "cargo:rerun-if-changed={}",
            kj_private_header_file
                .to_str()
                .ok_or_else(|| anyhow!("non–UTF-8 path: {:?}", kj_private_header_file))?
        );
        fs::copy(
            &*kj_private_header_file,
            &*kj_source_dir.join(kj_private_header),
        )?;
    }

    cxx_build::CFG.include_prefix = "kj";
    let mut build = cxx_build::bridges(KJ_BRIDGES);
    let mut kj_cfg = kj_build::Cfg::default();
    kj_cfg.import_propagated_definitions()?;

    fs::create_dir_all(kj_source_dir.join("parse"))?;
    fs::create_dir_all(kj_source_dir.join("std"))?;
    for kj_source in KJ_SOURCES_LITE {
        let kj_source_file = source_dir.join(kj_source);
        println!(
            "cargo:rerun-if-changed={}",
            kj_source_file
                .to_str()
                .ok_or_else(|| anyhow!("non–UTF-8 path: {:?}", kj_source_file))?
        );
        let hermetic_kj_source = kj_source_dir.join(kj_source);
        fs::copy(&*kj_source_file, &*hermetic_kj_source)?;
        build.file(hermetic_kj_source);
    }
    if CAPNP_HEAVY {
        for kj_source in KJ_SOURCES_HEAVY {
            let kj_source_file = source_dir.join(kj_source);
            println!(
                "cargo:rerun-if-changed={}",
                kj_source_file
                    .to_str()
                    .ok_or_else(|| anyhow!("non–UTF-8 path: {:?}", kj_source_file))?
            );
            let hermetic_kj_source = kj_source_dir.join(kj_source);
            fs::copy(&*kj_source_file, &*hermetic_kj_source)?;
            build.file(hermetic_kj_source);
        }
    }
    capnp_configure(&mut build, &mut kj_cfg);
    build.flag("-std=c++14");
    println!("cargo:rustc-link-lib=pthread");
    if USE_LIBDL {
        println!("cargo:rustc-link-lib=dl");
    }
    build.propagate_definitions(&mut kj_cfg)?.compile("kj");

    Ok(())
}
