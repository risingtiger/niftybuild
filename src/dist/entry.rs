

use std::time::{SystemTime, UNIX_EPOCH};
use std::fs;
use anyhow::Result;


use crate::common_helperfuncs::pathp;
use crate::common_helperfuncs::PathE;



pub fn runit(appversion:u32) -> Result<()> {

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

    let entry_in   = pathp(PathE::InstanceClientOutputDev,"entry/");
    let entry_out  = pathp(PathE::InstanceClientOutputDist,"entry/");
    let html_in    = entry_in.join("index.html");
    let html_out   = entry_out.join("index.html");
    let js_in      = entry_in.join("index.js");
    let css_in     = entry_in.join("index.css");

    let html       = fs::read_to_string(html_in).expect("read error");
    let js         = fs::read_to_string(js_in).expect("read error");
    let css        = fs::read_to_string(css_in).expect("read error");

    let js_tag = format!(r#"<script type="module">{}</script>"#, js);
    let html = html.replace(r#"<script src="/entry/index.js" type="module"></script>"#, &js_tag);

    let css_tag = format!(r#"<style>{}</style>"#, css);
    let html = html.replace(r#"<link rel="stylesheet" href="/entry/index.css">"#, &css_tag);

    let html = html.replace("APPVERSION=0", format!("APPVERSION={}", appversion).as_str());
    let html = html.replace("APPUPDATE_TS=0", format!("APPUPDATE_TS={}", now).as_str());

    fs::create_dir_all(entry_out).expect("create dir error");
    fs::write(&html_out, html).expect("write error");

    Ok(())
}

