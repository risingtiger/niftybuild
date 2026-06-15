use anyhow::Result;

use crate::common_helperfuncs::pathp;
use crate::common_helperfuncs::PathE;

pub fn runit() -> Result<()> {
    let client_deep_copy_media = std::thread::spawn(move || {
        let src = pathp(PathE::ClientSrc, "media/");
        let dest = pathp(PathE::ClientOutputDev, "media/");
        let ignore = format!("{}icons/**/*", src.display());
        crate::common_helperfuncs::copy_deep(src, dest, "**/*", &ignore)
    });

    let instance_deep_copy_media = std::thread::spawn(move || {
        let src = pathp(PathE::InstanceClientSrc, "media/");
        let dest = pathp(PathE::InstanceClientOutputDev, "media/");
        let ignore = String::from("_____");
        crate::common_helperfuncs::copy_deep(src, dest, "**/*", &ignore)
    });

    client_deep_copy_media
        .join()
        .map_err(|e| anyhow::anyhow!("Lazy deep copy media panicked: {:?}", e))??;
    instance_deep_copy_media
        .join()
        .map_err(|e| anyhow::anyhow!("Lazy deep copy media panicked: {:?}", e))??;

    Ok(())
}
