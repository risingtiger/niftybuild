use anyhow::Result;
use regex::Regex;
use std::fs::{self};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::common_helperfuncs;

use crate::common_helperfuncs::path;
use crate::common_helperfuncs::pathp;
use crate::common_helperfuncs::PathE;

pub fn runit(appversion: u32) -> Result<u32> {
    let _ = process_manifest(appversion)?;
    let _ = process_indexhtml(appversion)?;
    let _ = process_json()?;
    let _ = process_sw(appversion)?;
    let _ = process_shared_worker()?;
    let _ = process_thirdparty()?;
    let _ = process_css()?;
    let _ = process_media()?;
    let _ = process_server(appversion)?;

    Ok(appversion)
}

fn process_manifest(appversion: u32) -> Result<()> {
    let manifest_in_path = pathp(PathE::ClientOutputDev, "app.webmanifest");
    let manifest_out_path = pathp(PathE::ClientOutputDist, "app.webmanifest");

    let manifest_in_content = fs::read_to_string(&manifest_in_path)?;

    let version_regex = Regex::new(r#""version":\s*"(\d+)""#)?;
    let manifest_in_content = version_regex.replace(
        &manifest_in_content,
        format!(r#""version": "{}""#, appversion),
    );

    fs::write(&manifest_out_path, manifest_in_content.as_bytes())?;

    Ok(())
}

fn process_indexhtml(appversion: u32) -> Result<()> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let html_in_str = pathp(PathE::ClientOutputDev, "index.html");
    let html_out_str = pathp(PathE::ClientOutputDist, "index.html");
    let html_content = fs::read_to_string(&html_in_str)?;

    let html_str = html_content.replace(
        "APPVERSION=0",
        format!("APPVERSION={}", appversion).as_str(),
    );
    let html_str = html_str.replace("APPUPDATE_TS=0", format!("APPUPDATE_TS={}", now).as_str());

    fs::write(&html_out_str, &html_str)?;

    Ok(())
}

fn process_json() -> Result<()> {
    let main_json_in_path = pathp(PathE::ClientOutputDev, "main.json");
    let main_json_out_path = pathp(PathE::ClientOutputDist, "main.json");

    fs::copy(&main_json_in_path, &main_json_out_path)?;

    Ok(())
}

fn process_sw(appversion: u32) -> Result<()> {
    let sw_in_str = pathp(PathE::ClientOutputDev, "sw.js");
    let sw_out_str = pathp(PathE::ClientOutputDist, "sw.js");

    let sw_content = fs::read_to_string(&sw_in_str)?;

    let sw_str = sw_content
        .replace("cacheV__0__", &format!("cacheV__{}__", appversion))
        .replace("export { };", "");

    fs::write(&sw_out_str, &sw_str)?;

    Ok(())
}

fn process_shared_worker() -> Result<()> {
    let shared_worker_in_str = pathp(PathE::ClientOutputDev, "shared_worker.js");
    let shared_worker_out_str = pathp(PathE::ClientOutputDist, "shared_worker.js");

    let shared_worker_content = fs::read_to_string(&shared_worker_in_str)?;

    fs::write(&shared_worker_out_str, &shared_worker_content)?;

    Ok(())
}

fn process_thirdparty() -> Result<()> {
    let in_path = pathp(PathE::ClientOutputDev, "thirdparty/");
    let out_path = pathp(PathE::ClientOutputDist, "thirdparty/");

    let in_instance_path = pathp(PathE::InstanceClientOutputDev, "thirdparty/");
    let out_instance_path = pathp(PathE::InstanceClientOutputDist, "thirdparty/");

    common_helperfuncs::copy_deep(in_path, out_path, "**/*", "_____").unwrap();
    common_helperfuncs::copy_deep(in_instance_path, out_instance_path, "**/*", "_____").unwrap();

    Ok(())
}

fn process_css() -> Result<()> {
    let tmp_path = pathp(PathE::TMPDir, "files/");
    let css_woff2_prefix = path(PathE::ClientOutputDev);
    let cssindex_in_str = pathp(PathE::ClientOutputDev, "index.css");
    let cssindex_out_str = pathp(PathE::ClientOutputDist, "index.css");
    let cssmain_in_str = pathp(PathE::ClientOutputDev, "main.css");
    let cssmain_out_str = pathp(PathE::ClientOutputDist, "main.css");
    let icons_in_str = pathp(PathE::InstanceClientOutputDev, "icons.css");
    let icons_out_str = pathp(PathE::InstanceClientOutputDist, "icons.css");

    let replace_with_path = format!("url(\"{}", css_woff2_prefix.to_string_lossy());
    let cssindex_content = fs::read_to_string(&cssindex_in_str)?;
    let cssindex_content = cssindex_content.replace("url(\"/assets/", &replace_with_path);
    fs::write(&cssindex_in_str, &cssindex_content)?;

    let cssindex_cmd = Command::new("esbuild")
        .args([
            cssindex_in_str.to_str().unwrap(),
            "--bundle",
            "--minify",
            "--loader:.woff2=dataurl",
        ])
        .current_dir(&tmp_path)
        .output()
        .expect("esbuild chucked an error");

    if !cssindex_cmd.status.success() {
        if !cssindex_cmd.stderr.is_empty() {
            eprintln!(
                "esbuild index.css error: {}",
                String::from_utf8_lossy(&cssindex_cmd.stderr)
            );
        }
        if !cssindex_cmd.stdout.is_empty() {
            eprintln!(
                "esbuild index.css: {}",
                String::from_utf8_lossy(&cssindex_cmd.stdout)
            );
        }
        eprintln!(
            "esbuild index.css command failed with exit code: {:?}",
            cssindex_cmd.status.code()
        );
    } else {
        let cssindex_content = String::from_utf8(cssindex_cmd.stdout).expect("css stdout error");
        let _ = fs::write(&cssindex_out_str, &cssindex_content);
    }

    let cssmain_cmd = Command::new("esbuild")
        .args([cssmain_in_str.to_str().unwrap(), "--minify"])
        .current_dir(&tmp_path)
        .output()
        .expect("esbuild chucked an error");

    if !cssmain_cmd.status.success() {
        if !cssmain_cmd.stderr.is_empty() {
            eprintln!(
                "esbuild main.css error: {}",
                String::from_utf8_lossy(&cssmain_cmd.stderr)
            );
        }
        if !cssmain_cmd.stdout.is_empty() {
            eprintln!(
                "esbuild main.css: {}",
                String::from_utf8_lossy(&cssmain_cmd.stdout)
            );
        }
        eprintln!(
            "esbuild main.css command failed with exit code: {:?}",
            cssmain_cmd.status.code()
        );
    } else {
        let cssmain_content = String::from_utf8(cssmain_cmd.stdout).expect("main.css stdout error");
        let _ = fs::write(&cssmain_out_str, &cssmain_content);
    }

    let icons_cmd = Command::new("esbuild")
        .args([icons_in_str.to_str().unwrap(), "--minify"])
        .current_dir(&tmp_path)
        .output()
        .expect("esbuild chucked an error");

    if !icons_cmd.status.success() {
        if !icons_cmd.stderr.is_empty() {
            eprintln!(
                "esbuild icons.css error: {}",
                String::from_utf8_lossy(&icons_cmd.stderr)
            );
        }
        if !icons_cmd.stdout.is_empty() {
            eprintln!(
                "esbuild icons.css: {}",
                String::from_utf8_lossy(&icons_cmd.stdout)
            );
        }
        anyhow::bail!(
            "esbuild icons.css command failed with exit code: {:?}",
            icons_cmd.status.code()
        );
    } else {
        let icons_content = String::from_utf8(icons_cmd.stdout).expect("icons.css stdout error");
        fs::write(&icons_out_str, &icons_content)?;
    }

    Ok(())
}

fn process_media() -> Result<()> {
    let media_in_path = pathp(PathE::ClientOutputDev, "media/");
    let media_out_path = pathp(PathE::ClientOutputDist, "media/");
    let media_instance_in_path = pathp(PathE::InstanceClientOutputDev, "media/");
    let media_instance_out_path = pathp(PathE::InstanceClientOutputDist, "media/");

    common_helperfuncs::copy_deep(media_in_path, media_out_path, "**/*", "_____").unwrap();
    common_helperfuncs::copy_deep(
        media_instance_in_path,
        media_instance_out_path,
        "**/*",
        "_____",
    )
    .unwrap();

    Ok(())
}

fn process_server(appversion: u32) -> Result<()> {
    let server_in_path = pathp(PathE::ServerOutput, "index.js");

    let server_content = fs::read_to_string(&server_in_path)?;
    let server_regex = Regex::new(r#"APPVERSION = \d+"#).unwrap();
    let server_content = server_regex
        .replace_all(
            &server_content,
            format!("APPVERSION = {}", appversion).as_str(),
        )
        .to_string();

    fs::write(&server_in_path, &server_content)?;

    Ok(())
}
