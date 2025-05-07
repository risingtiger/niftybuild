
use anyhow::Result;
use std::process::Command;
use std::fs;

use crate::common_helperfuncs::PathE;
use crate::common_helperfuncs::pathp;




pub fn runit() -> Result<()> {

    let client_deep_copy_media = std::thread::spawn(move || {
        let src = pathp(PathE::ClientSrc,"media/");
        let dest = pathp(PathE::ClientOutputDev,"media/");
        let ignore = format!("{}icons/**/*", src.display());
        crate::common_helperfuncs::copy_deep(src, dest, "**/*", &ignore)
    });

    let instance_deep_copy_media = std::thread::spawn(move || {
        let src = pathp(PathE::InstanceClientSrc,"media/");
        let dest = pathp(PathE::InstanceClientOutputDev,"media/");
        let ignore = String::from("_____");
        crate::common_helperfuncs::copy_deep(src, dest, "**/*", &ignore)
    });

    client_deep_copy_media.join().map_err(|e| anyhow::anyhow!("Lazy deep copy media panicked: {:?}", e))??;
    instance_deep_copy_media.join().map_err(|e| anyhow::anyhow!("Lazy deep copy media panicked: {:?}", e))??;

    Ok(())
}




pub fn iconsfont() -> Result<()> {

    let media_in              = pathp(PathE::ClientSrc, "media/");
    let icons_in              = media_in.join("icons/");
    let icons_out             = media_in.join("iconsfont/");

    fs::create_dir_all(&icons_out)?;

    let icons_in_str          = icons_in.to_string_lossy();
    let icons_out_str         = icons_out.to_string_lossy();

    let fantasticonargs = ["fantasticon", &icons_in_str, "-n", "icons", "-t", "woff2", "-o", &icons_out_str];
    let _fantasticon    = Command::new("npx").args(fantasticonargs).output().expect("iconsfont fantasticon chucked an error on main media");


    Ok(())
}




