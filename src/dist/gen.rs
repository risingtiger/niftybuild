
use std::time::{SystemTime, UNIX_EPOCH};
use anyhow::Result;
use regex::Regex;
use std::fs::{self}; 
use std::process::Command;

use crate::common_helperfuncs;

use crate::common_helperfuncs::PathE;
use crate::common_helperfuncs::pathp;
use crate::common_helperfuncs::path;




pub fn runit(appversion:u32) -> Result<u32> {

    let _          = process_manifest(appversion)?;
    let _          = process_indexhtml(appversion)?;
    let _          = process_sw(appversion)?;
    let _          = process_thirdparty()?;
    let _          = process_css()?;
    let _          = process_media()?;
    let _          = process_server(appversion)?;

    Ok(appversion)
}




fn process_manifest(appversion:u32) -> Result<()> {

    let manifest_in_path           = pathp(PathE::ClientOutputDev, "app.webmanifest");
    let manifest_out_path          = pathp(PathE::ClientOutputDist, "app.webmanifest");

    let manifest_in_content        = fs::read_to_string(&manifest_in_path)?;

    let version_regex       = Regex::new(r#""version":\s*"(\d+)""#)?;
    let manifest_in_content = version_regex.replace(&manifest_in_content, format!(r#""version": "{}""#, appversion));

    fs::write(&manifest_out_path, manifest_in_content.as_bytes())?;

    Ok(())
}



fn process_indexhtml(appversion:u32) -> Result<()> {

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

    let html_in_str     = pathp(PathE::ClientOutputDev, "index.html");
    let html_out_str    = pathp(PathE::ClientOutputDist, "index.html");
    let html_content    = fs::read_to_string(&html_in_str)?;

    let html_str = html_content.replace("APPVERSION=0", format!("APPVERSION={}", appversion).as_str());
    let html_str = html_str.replace("APPUPDATE_TS=0", format!("APPUPDATE_TS={}", now).as_str());

    fs::write(&html_out_str, &html_str)?;

    Ok(())
}




fn process_sw(appversion:u32) -> Result<()> {

    let sw_in_str     = pathp(PathE::ClientOutputDev, "sw.js");
    let sw_out_str    = pathp(PathE::ClientOutputDist, "sw.js");

    let sw_content  = fs::read_to_string(&sw_in_str)?;

    let sw_str       = sw_content.replace("cacheV__0__", format!("cacheV__{}__", appversion).as_str());

    fs::write(&sw_out_str, &sw_str)?;

    Ok(())
}




fn process_thirdparty() -> Result<()> {

    let in_path     = pathp(PathE::ClientOutputDev, "thirdparty/");
    let out_path    = pathp(PathE::ClientOutputDist, "thirdparty/");

    let in_instance_path     = pathp(PathE::InstanceClientOutputDev, "thirdparty/");
    let out_instance_path    = pathp(PathE::InstanceClientOutputDist, "thirdparty/");

    common_helperfuncs::copy_deep(in_path, out_path, "**/*", "_____").unwrap();
    common_helperfuncs::copy_deep(in_instance_path, out_instance_path, "**/*", "_____").unwrap();

    Ok(())
}




fn process_css() -> Result<()> {

    let main_path         = path(PathE::MainSrc);
    let cssindex_in_str   = pathp(PathE::ClientOutputDev, "index.css");
    let cssindex_out_str  = pathp(PathE::ClientOutputDist, "index.css");
    let cssmain_in_str    = pathp(PathE::ClientOutputDev, "main.css");
    let cssmain_out_str   = pathp(PathE::ClientOutputDist, "main.css");

    let cssindex_cmd      = Command::new("npx").args(["esbuild", cssindex_in_str.to_str().unwrap(), "--bundle", "--loader:.woff2=dataurl"]).current_dir(main_path).output().expect("esbuild chucked an error");
    let cssindex_content  = String::from_utf8(cssindex_cmd.stdout).expect("css stdout error");
    let _                 = fs::write(&cssindex_out_str, &cssindex_content);

    fs::copy(&cssmain_in_str, &cssmain_out_str)?;

    Ok(())
}




fn process_media() -> Result<()> {

    let media_in_path           = pathp(PathE::ClientOutputDev, "media/");
    let media_out_path          = pathp(PathE::ClientOutputDist, "media/");
    let media_instance_in_path  = pathp(PathE::InstanceClientOutputDev, "media/");
    let media_instance_out_path = pathp(PathE::InstanceClientOutputDist, "media/");

    common_helperfuncs::copy_deep(media_in_path, media_out_path, "**/*", "_____").unwrap();
    common_helperfuncs::copy_deep(media_instance_in_path, media_instance_out_path, "**/*", "_____").unwrap();

    Ok(())
}




fn process_server(appversion:u32) -> Result<()> {

    let server_in_path     = pathp(PathE::ServerOutput, "index.js");

    let server_content     = fs::read_to_string(&server_in_path)?;
    let server_regex        = Regex::new(r#"APPVERSION = \d+"#).unwrap();
    let server_content      = server_regex.replace_all(&server_content, format!("APPVERSION = {}", appversion).as_str()).to_string();

    fs::write(&server_in_path, &server_content)?;

    Ok(())
}





