
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

use crate::common_helperfuncs::PathE;
use crate::common_helperfuncs::path;








pub fn runit() -> Vec<PathBuf> {

    let overrides_folder = path(PathE::ChromeOverridesDir);
    let port = crate::HTTP_PORT.clone();
    let instance_folder = path(PathE::InstanceClientSrc);
    let main_folder = path(PathE::ClientSrc);
    let path = format!("{}localhost:{}/assets/", overrides_folder.display(), port);
    let mut affected_paths:Vec<PathBuf> = vec![];

    for entry in WalkDir::new(&path) {
        let entry = entry.unwrap();
        let path = entry.path();

        if entry.file_type().is_dir() {
            continue;
        }

        if let Some(extension) = path.extension() {
            if extension.eq_ignore_ascii_case("css") {
                let path = process_file(&path.to_path_buf(), &main_folder, &instance_folder, &port);
                affected_paths.push(path);
            }
        }
    }

    affected_paths
}




fn process_file(file:&PathBuf, main_folder:&PathBuf, instance_folder:&PathBuf, port:&String) -> PathBuf { 

    let path_str           = file.to_string_lossy();
    let instance_split_str = format!("localhost:{}/assets/instance/", &port);
    let main_split_str     = format!("localhost:{}/assets/", &port);

    if path_str.contains(&instance_split_str) {
        let s          = instance_split_str.as_str();
        let split_path = path_str.split(s).last().unwrap_or("");
        let new_path   = format!("{}{}", instance_folder.display(), split_path);

        let _ = fs::copy(&file, &new_path);
        fs::remove_file(file).unwrap();

        return PathBuf::from(new_path);
    }

    else {
        let s          = main_split_str.as_str();
        let split_path = path_str.split(s).last().unwrap_or("");
        let new_path   = format!("{}{}", main_folder.display(), split_path);

        let _ = fs::copy(&file, &new_path);
        fs::remove_file(file).unwrap();

        return PathBuf::from(new_path);
    }
}




