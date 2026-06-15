use anyhow::{Context, Result};
use glob::glob;
use glob_match::glob_match;
use rayon::prelude::*;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

#[derive(Clone, Copy)]
pub enum PathE {
    TMPDir,
    TMPDirFiles,
    TMPDirConfigs,

    ClientSrc,
    ClientOutputDev,
    ClientOutputDist,

    ServerSrc,
    ServerOutput,

    InstanceClientSrc,
    InstanceClientOutputDev,
    InstanceClientOutputDist,
    InstanceClientOutputTMP,

    InstanceServerSrc,
    InstanceServerOutput,

    ChromeOverridesDir,
}

pub fn copy_if_changed(src_path: &Path, dest_path: &Path) -> Result<bool> {
    if let (Ok(src_meta), Ok(dest_meta)) = (fs::metadata(src_path), fs::metadata(dest_path)) {
        if src_meta.len() == dest_meta.len() {
            if let (Ok(src_modified), Ok(dest_modified)) =
                (src_meta.modified(), dest_meta.modified())
            {
                if dest_modified >= src_modified {
                    return Ok(false);
                }
            }
        }
    }

    if let Some(parent) = dest_path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "failed to create destination directory {}",
                parent.display()
            )
        })?;
    }

    fs::copy(src_path, dest_path).with_context(|| {
        format!(
            "failed to copy {} to {}",
            src_path.display(),
            dest_path.display()
        )
    })?;

    Ok(true)
}

pub fn copy_deep<S: AsRef<str>>(
    src_path: PathBuf,
    dest_path: PathBuf,
    glob_str: &str,
    glob_skip_str: S,
) -> Result<()> {
    let src_glob_str = format!("{}{}", src_path.to_string_lossy(), glob_str);
    let glob_skip_str = glob_skip_str.as_ref().to_string();
    let mut copy_jobs: Vec<(PathBuf, PathBuf)> = Vec::new();

    for entry in glob(&src_glob_str)? {
        let path = entry?;

        if !path.is_file() {
            continue;
        }

        if glob_match(&glob_skip_str, path.to_str().unwrap_or_default()) {
            continue;
        }

        let relative_path = path.strip_prefix(Path::new(&src_path)).with_context(|| {
            format!(
                "failed to strip source prefix {} from {}",
                src_path.display(),
                path.display()
            )
        })?;
        let destination_path = Path::new(&dest_path).join(relative_path);
        copy_jobs.push((path, destination_path));
    }

    copy_jobs
        .par_iter()
        .try_for_each(|(src, dest)| copy_if_changed(src, dest).map(|_| ()))?;

    Ok(())
}

pub fn run_swc(src: PathBuf, dest: PathBuf, glob_files: Vec<&str>) -> Result<()> {
    let src_folder_name = src
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .context("failed to determine SWC source folder name")?;
    let src_parent_folder = src
        .parent()
        .context("failed to determine SWC source parent folder")?;
    let dest_path_trimmed = dest.to_string_lossy().trim_end_matches('/').to_string();
    let swrc_path = pathp(PathE::TMPDirConfigs, "swcrc");
    let swrc_path = swrc_path
        .to_str()
        .context("failed to convert SWC config path to string")?;

    let mut swc_args: Vec<String> = vec![
        String::from(src_folder_name),
        String::from("-d"),
        dest_path_trimmed,
        String::from("--config-file"),
        String::from(swrc_path),
        String::from("--strip-leading-paths"),
    ];

    for arg in glob_files {
        swc_args.push(String::from("--only"));
        swc_args.push(format!("{}/{}", &src_folder_name, arg));
    }

    run_swc_cli(src_parent_folder, swc_args)
}

pub fn run_swc_cli(current_dir: &Path, swc_args: Vec<String>) -> Result<()> {
    let output = Command::new("swc")
        .args(&swc_args)
        .current_dir(current_dir)
        .output()
        .with_context(|| {
            format!(
                "failed to execute swc in {}. Install @swc/cli/@swc/core or put swc on PATH",
                current_dir.display()
            )
        })?;

    if !output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let mut message = format!(
            "swc command failed in {}\ncommand: swc {}\nstatus: {}",
            current_dir.display(),
            swc_args.join(" "),
            output.status
        );

        if !stderr.trim().is_empty() {
            message.push_str("\nstderr:\n");
            message.push_str(&stderr);
        }

        if !stdout.trim().is_empty() {
            message.push_str("\nstdout:\n");
            message.push_str(&stdout);
        }

        return Err(anyhow::anyhow!(message));
    }

    Ok(())
}

pub fn path(request_path: PathE) -> PathBuf {
    match request_path {
        PathE::TMPDir => PathBuf::from(crate::TMP_PATH.clone()),

        PathE::TMPDirFiles => path(PathE::TMPDir).join("files/"),

        PathE::TMPDirConfigs => path(PathE::TMPDir).join("configs/"),

        PathE::ClientSrc => PathBuf::from(crate::MAIN_CLIENT_PATH.clone()),

        PathE::ClientOutputDev => PathBuf::from(crate::MAIN_SERVER_PATH.clone() + "static_dev/"),

        PathE::ClientOutputDist => PathBuf::from(crate::MAIN_SERVER_PATH.clone() + "static_dist/"),

        PathE::ServerSrc => PathBuf::from(crate::MAIN_SERVER_PATH.clone() + "src/"),

        PathE::ServerOutput => PathBuf::from(crate::MAIN_SERVER_PATH.clone() + "build/"),

        PathE::InstanceClientSrc => PathBuf::from(crate::INSTANCE_CLIENT_PATH.clone()),

        PathE::InstanceClientOutputDev => path(PathE::ClientOutputDev).join("instance/"),

        PathE::InstanceClientOutputDist => path(PathE::ClientOutputDist).join("instance/"),

        PathE::InstanceClientOutputTMP => path(PathE::TMPDirFiles).join("instance/"),

        PathE::InstanceServerSrc => PathBuf::from(crate::INSTANCE_SERVER_PATH.clone()),

        PathE::InstanceServerOutput => path(PathE::ServerOutput).join("instance/"),

        PathE::ChromeOverridesDir => PathBuf::from(crate::CHROME_OVERRIDES_PATH.clone()),
    }
}

pub fn pathp<S: AsRef<str>>(request_path: PathE, postpend: S) -> PathBuf {
    let p = path(request_path);
    p.join(postpend.as_ref())
}
