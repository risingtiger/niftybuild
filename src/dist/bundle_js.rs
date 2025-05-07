
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
    let tmp_path           = path(PathE::TMPDir);
    let main_path          = crate::MAIN_PATH.clone();
    let main_path          = Path::new(&main_path);
    let dist_path          = path(PathE::ClientOutputDist);
    let lazy_path          = pathp(PathE::TMPDir, "lazy");
    let lazy_instance_path = pathp(PathE::InstanceClientOutputTMP, "lazy");
       
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
    fs::write(tmp_filestobundle_path, json_string)?;
    
    let esbuild_cmd = Command::new("node").args(["esbuild.config.mjs"]).current_dir(&main_path).output().expect("esbuild chucked an error");
    let _           = String::from_utf8(esbuild_cmd.stdout).expect("js stdout error");

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

    let main_client_str = crate::TMP_PATH.clone();

    let mainjs = main_client_str.clone() + "main.js";

    list.push(Path::new(&mainjs).to_path_buf());

    Ok(list)
}




fn write_esbuild_config_file() -> Result<()> {

    let esbuild_config = r#"

        import * as esbuild from 'esbuild';
        import { env } from 'process';
        import path from 'path';
        import fs from 'fs';

        const files_instructions_path_str = '/tmp/niftybuildit/filestobundle.json'
        const files_instructions          = JSON.parse(fs.readFileSync(files_instructions_path_str, 'utf8'))
        //const files_instructions          = JSON.parse('["/Users/dave/Code/nifty/server/static_dev/main_test.js","/Users/dave/Code/nifty/server/static_dist/"]')


        const outdir_str  = files_instructions.pop();
        const entryPoints = files_instructions;


        /*
        const ENTRY_POINTS = [
            'server/static_dev/main.min.js', 
        ];

        const OUTDIR = 'server/static_dist';
        */

        const buildOptions = {
            entryPoints,
            platform: 'browser',
            bundle: true, 
            outdir: outdir_str,
            target: 'esnext',
            minify: false, 
            sourcemap: false, 
            loader: {},
            define: {},
            loader: {
                '.js': 'ts',
            },
        };


        esbuild
            .build(buildOptions)
            .then(() => {
            })
            .catch((error) => {
                console.error('Build failed:', error);
                process.exit(1);
            });
"#;

    Ok(())
}



