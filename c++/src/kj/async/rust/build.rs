use anyhow::anyhow;
use kj_build::BuildExt;
use std::{env, fs, path::Path};

static KJ_ASYNC_SOURCES: &[&str] = &[
    "async.c++",
    "async-unix.c++",
    "async-win32.c++",
    "async-io-win32.c++",
    "async-io.c++",
    "async-io-unix.c++",
    "timer.c++",
];
static KJ_ASYNC_HEADERS: &[&str] = &[
    "async-prelude.h",
    "async.h",
    "async-inl.h",
    "async-unix.h",
    "async-win32.h",
    "async-io.h",
    "async-queue.h",
    "timer.h",
];
static KJ_ASYNC_PRIVATE_HEADERS: &[&str] = &["async-io-internal.h", "miniposix.h"];
static KJ_BRIDGES: &[&str] = &["src/lib.rs"];

fn main() -> anyhow::Result<()> {
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = env::var_os("OUT_DIR").ok_or_else(|| anyhow!("OUT_DIR not set"))?;
    let headers = Path::new(&out_dir).join("headers");
    cxx_build::CFG.exported_header_dirs.push(&headers);
    let sources = Path::new(&out_dir).join("sources");

    let source_dir = Path::new("../..");

    let kj_header_dir = headers.join("kj");
    fs::create_dir_all(&kj_header_dir)?;
    for kj_header in KJ_ASYNC_HEADERS {
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

    for kj_private_header in KJ_ASYNC_PRIVATE_HEADERS {
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

    for kj_source in KJ_ASYNC_SOURCES {
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
    build.flag("-std=c++14");
    build
        .propagate_definitions(&mut kj_cfg)?
        .compile("kj-async");

    Ok(())
}
