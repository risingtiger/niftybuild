
use std::fs;
use regex::Regex;
//use std::os::unix::fs::symlink;
use anyhow::Result;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

use crate::common_helperfuncs;
use crate::common_helperfuncs::PathE;
use crate::common_helperfuncs::path;
use crate::common_helperfuncs::pathp;

pub mod media;
pub mod thirdparty;
pub mod handlefile;
pub mod server;
pub mod copy_chrome_css_changes;
pub mod setinstance;





#[derive(Serialize, Deserialize, Debug)]
struct ManifestIconT {   purpose: String, src: String, sizes: String  }
#[derive(Serialize, Deserialize, Debug)]
struct ManifestScreenshotT {   src: String, sizes: String, form_factor: String  }
#[derive(Serialize, Deserialize, Debug)]
struct ManifestT {   
    name: String, 
    short_name: String, 
    description: String, 
    theme_color: String, 
    background_color: String, 
    icons: Vec<ManifestIconT>, 
    screenshots: Vec<ManifestScreenshotT>,
    scope: String, 
    start_url: String, 
    version: String, 
    display: String,   
}



pub fn alldev() -> Result<()> {
     
    reset_dev_dirs()?;

    let _  = handle_core()?;

    let lazy_thread            = std::thread::spawn(|| handle_lazy());
    let thirdparty_thread      = std::thread::spawn(|| thirdparty::runit());
    let media_thread           = std::thread::spawn(|| media::runit());
    let server_thread          = std::thread::spawn(|| server::runit());
    let instance_entry_thread  = std::thread::spawn(|| handle_instance_entry());

    lazy_thread.join().map_err(|e| anyhow::anyhow!("Dev Lazy thread panicked: {:?}", e))??;
    thirdparty_thread.join().map_err(|e| anyhow::anyhow!("Dev Thirdparty thread panicked: {:?}", e))??;
    media_thread.join().map_err(|e| anyhow::anyhow!("Dev Media thread panicked: {:?}", e))??;
    server_thread.join().map_err(|e| anyhow::anyhow!("Dev Server thread panicked: {:?}", e))??;
    instance_entry_thread.join().map_err(|e| anyhow::anyhow!("Dev Entry thread panicked: {:?}", e))??;

    Ok(())

    /*
        manifest file
        index.html
        sw.js
        server index.js
    */
}




