use anyhow::Result;

use serde_json;
use std::fs::{self};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::common_helperfuncs::path;
use crate::common_helperfuncs::pathp;
use crate::common_helperfuncs::PathE;

pub fn runit() -> Result<()> {
    let tmp_path = path(PathE::TMPDir);
    let dist_path = path(PathE::ClientOutputDist);
    let lazy_path = pathp(PathE::TMPDirFiles, "lazy/");
    let lazy_instance_path = pathp(PathE::InstanceClientOutputTMP, "lazy/");

    let lazy_list = generate_lazy_list(&lazy_path)?;
    let lazy_instance_list = generate_lazy_list(&lazy_instance_path)?;
    let gen_list = set_gen_list()?;
    let mut all_list = vec![];

    all_list.extend(lazy_list);
    all_list.extend(lazy_instance_list);
    all_list.extend(gen_list);
    all_list.push(dist_path.to_path_buf());

    let json_string = serde_json::to_string(&all_list)?;
    let tmp_filestobundle_path = tmp_path.join("filestobundle.json");
    fs::write(&tmp_filestobundle_path, json_string).expect("Failed to write filestobundle.json");

    let _ = esbuild_it();

    Ok(())
}

fn generate_lazy_list(start_path: &Path) -> Result<Vec<PathBuf>> {
    let mut list: Vec<PathBuf> = Vec::new();
    let entries = fs::read_dir(start_path)?;

    for entry in entries {
        let path = entry?.path();

        if !path.is_dir() {
            continue;
        }

        for subentry in fs::read_dir(&path)? {
            let mut subpath = subentry?.path();

            let path_str = subpath.file_name().unwrap().to_str().unwrap().to_string();

            if path_str.ends_with(".js") {
                list.push(subpath);
            } else if subpath.is_dir() {
                let subpath_name = subpath.file_name().unwrap().to_str().unwrap().to_string();
                subpath.push(subpath_name + ".js");
                list.push(subpath);
            }
        }
    }

    Ok(list)
}

fn set_gen_list() -> Result<Vec<PathBuf>> {
    let mut list: Vec<PathBuf> = Vec::new();

    let mainjs = pathp(PathE::TMPDirFiles, "main.js");

    list.push(Path::new(&mainjs).to_path_buf());

    Ok(list)
}

fn esbuild_it() -> Result<(), Box<dyn std::error::Error>> {
    let tmp_path = path(PathE::TMPDir).to_string_lossy().to_string();

    let files_instructions_path = format!("{}/filestobundle.json", tmp_path);

    let files_instructions_content = fs::read_to_string(files_instructions_path)?;
    let mut files_instructions: Vec<String> = serde_json::from_str(&files_instructions_content)?;

    // The last item is the output directory
    let outdir = files_instructions.pop().unwrap();
    let entry_points = files_instructions;

    let mut args = vec![
        "--bundle",
        "--platform=browser",
        "--target=esnext",
        "--minify",
    ];

    let outdirarg = format!("--outdir={}", outdir);

    args.push(&outdirarg);

    args.push("--loader:.js=ts");

    for entry in &entry_points {
        args.push(entry);
    }

    let output = Command::new("esbuild")
        .args(&args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()?;

    if !output.status.success() {
        eprintln!("Build failed");
        std::process::exit(1);
    }

    println!("Build completed successfully");
    Ok(())
}
