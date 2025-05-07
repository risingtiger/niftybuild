
use anyhow::Result;
use std::fs;
use std::fs::{copy, remove_file};
use regex::Regex;





mod initial_js;
mod bundle_js;
mod entry;
mod gen;
mod brotli;

use crate::common_helperfuncs::PathE;
use crate::common_helperfuncs::path;
use crate::common_helperfuncs::pathp;





struct ProcessedStatsT {
    js_files_count: u32,
    html_files_count: u32,
    css_files_count: u32,
    lines_of_js: u32,
}





pub fn runit() -> Result<()> {

    reset_dist_dirs()?;

    let mut stats = ProcessedStatsT {
        js_files_count: 0,
        html_files_count: 0,
        css_files_count: 0,
        lines_of_js: 0,
    };

    let appversion = iterate_manifest_appversion()?;
    let _          = initial_js::runit(&mut stats);
    let _          = handle_defs_files();
    let _          = bundle_js::runit();
    let _          = gen::runit(appversion)?;
    let _          = entry::runit(appversion);
    let _          = brotli::runit();

    println!("APPVersion {}", appversion);
    println!("Processed {} JS files", stats.js_files_count);
    println!("Processed {} HTML files", stats.html_files_count);
    println!("Processed {} CSS files", stats.css_files_count);
    println!("Processed {} lines of JS", stats.lines_of_js);

    Ok(())
}




fn iterate_manifest_appversion() -> Result<u32> {

    let manifest_path     = pathp(PathE::InstanceClientSrc,"app.webmanifest");
    let manifest_content  = fs::read_to_string(&manifest_path).unwrap();

    let manifest_regex          = Regex::new(r#""version":\s*"(\d+)""#).unwrap();
    let manifest_regex_captures = manifest_regex.captures(&manifest_content).unwrap();
    let version                 = manifest_regex_captures.get(1).unwrap().as_str().parse::<u32>().unwrap();
    let next_version            = version + 1;

    let manifest_content       = manifest_regex.replace_all(&manifest_content, format!("\"version\":\"{}\"", next_version).as_str()).to_string();

    fs::write(&manifest_path, &manifest_content)?;

    Ok(next_version)
}



fn reset_dist_dirs() -> Result<()> {

    let _xx = std::fs::remove_dir_all(path(PathE::ClientOutputDist));
    let _   = std::fs::remove_dir_all(crate::TMP_PATH.clone());

    std::fs::create_dir_all(path(PathE::ClientOutputDist))?;
    std::fs::create_dir_all(path(PathE::InstanceClientOutputDist))?;

    Ok(())
}




fn handle_defs_files() -> Result<()> {

    remove_file(pathp(PathE::TMPDir,"defs.js")).unwrap();
    remove_file(pathp(PathE::TMPDir,"defs_server_symlink.js")).unwrap();
    remove_file(pathp(PathE::InstanceClientOutputTMP,"defs.js")).unwrap();
    remove_file(pathp(PathE::InstanceClientOutputTMP,"defs_client_symlink.js")).unwrap();
    remove_file(pathp(PathE::InstanceClientOutputTMP,"defs_server_symlink.js")).unwrap();
    remove_file(pathp(PathE::InstanceClientOutputTMP,"defs_instance_server_symlink.js")).unwrap();

    copy(pathp(PathE::ClientSrc,"defs.ts"), pathp(PathE::TMPDir,"defs.ts")).unwrap();
    copy(pathp(PathE::ServerSrc,"defs.ts"), pathp(PathE::TMPDir,"defs_server_symlink.ts")).unwrap();
    copy(pathp(PathE::InstanceClientSrc,"defs.ts"), pathp(PathE::InstanceClientOutputTMP,"defs.ts")).unwrap();
    copy(pathp(PathE::ClientSrc,"defs.ts"), pathp(PathE::InstanceClientOutputTMP,"defs_client_symlink.ts")).unwrap();
    copy(pathp(PathE::ServerSrc,"defs.ts"), pathp(PathE::InstanceClientOutputTMP,"defs_server_symlink.ts")).unwrap();
    copy(pathp(PathE::InstanceServerSrc,"defs.ts"), pathp(PathE::InstanceClientOutputTMP,"defs_instance_server_symlink.ts")).unwrap();


    Ok(())
}