pub fn handle_core() -> Result<()> {

    let manifest = handle_manifest()?;

    let glob_files = vec!["sw.ts", "main.ts", "alwaysload/**/*", "defs*.ts"];
    let js_client_thread = std::thread::spawn(move || common_helperfuncs::run_swc(path(PathE::ClientSrc), path(PathE::ClientOutputDev), glob_files));

    let glob_files = vec!["main.ts", "alwaysload/**/*", "defs*.ts"];
    let js_instance_thread = std::thread::spawn(move || common_helperfuncs::run_swc(path(PathE::InstanceClientSrc), path(PathE::InstanceClientOutputDev), glob_files));

    let handle_primary_css_files_thread = std::thread::spawn(move || handle_primary_css_files());

    js_client_thread.join().map_err(|e| anyhow::anyhow!("JS thread panicked: {:?}", e))??;
    js_instance_thread.join().map_err(|e| anyhow::anyhow!("JS thread panicked: {:?}", e))??;
    handle_primary_css_files_thread.join().map_err(|e| anyhow::anyhow!("Primary css files thread panicked: {:?}", e))??;

    let _ = handle_indexhtml(&manifest.short_name); // does write over index.html that happens to get created by client_deep_copy_html_css
    let _ = handle_mainjs();

    let devappversion = crate::DEVAPPVERSION.clone();
    if devappversion > 0 {

        let manifest_path = pathp(PathE::ClientOutputDev, "app.webmanifest"); 
        let manifest_content = fs::read_to_string(&manifest_path)?;
        let version_regex       = Regex::new(r#""version":\s*"(\d+)""#)?;
        let manifest_content = version_regex.replace(&manifest_content, format!("\"version\": \"{}\"", devappversion).as_str()).to_string();
        fs::write(&manifest_path, &manifest_content)?;

        let clientindex_path = pathp(PathE::ClientOutputDev, "index.html"); 
        let clientindex_content = fs::read_to_string(&clientindex_path)?;
        let clientindex_content = clientindex_content.replace("APPVERSION=0", format!("APPVERSION={}", devappversion).as_str());
        fs::write(&clientindex_path, &clientindex_content)?;

        let sw_path    = pathp(PathE::ClientOutputDev, "sw.js"); 
        let sw_content = fs::read_to_string(&sw_path)?;
        let sw_content = sw_content.replace("cacheV__0__", format!("cacheV__{}__", devappversion).as_str());
        fs::write(&sw_path, &sw_content)?;
    }

    Ok(())
}




pub fn handle_lazy() -> Result<()> {

    let glob_files = vec!["lazy/**/*"];
    let js_client_thread = std::thread::spawn(move || common_helperfuncs::run_swc(path(PathE::ClientSrc), path(PathE::ClientOutputDev), glob_files));

    let glob_files = vec!["lazy/**/*"];
    let js_instance_thread = std::thread::spawn(move || common_helperfuncs::run_swc(path(PathE::InstanceClientSrc), path(PathE::InstanceClientOutputDev), glob_files));

    let client_deep_copy_except_js = std::thread::spawn(move || common_helperfuncs::copy_deep(pathp(PathE::ClientSrc, "lazy/"), pathp(PathE::ClientOutputDev, "lazy/"), "**/*", "**/*.ts"));

    let instance_deep_copy_except_js = std::thread::spawn(move || common_helperfuncs::copy_deep(pathp(PathE::InstanceClientSrc, "lazy/"), pathp(PathE::InstanceClientOutputDev, "lazy/"), "**/*", "**/*.ts"));

    js_client_thread.join().map_err(|e| anyhow::anyhow!("JS thread panicked: {:?}", e))??;
    js_instance_thread.join().map_err(|e| anyhow::anyhow!("JS thread panicked: {:?}", e))??;
    client_deep_copy_except_js.join().map_err(|e| anyhow::anyhow!("Lazy deep copy except js panicked: {:?}", e))??;
    instance_deep_copy_except_js.join().map_err(|e| anyhow::anyhow!("Lazy deep copy except js panicked: {:?}", e))??;

    Ok(())
}




pub fn handle_corelazy() -> Result<()> {

    let _ = handle_core();
    let _ = handle_lazy();

    Ok(())
}




fn handle_manifest() -> Result<ManifestT> {

    let manifest_in_path            = pathp(PathE::ClientSrc,"app.webmanifest");
    let manifest_instance_in_path   = pathp(PathE::InstanceClientSrc,"app.webmanifest");
    let manifest_out_path           = pathp(PathE::ClientOutputDev,"app.webmanifest");

    let manifest_main               = fs::read_to_string(manifest_in_path).expect("read error manifest file");
    let manifest_instance           = fs::read_to_string(manifest_instance_in_path).expect("read error instance manifest file");

    let mut manifest: ManifestT = serde_json::from_str(&manifest_main).expect("main json error");
    let manifest_instance: ManifestT = serde_json::from_str(&manifest_instance).expect("instance json error");

    manifest.name = manifest_instance.name;
    manifest.short_name = manifest_instance.short_name;
    manifest.description = format!("App Version: {}", manifest_instance.description);
    manifest.icons = manifest_instance.icons;
    manifest.screenshots = manifest_instance.screenshots;
    manifest.theme_color = manifest_instance.theme_color;
    manifest.background_color = manifest_instance.background_color;
    manifest.version = manifest_instance.version;

    let manifest_str = serde_json::to_string_pretty(&manifest).expect("app manifest to string error");
    fs::write(manifest_out_path, manifest_str).expect("write error");

    Ok(manifest)
}




fn handle_indexhtml(manifestname:&str) -> Result<()> {

    let index_in_path           = pathp(PathE::ClientSrc,"index.html");
    let index_out_path          = pathp(PathE::ClientOutputDev,"index.html");

    let index    = fs::read_to_string(index_in_path).expect("read error");
    let index = index.replace("<title></title>", &format!("<title>{}</title>", manifestname));
    fs::write(index_out_path, index).expect("write error");

    Ok(())
}




fn handle_mainjs() -> Result<()> {

    let mainjs_in_path           = pathp(PathE::ClientOutputDev,"main.js");

    let mainjs = fs::read_to_string(&mainjs_in_path).expect("read error");
    let mainjs = mainjs.replace("//{--main_instance.js--}", "import INSTANCE from './instance/main.js';");
    fs::write(&mainjs_in_path, mainjs).expect("mainjs write error");

    Ok(())
}




fn handle_primary_css_files() -> Result<()> {

    // may do more with css, like parsing for specific things (like fonts and sprite sheets and sucking in instance css in the furture

    let index_in_path           = pathp(PathE::ClientSrc,"index.css");
    let index_out_path          = pathp(PathE::ClientOutputDev,"index.css");
    let main_in_path            = pathp(PathE::ClientSrc,"main.css");
    let main_out_path           = pathp(PathE::ClientOutputDev,"main.css");

    fs::copy(&index_in_path, &index_out_path)?;
    fs::copy(&main_in_path, &main_out_path)?;

    Ok(())
}




fn handle_instance_entry() -> Result<()> {
     
    let entry_in    = pathp(PathE::InstanceClientSrc,"entry/");
    let entry_out   = pathp(PathE::InstanceClientOutputDev,"entry/");

    std::fs::create_dir_all(entry_out.clone())?;
    std::fs::copy(entry_in.join("index.html"), entry_out.join("index.html"))?;
    std::fs::copy(entry_in.join("index.css"), entry_out.join("index.css"))?;

    common_helperfuncs::run_swc(entry_in.clone(), entry_out.clone(), vec!["*.ts"])?;

    let devappversion = crate::DEVAPPVERSION.clone();
    if devappversion > 0 {

        let entryindex_path = pathp(PathE::InstanceClientOutputDev, "entry/index.html"); 
        let entryindex_content = fs::read_to_string(&entryindex_path)?;
        let entryindex_content = entryindex_content.replace("APPVERSION=0", format!("APPVERSION={}", devappversion).as_str());
        fs::write(&entryindex_path, &entryindex_content)?;
    }

    Ok(())
}




pub fn handle_file_changed(changed_file:&PathBuf) -> Result<()> {

    let file_meta = handlefile::file_changed(changed_file)?;

    if matches!(file_meta.action, handlefile::FileActionE::Core)  {
        let _ = handle_core();
    } else if matches!(file_meta.action, handlefile::FileActionE::ThirdParty) {
        let _ = thirdparty::runit();
    }

    Ok(())
}




pub fn handle_set_instance(instance:&str) -> Result<()> {
    let _ = setinstance::runit(instance)?;
    Ok(())
}




pub fn handle_set_devappversion(devappversion:&str) -> Result<()> {
    std::fs::create_dir_all("/tmp/niftybuildit")?;
    fs::write("/tmp/niftybuildit/devappversion.txt", devappversion.to_string())?;
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




