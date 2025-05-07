
use anyhow::Result;
use std::path::Path;
use std::path::PathBuf;
use std::fs;
use glob::glob;
use glob_match::glob_match;
use std::process::Command;



pub enum PathE {
    Configs,
    TMPDir,

    ClientSrc,
    ClientOutputDev,
    ClientOutputDist,

    ServerSrc,
    ServerOutput,
    
    InstanceClientSrc,
    InstanceClientOutputDev,
    InstanceClientOutputDist,
    InstanceClientOutputTMP,

    InstanceServerSrc,
    InstanceServerOutput,

    ChromeOverridesDir,
}






pub fn copy_deep<S: AsRef<str>>(src_path:PathBuf, dest_path: PathBuf, glob_str: &str, glob_skip_str: S) -> Result<()> {
    
    let src_glob_str = format!("{}{}", src_path.to_string_lossy(), glob_str);

    for entry in glob(&src_glob_str)? {
        let path = entry.unwrap();

        if path.is_file() {
            if glob_match(glob_skip_str.as_ref(), path.to_str().unwrap()) {
                continue;
            }

            let relative_path = path.strip_prefix(Path::new(&src_path)).unwrap();
            let destination_path = Path::new(&dest_path).join(relative_path);

            if let Some(parent) = destination_path.parent() {
                fs::create_dir_all(parent)?;
            }

            fs::create_dir_all(&dest_path).unwrap();
            fs::copy(&path, &destination_path).unwrap();
        }
    }

    Ok(())
}




pub fn run_swc(src: PathBuf, dest: PathBuf, glob_files: Vec<&str>) -> Result<()> {

    let src_folder_name         = src.file_name().expect("file name error").to_str().expect("to str error");
    let src_parent_folder_str   = src.parent().expect("parent error").to_str().expect("to str error");
    let dest_path_trimmed       = dest.to_string_lossy().trim_end_matches("/").to_string();
    let swrc_path               = pathp(PathE::Configs,"swcrc");
    let swrc_path               = swrc_path.to_str().expect("to str error");

    let mut commandargs:Vec<String> = vec![
        String::from("swc"), 
        String::from(src_folder_name), 
        String::from("-d"), 
        String::from(dest_path_trimmed), 
        String::from("--config-file"), 
        String::from(swrc_path), 
        String::from("--strip-leading-paths")
    ];

    for arg in glob_files {
        commandargs.push(String::from("--only"));
        commandargs.push(format!("{}/{}", &src_folder_name, arg));
    }

    let _swc_cmd = Command::new("npx").args(commandargs).current_dir(src_parent_folder_str).output().expect("swc chucked an error");

    Ok(())
}




pub fn path(request_path:PathE) -> PathBuf {

    match request_path {
        PathE::Configs => {
            PathBuf::from(crate::CONFIGS_PATH.clone())
        },

        PathE::TMPDir => {
            PathBuf::from(crate::TMP_PATH.clone())
        },

        PathE::ClientSrc => {
            PathBuf::from(crate::MAIN_CLIENT_PATH.clone())
        },

        PathE::ClientOutputDev => {
            PathBuf::from(crate::MAIN_SERVER_PATH.clone() + "static_dev/")
        },

        PathE::ClientOutputDist => {
            PathBuf::from(crate::MAIN_SERVER_PATH.clone() + "static_dist/")
        },

        PathE::ServerSrc => {
            PathBuf::from(crate::MAIN_SERVER_PATH.clone() + "src/")
        },

        PathE::ServerOutput => {
            PathBuf::from(crate::MAIN_SERVER_PATH.clone() + "build/")
        },

        PathE::InstanceClientSrc => {
            PathBuf::from(crate::INSTANCE_CLIENT_PATH.clone())
        },

        PathE::InstanceClientOutputDev => {
            path(PathE::ClientOutputDev).join("instance/")
        },

        PathE::InstanceClientOutputDist => {
            path(PathE::ClientOutputDist).join("instance/")
        },

        PathE::InstanceClientOutputTMP => {
            path(PathE::TMPDir).join("instance/")
        },

        PathE::InstanceServerSrc => {
            PathBuf::from(crate::INSTANCE_SERVER_PATH.clone())
        },

        PathE::InstanceServerOutput => {
            path(PathE::ServerOutput).join("instance/")
        },

        PathE::ChromeOverridesDir => {
            PathBuf::from(crate::CHROME_OVERRIDES_PATH.clone())
        },
    }
}




pub fn pathp<S: AsRef<str>>(request_path:PathE, postpend:S) -> PathBuf {
    let p = path(request_path);
    p.join(postpend.as_ref())
}
















