use regex::Regex;
use std::fs;
//use std::os::unix::fs::symlink;
use anyhow::{Context, Result};
use rayon::prelude::*;
use std::path::PathBuf;

use crate::common_helperfuncs;
use crate::common_helperfuncs::path;
use crate::common_helperfuncs::pathp;
use crate::common_helperfuncs::PathE;

pub mod copy_chrome_css_changes;
pub mod handlefile;
pub mod media;
pub mod server;
pub mod thirdparty;

struct ManifestInfoT {
    short_name: String,
    theme_color: String,
}

pub fn alldev() -> Result<()> {
    reset_dev_dirs()?;

    let core_thread = std::thread::spawn(|| handle_core());
    let lazy_thread = std::thread::spawn(|| handle_lazy());
    let thirdparty_thread = std::thread::spawn(|| thirdparty::runit());
    let media_thread = std::thread::spawn(|| media::runit());
    let server_thread = std::thread::spawn(|| server::runit());

    core_thread
        .join()
        .map_err(|e| anyhow::anyhow!("Dev Core thread panicked: {:?}", e))??;
    lazy_thread
        .join()
        .map_err(|e| anyhow::anyhow!("Dev Lazy thread panicked: {:?}", e))??;
    thirdparty_thread
        .join()
        .map_err(|e| anyhow::anyhow!("Dev Thirdparty thread panicked: {:?}", e))??;
    media_thread
        .join()
        .map_err(|e| anyhow::anyhow!("Dev Media thread panicked: {:?}", e))??;
    server_thread
        .join()
        .map_err(|e| anyhow::anyhow!("Dev Server thread panicked: {:?}", e))??;

    Ok(())
}

pub fn handle_core() -> Result<()> {
    let manifest = handle_manifest()?;

    let glob_files = vec![
        "sw.ts",
        "shared_worker.ts",
        "main.ts",
        "alwaysload/**/*",
        "defs*.ts",
    ];
    let js_client_thread = std::thread::spawn(move || {
        common_helperfuncs::run_swc(
            path(PathE::ClientSrc),
            path(PathE::ClientOutputDev),
            glob_files,
        )
    });

    let glob_files = vec!["main.ts", "alwaysload/**/*", "defs*.ts"];
    let js_instance_thread = std::thread::spawn(move || {
        common_helperfuncs::run_swc(
            path(PathE::InstanceClientSrc),
            path(PathE::InstanceClientOutputDev),
            glob_files,
        )
    });

    let handle_primary_css_files_thread = std::thread::spawn(move || handle_css_files());

    js_client_thread
        .join()
        .map_err(|e| anyhow::anyhow!("JS thread panicked: {:?}", e))??;
    js_instance_thread
        .join()
        .map_err(|e| anyhow::anyhow!("JS thread panicked: {:?}", e))??;
    handle_primary_css_files_thread
        .join()
        .map_err(|e| anyhow::anyhow!("Primary css files thread panicked: {:?}", e))??;

    let short_name = manifest.short_name;
    let theme_color = manifest.theme_color;
    let index_thread = std::thread::spawn(move || handle_indexhtml(&short_name, &theme_color));
    let main_thread = std::thread::spawn(|| handle_mainjs_n_json());

    index_thread
        .join()
        .map_err(|e| anyhow::anyhow!("Index html thread panicked: {:?}", e))??;
    main_thread
        .join()
        .map_err(|e| anyhow::anyhow!("Main JS/json thread panicked: {:?}", e))??;

    apply_devappversion_core_outputs()?;

    Ok(())
}

pub fn handle_lazy() -> Result<()> {
    let glob_files = vec!["lazy/**/*"];
    let js_client_thread = std::thread::spawn(move || {
        common_helperfuncs::run_swc(
            path(PathE::ClientSrc),
            path(PathE::ClientOutputDev),
            glob_files,
        )
    });

    let glob_files = vec!["lazy/**/*"];
    let js_instance_thread = std::thread::spawn(move || {
        common_helperfuncs::run_swc(
            path(PathE::InstanceClientSrc),
            path(PathE::InstanceClientOutputDev),
            glob_files,
        )
    });

    let client_deep_copy_except_js = std::thread::spawn(move || {
        common_helperfuncs::copy_deep(
            pathp(PathE::ClientSrc, "lazy/"),
            pathp(PathE::ClientOutputDev, "lazy/"),
            "**/*",
            "**/*.ts",
        )
    });

    let instance_deep_copy_except_js = std::thread::spawn(move || {
        common_helperfuncs::copy_deep(
            pathp(PathE::InstanceClientSrc, "lazy/"),
            pathp(PathE::InstanceClientOutputDev, "lazy/"),
            "**/*",
            "**/*.ts",
        )
    });

    js_client_thread
        .join()
        .map_err(|e| anyhow::anyhow!("JS thread panicked: {:?}", e))??;
    js_instance_thread
        .join()
        .map_err(|e| anyhow::anyhow!("JS thread panicked: {:?}", e))??;
    client_deep_copy_except_js
        .join()
        .map_err(|e| anyhow::anyhow!("Lazy deep copy except js panicked: {:?}", e))??;
    instance_deep_copy_except_js
        .join()
        .map_err(|e| anyhow::anyhow!("Lazy deep copy except js panicked: {:?}", e))??;

    Ok(())
}

