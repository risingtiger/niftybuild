
use std::process::Command;
use anyhow::Result;
use std::path::PathBuf;

use crate::common_helperfuncs::PathE;
use crate::common_helperfuncs::pathp;



pub fn runit() -> Result<()> {

    let i = pathp(PathE::ClientSrc,"thirdparty/");
    let j = pathp(PathE::ClientOutputDev,"thirdparty/");

    let client_js = std::thread::spawn(move || handle_thirdparty_js(&i, &j));

    let k = pathp(PathE::InstanceClientSrc,"thirdparty/");
    let l = pathp(PathE::InstanceClientOutputDev,"thirdparty/");

    let instance_js = std::thread::spawn(move || handle_thirdparty_js(&k, &l));

    client_js.join().map_err(|e| anyhow::anyhow!("Thirdparty JS thread panicked: {:?}", e))??;
    instance_js.join().map_err(|e| anyhow::anyhow!("Thirdparty JS thread panicked: {:?}", e))??;

    Ok(())
}




pub fn handle_thirdparty_js(src:&PathBuf, dest:&PathBuf) -> Result<()> {


    let dest_path_trimmed      = dest.to_string_lossy().trim_end_matches('/').to_string();

    let src_str    = format!("{}{}", src.display(), "*");
    let outdir_str = format!("{}{}", "--outdir=", dest_path_trimmed);

    let args = ["esbuild", &src_str, "--bundle", &outdir_str];

    let _cmd = Command::new("npx").args(args).output().expect("ebuild chucked an error on handle_thirdparty_js");

    Ok(())
}

