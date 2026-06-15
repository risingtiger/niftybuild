use anyhow::Result;
use std::path::PathBuf;
use std::process::Command;

use crate::common_helperfuncs::pathp;
use crate::common_helperfuncs::PathE;

pub fn runit() -> Result<()> {
    let i = pathp(PathE::ClientSrc, "thirdparty/");
    let j = pathp(PathE::ClientOutputDev, "thirdparty/");

    let client_js = std::thread::spawn(move || handle_thirdparty_js(&i, &j));

    let k = pathp(PathE::InstanceClientSrc, "thirdparty/");
    let l = pathp(PathE::InstanceClientOutputDev, "thirdparty/");

    let instance_js = std::thread::spawn(move || handle_thirdparty_js(&k, &l));

    client_js
        .join()
        .map_err(|e| anyhow::anyhow!("Thirdparty JS thread panicked: {:?}", e))??;
    instance_js
        .join()
        .map_err(|e| anyhow::anyhow!("Thirdparty JS thread panicked: {:?}", e))??;

    Ok(())
}

pub fn handle_thirdparty_js(src: &PathBuf, dest: &PathBuf) -> Result<()> {
    let dest_path_trimmed = dest.to_string_lossy().trim_end_matches('/').to_string();

    let src_str = format!("{}{}", src.display(), "*");
    let outdir_str = format!("{}{}", "--outdir=", dest_path_trimmed);

    let args = [src_str.as_str(), "--bundle", outdir_str.as_str()];

    let output = Command::new("esbuild").args(args).output().map_err(|e| {
        anyhow::anyhow!(
            "Failed to execute esbuild directly: {}. Put esbuild on PATH",
            e
        )
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        eprintln!("esbuild error for {}", src.display());
        eprintln!("Status: {}", output.status);

        if !stderr.is_empty() {
            eprintln!("stderr:\n{}", stderr);
        }

        if !stdout.is_empty() {
            eprintln!("stdout:\n{}", stdout);
        }

        return Err(anyhow::anyhow!(
            "esbuild failed with status: {}",
            output.status
        ));
    }

    Ok(())
}