pub fn handle_corelazy() -> Result<()> {
    let core_thread = std::thread::spawn(|| handle_core());
    let lazy_thread = std::thread::spawn(|| handle_lazy());

    core_thread
        .join()
        .map_err(|e| anyhow::anyhow!("Core thread panicked: {:?}", e))??;
    lazy_thread
        .join()
        .map_err(|e| anyhow::anyhow!("Lazy thread panicked: {:?}", e))??;

    Ok(())
}

fn handle_manifest() -> Result<ManifestInfoT> {
    let manifest_in_path = pathp(PathE::ClientSrc, "app.webmanifest");
    let manifest_instance_in_path = pathp(PathE::InstanceClientSrc, "app.webmanifest");
    let manifest_out_path = pathp(PathE::ClientOutputDev, "app.webmanifest");

    let manifest_main = fs::read_to_string(&manifest_in_path).with_context(|| {
        format!(
            "failed to read main manifest {}",
            manifest_in_path.display()
        )
    })?;
    let manifest_instance = fs::read_to_string(&manifest_instance_in_path).with_context(|| {
        format!(
            "failed to read instance manifest {}",
            manifest_instance_in_path.display()
        )
    })?;

    let mut manifest: serde_json::Value = serde_json::from_str(&manifest_main).with_context(|| {
        format!(
            "failed to parse main manifest {}",
            manifest_in_path.display()
        )
    })?;
    let manifest_instance: serde_json::Value =
        serde_json::from_str(&manifest_instance).with_context(|| {
            format!(
                "failed to parse instance manifest {}",
                manifest_instance_in_path.display()
            )
        })?;

    // instance-owned fields always overlay the main template; the optional ones only when present
    let overlay_keys = [
        "name",
        "short_name",
        "description",
        "theme_color",
        "background_color",
        "icons",
        "screenshots",
        "version",
        "orientation",
        "categories",
        "shortcuts",
    ];
    for key in overlay_keys {
        if let Some(v) = manifest_instance.get(key) {
            manifest[key] = v.clone();
        }
    }

    inline_manifest_media_as_datauris(&mut manifest)?;

    let short_name = manifest["short_name"].as_str().unwrap_or("").to_string();
    let theme_color = manifest["theme_color"].as_str().unwrap_or("#FFFFFF").to_string();

    let manifest_str = serde_json::to_string_pretty(&manifest)
        .context("failed to serialize combined app manifest")?;
    fs::write(&manifest_out_path, manifest_str).with_context(|| {
        format!(
            "failed to write combined manifest {}",
            manifest_out_path.display()
        )
    })?;

    Ok(ManifestInfoT { short_name, theme_color })
}

// rewrites local media srcs in icons/screenshots as data: URIs so the browser never
// fetches them as separate files (manifest itself is cached by the service worker)
fn inline_manifest_media_as_datauris(manifest: &mut serde_json::Value) -> Result<()> {
    for listkey in ["icons", "screenshots"] {
        let Some(entries) = manifest.get_mut(listkey).and_then(|v| v.as_array_mut()) else {
            continue;
        };

        for entry in entries {
            let Some(src) = entry.get("src").and_then(|s| s.as_str()) else {
                continue;
            };
            if src.starts_with("data:") || src.starts_with("http") {
                continue;
            }

            let Some(file_path) = resolve_media_src_path(src) else {
                continue;
            };

            let bytes = fs::read(&file_path).with_context(|| {
                format!(
                    "manifest {} src not found on disk: {}",
                    listkey,
                    file_path.display()
                )
            })?;

            let mime = mime_for_media_path(&file_path);
            let datauri = format!("data:{};base64,{}", mime, base64_encode(&bytes));

            entry["src"] = serde_json::Value::String(datauri);
            entry["type"] = serde_json::Value::String(mime.to_string());
        }
    }

    Ok(())
}

