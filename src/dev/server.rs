use anyhow::{Context, Result};
use regex::Regex;
use std::fs;
use std::path::PathBuf;

use crate::common_helperfuncs::{path, pathp, run_swc, PathE};

pub fn runit() -> Result<()> {
    let a = path(PathE::ServerSrc);
    let b = path(PathE::ServerOutput);

    let c = vec!["*.ts"]; // only top level ts files, skipping instance files
    let js_server_thread = std::thread::spawn(move || handle_server_js(&a, &b, c));

    let d = path(PathE::InstanceServerSrc);
    let e = path(PathE::InstanceServerOutput);

    let f = vec!["**/*.ts"];
    let js_instance_server_thread = std::thread::spawn(move || handle_server_js(&d, &e, f));

    js_server_thread
        .join()
        .map_err(|e| anyhow::anyhow!("JS thread panicked: {:?}", e))??;
    js_instance_server_thread
        .join()
        .map_err(|e| anyhow::anyhow!("JS thread panicked: {:?}", e))??;

    postprocess_indexjs()?;

    Ok(())
}

pub fn handle_server_js(src: &PathBuf, dest: &PathBuf, glob_files: Vec<&str>) -> Result<()> {
    run_swc(src.to_path_buf(), dest.to_path_buf(), glob_files)?;

    Ok(())
}

pub(super) fn postprocess_indexjs() -> Result<()> {
    handle_indexjs()?;
    apply_devappversion_indexjs()?;

    Ok(())
}

fn handle_indexjs() -> Result<()> {
    let indexjs_in_path = pathp(PathE::ServerOutput, "index.js");

    let indexjs = fs::read_to_string(&indexjs_in_path).with_context(|| {
        format!(
            "failed to read generated server index {}",
            indexjs_in_path.display()
        )
    })?;
    let indexjs = indexjs.replace(
        "//{--index_instance.js--}",
        "import INSTANCE from './instance/index.js'",
    );
    fs::write(&indexjs_in_path, indexjs)
        .with_context(|| format!("failed to write server index {}", indexjs_in_path.display()))?;

    Ok(())
}

fn apply_devappversion_indexjs() -> Result<()> {
    let devappversion = crate::DEVAPPVERSION.clone();
    if devappversion == 0 {
        return Ok(());
    }

    let serverindex_path = pathp(PathE::ServerOutput, "index.js");
    let serverindex_content = fs::read_to_string(&serverindex_path)?;
    let server_regex = Regex::new(r#"APPVERSION = \d+"#).unwrap();
    let serverindex_content = server_regex
        .replace_all(
            &serverindex_content,
            format!("APPVERSION = {}", devappversion).as_str(),
        )
        .to_string();
    fs::write(&serverindex_path, &serverindex_content)?;

    Ok(())
}
