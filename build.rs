use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let out_dir = env::var("OUT_DIR").expect("Cargo should provide OUT_DIR");
    build_and_embed_webui(Path::new(&out_dir));
}

fn build_and_embed_webui(out_dir: &Path) {
    for path in [
        "frontend/package.json",
        "frontend/package-lock.json",
        "frontend/svelte.config.js",
        "frontend/vite.config.ts",
        "frontend/tailwind.config.ts",
        "frontend/postcss.config.js",
        "frontend/src",
        "frontend/static",
    ] {
        println!("cargo:rerun-if-changed={path}");
    }

    let npm = if cfg!(windows) { "npm.cmd" } else { "npm" };
    let manifest_dir = PathBuf::from(
        env::var("CARGO_MANIFEST_DIR").expect("Cargo should provide CARGO_MANIFEST_DIR"),
    );
    let build_instance = out_dir
        .parent()
        .and_then(Path::file_name)
        .and_then(|name| name.to_str())
        .unwrap_or("default");
    let cargo_webui_dir = manifest_dir.join("frontend/.cargo-svelte").join(format!(
        "{}-{build_instance}",
        env::var("PROFILE").unwrap_or_else(|_| "unknown".to_string())
    ));
    let svelte_kit_out_dir = cargo_webui_dir.join("svelte-kit");
    let build_dir = cargo_webui_dir.join("build");
    let status = Command::new(npm)
        .args(["run", "build"])
        .current_dir("frontend")
        .env("SVELTE_KIT_OUT_DIR", &svelte_kit_out_dir)
        .env("WEBUI_BUILD_DIR", &build_dir)
        .status()
        .unwrap_or_else(|error| {
            panic!(
                "failed to run the WebUI build with '{npm}': {error}\n\
                 The build embeds the frontend at build time and needs Node.js/npm \
                 on PATH. Install Node, then run `npm --prefix frontend ci` once and rebuild."
            );
        });

    if !status.success() {
        panic!(
            "WebUI build failed with status {status}.\n\
             If frontend dependencies are missing, run `npm --prefix frontend ci` and rebuild."
        );
    }

    let mut files = Vec::new();
    collect_files(&build_dir, &mut files)
        .unwrap_or_else(|error| panic!("failed to inspect WebUI build output: {error}"));
    files.sort();

    if files.is_empty() {
        panic!("WebUI build produced no files in {}", build_dir.display());
    }

    let mut generated = String::from("static EMBEDDED_ASSETS: &[EmbeddedAsset] = &[\n");
    for file in files {
        let relative = file
            .strip_prefix(&build_dir)
            .expect("WebUI asset should be inside the build directory")
            .to_string_lossy()
            .replace('\\', "/");
        let absolute = fs::canonicalize(&file)
            .unwrap_or_else(|error| {
                panic!(
                    "failed to canonicalize WebUI asset {}: {error}",
                    file.display()
                )
            })
            .to_string_lossy()
            .into_owned();
        generated.push_str(&format!(
            "    EmbeddedAsset {{ path: {relative:?}, content_type: {:?}, bytes: include_bytes!({absolute:?}) }},\n",
            content_type(&file)
        ));
    }
    generated.push_str("];\n");

    fs::write(out_dir.join("webui_assets.rs"), generated)
        .unwrap_or_else(|error| panic!("failed to generate embedded WebUI assets: {error}"));
}

fn collect_files(directory: &Path, files: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_files(&path, files)?;
        } else if path.is_file() {
            files.push(path);
        }
    }
    Ok(())
}

fn content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("css") => "text/css; charset=utf-8",
        Some("html") => "text/html; charset=utf-8",
        Some("js") | Some("mjs") => "text/javascript; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("ico") => "image/x-icon",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        Some("txt") => "text/plain; charset=utf-8",
        Some("xml") => "application/xml; charset=utf-8",
        Some("wasm") => "application/wasm",
        _ => "application/octet-stream",
    }
}
