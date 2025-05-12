
use anyhow::Result;

use std::fs::{self};
use std::path::{Path, PathBuf};
use serde_json;
use std::process::Command;

use crate::common_helperfuncs::PathE;
use crate::common_helperfuncs::path;
use crate::common_helperfuncs::pathp;





pub fn runit() -> Result<()> {

    //let instance_name      = crate::INSTANCE_DIR_NAME.clone();
    let tmp_path           = path(PathE::TMPDirConfigs);
    let dist_path          = path(PathE::ClientOutputDist);
    let lazy_path          = pathp(PathE::TMPDir, "lazy/");
    let lazy_instance_path = pathp(PathE::InstanceClientOutputTMP, "lazy/");
       
    let lazy_list          = generate_lazy_list(&lazy_path)?;
    let lazy_instance_list = generate_lazy_list(&lazy_instance_path)?;
    let gen_list           = set_gen_list()?;
    let mut all_list       = vec![];

    all_list.extend(lazy_list);
    all_list.extend(lazy_instance_list);
    all_list.extend(gen_list);
    all_list.push(dist_path.to_path_buf());

    let json_string            = serde_json::to_string(&all_list)?;
    let tmp_filestobundle_path = tmp_path.join("filestobundle.json");
    fs::write(&tmp_filestobundle_path, json_string)?;

    let esbuild_cmd = Command::new("node").args(["esbuild.config.mjs"]).current_dir(&tmp_path).output().expect("esbuild chucked an error");

    if !esbuild_cmd.status.success() {
        if !esbuild_cmd.stderr.is_empty() {
            eprintln!("esbuild error: {}", String::from_utf8_lossy(&esbuild_cmd.stderr));
        }
        if !esbuild_cmd.stdout.is_empty() {
            eprintln!("esbuild: {}", String::from_utf8_lossy(&esbuild_cmd.stderr));
        }
        eprintln!("esbuild command failed with exit code: {:?}", esbuild_cmd.status.code());
    }

    Ok(())
}



fn generate_lazy_list(start_path: &Path) -> Result<Vec<PathBuf>> {

    let mut list:Vec<PathBuf> = Vec::new();
    let entries = fs::read_dir(start_path)?;

    for entry in entries {

        let path = entry?.path();

        if !path.is_dir() {   continue;   }
    
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

    let mut list:Vec<PathBuf> = Vec::new();

    let mainjs = pathp(PathE::TMPDirFiles, "main.js");

    list.push(Path::new(&mainjs).to_path_buf());

    Ok(list)
}