// manifest srcs are output urls (/assets/...); map them back to their source files
fn resolve_media_src_path(src: &str) -> Option<PathBuf> {
    if let Some(rest) = src.strip_prefix("/assets/instance/media/") {
        return Some(pathp(PathE::InstanceClientSrc, format!("media/{}", rest)));
    }
    if let Some(rest) = src.strip_prefix("/assets/media/") {
        return Some(pathp(PathE::ClientSrc, format!("media/{}", rest)));
    }
    None
}

fn mime_for_media_path(file_path: &PathBuf) -> &'static str {
    match file_path.extension().and_then(|e| e.to_str()).unwrap_or("") {
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "gif" => "image/gif",
        _ => "application/octet-stream",
    }
}

fn base64_encode(bytes: &[u8]) -> String {
    use base64::Engine as _;
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

fn handle_indexhtml(manifestname: &str, theme_color: &str) -> Result<()> {
    let index_in_path = pathp(PathE::ClientSrc, "index.html");
    let index_out_path = pathp(PathE::ClientOutputDev, "index.html");

    let index = fs::read_to_string(&index_in_path)
        .with_context(|| format!("failed to read index html {}", index_in_path.display()))?;
    let index = index.replace(
        "<title></title>",
        &format!("<title>{}</title>", manifestname),
    );
    let index = index.replace("{--appname--}", manifestname);
    let index = index.replace("{--themecolor--}", theme_color);
    let index = index.replace("<!--{--favicon--}-->", &build_favicon_link_tag()?);
    fs::write(&index_out_path, index)
        .with_context(|| format!("failed to write index html {}", index_out_path.display()))?;

    Ok(())
}

// inline svg favicon as a data uri so browsers never auto-request /favicon.ico
fn build_favicon_link_tag() -> Result<String> {
    let instance_favicon_path = pathp(PathE::InstanceClientSrc, "media/favicon.svg");
    let main_favicon_path = pathp(PathE::ClientSrc, "media/pwticons/favicon.svg");

    let favicon_path = if instance_favicon_path.exists() {
        instance_favicon_path
    } else {
        main_favicon_path
    };

    let bytes = fs::read(&favicon_path).with_context(|| {
        format!("failed to read favicon svg {}", favicon_path.display())
    })?;

    Ok(format!(
        r#"<link rel="icon" href="data:image/svg+xml;base64,{}">"#,
        base64_encode(&bytes)
    ))
}

fn handle_mainjs_n_json() -> Result<()> {
    let mainjs_in_path = pathp(PathE::ClientOutputDev, "main.js");
    let instancejs_in_path = pathp(PathE::InstanceClientOutputDev, "main.js");

    let main_json_in_path = pathp(PathE::ClientSrc, "main.json");
    let instance_json_in_path = pathp(PathE::InstanceClientSrc, "main.json");

    let mainjs = fs::read_to_string(&mainjs_in_path).with_context(|| {
        format!(
            "failed to read generated main js {}",
            mainjs_in_path.display()
        )
    })?;
    let instancejs = fs::read_to_string(&instancejs_in_path).with_context(|| {
        format!(
            "failed to read generated instance main js {}",
            instancejs_in_path.display()
        )
    })?;

    let mainjson = fs::read_to_string(&main_json_in_path)
        .with_context(|| format!("failed to read main json {}", main_json_in_path.display()))?;
    let instancejson = fs::read_to_string(&instance_json_in_path).with_context(|| {
        format!(
            "failed to read instance json {}",
            instance_json_in_path.display()
        )
    })?;

    let combined_json = format!(
        r#"{{ "MAIN": {}, "INSTANCE": {} }}"#,
        mainjson, instancejson
    );

    let instance_n_json_combined = format!(r#"{} const SETTINGS={};"#, instancejs, combined_json);

    let main_json_out_path = pathp(PathE::ClientOutputDev, "main.json");

    let mainjs = mainjs.replace("//{--replace_slot.js--}", &instance_n_json_combined);

    fs::write(&mainjs_in_path, mainjs)
        .with_context(|| format!("failed to write main js {}", mainjs_in_path.display()))?;

    fs::write(&main_json_out_path, combined_json).with_context(|| {
        format!(
            "failed to write combined main json {}",
            main_json_out_path.display()
        )
    })?;

    Ok(())
}

fn handle_css_files() -> Result<()> {
    // may do more with css, like parsing for specific things (like fonts and sprite sheets and sucking in instance css in the furture

    let copy_jobs = vec![
        (
            pathp(PathE::ClientSrc, "index.css"),
            pathp(PathE::ClientOutputDev, "index.css"),
        ),
        (
            pathp(PathE::ClientSrc, "main.css"),
            pathp(PathE::ClientOutputDev, "main.css"),
        ),
        (
            pathp(PathE::InstanceClientSrc, "icons.css"),
            pathp(PathE::InstanceClientOutputDev, "icons.css"),
        ),
    ];

    copy_jobs
        .par_iter()
        .try_for_each(|(src, dest)| common_helperfuncs::copy_if_changed(src, dest).map(|_| ()))?;

    Ok(())
}

fn refresh_manifest_and_indexhtml() -> Result<()> {
    let manifest = handle_manifest()?;
    handle_indexhtml(&manifest.short_name, &manifest.theme_color)?;
    apply_devappversion_manifest()?;
    apply_devappversion_index()?;

    Ok(())
}

fn apply_devappversion_core_outputs() -> Result<()> {
    if crate::DEVAPPVERSION.clone() == 0 {
        return Ok(());
    }

    let manifest_thread = std::thread::spawn(|| apply_devappversion_manifest());
    let index_thread = std::thread::spawn(|| apply_devappversion_index());
    let sw_thread = std::thread::spawn(|| apply_devappversion_sw());

    manifest_thread
        .join()
        .map_err(|e| anyhow::anyhow!("Manifest devappversion thread panicked: {:?}", e))??;
    index_thread
        .join()
        .map_err(|e| anyhow::anyhow!("Index devappversion thread panicked: {:?}", e))??;
    sw_thread
        .join()
        .map_err(|e| anyhow::anyhow!("SW devappversion thread panicked: {:?}", e))??;

    Ok(())
}

fn apply_devappversion_manifest() -> Result<()> {
    let devappversion = crate::DEVAPPVERSION.clone();
    if devappversion == 0 {
        return Ok(());
    }

    let manifest_path = pathp(PathE::ClientOutputDev, "app.webmanifest");
    let manifest_content = fs::read_to_string(&manifest_path)?;
    let version_regex = Regex::new(r#""version":\s*"(\d+)""#)?;
    let manifest_content = version_regex
        .replace(
            &manifest_content,
            format!("\"version\": \"{}\"", devappversion).as_str(),
        )
        .to_string();
    fs::write(&manifest_path, &manifest_content)?;

    Ok(())
}

fn apply_devappversion_index() -> Result<()> {
    let devappversion = crate::DEVAPPVERSION.clone();
    if devappversion == 0 {
        return Ok(());
    }

    let clientindex_path = pathp(PathE::ClientOutputDev, "index.html");
    let clientindex_content = fs::read_to_string(&clientindex_path)?;
    let clientindex_content = clientindex_content.replace(
        "APPVERSION=0",
        format!("APPVERSION={}", devappversion).as_str(),
    );
    fs::write(&clientindex_path, &clientindex_content)?;

    Ok(())
}

fn apply_devappversion_sw() -> Result<()> {
    let devappversion = crate::DEVAPPVERSION.clone();
    if devappversion == 0 {
        return Ok(());
    }

    let sw_path = pathp(PathE::ClientOutputDev, "sw.js");
    let sw_content = fs::read_to_string(&sw_path)?;
    let sw_content = sw_content
        .replace("cacheV__0__", &format!("cacheV__{}__", devappversion))
        .replace("export { };", "");
    fs::write(&sw_path, &sw_content)?;

    Ok(())
}

pub fn handle_file_changed(changed_file: &PathBuf) -> Result<()> {
    handlefile::file_changed(changed_file)
}

pub fn handle_set_devappversion(devappversion: &str) -> Result<()> {
    std::fs::create_dir_all(pathp(PathE::TMPDir, "files/"))?;
    fs::write(
        pathp(PathE::TMPDir, "devappversion.txt"),
        devappversion.to_string(),
    )?;
    Ok(())
}

pub fn handle_copy_chrome_css_changes() -> Result<()> {
    let _affected_paths = copy_chrome_css_changes::runit();

    Ok(())
}

fn reset_dev_dirs() -> Result<()> {
    let _xx = std::fs::remove_dir_all(pathp(PathE::ClientOutputDev, ""));
    let _yy = std::fs::remove_dir_all(pathp(PathE::ServerOutput, ""));

    std::fs::create_dir_all(pathp(PathE::ClientOutputDev, ""))?;
    std::fs::create_dir_all(pathp(PathE::ServerOutput, ""))?;
    std::fs::create_dir_all(pathp(PathE::InstanceClientOutputDev, ""))?;

    Ok(())
}
