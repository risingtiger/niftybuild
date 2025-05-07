
use anyhow::Result;
use std::path::PathBuf;
use std::process::Command;

use crate::common_helperfuncs::PathE;
use crate::common_helperfuncs::pathp;
use crate::common_helperfuncs::path;



enum ClientSideFileTypeE { 
    Isnt,
    Core, 
    Lazy,
    ThirdParty
}


pub enum FileActionE {
    None,
    Core,
    ThirdParty,
}


pub struct GetFileMetaResultT {
    pub action: FileActionE,
}




pub fn file_changed(abs_file_path: &PathBuf) -> Result<GetFileMetaResultT> {

    let swcrc_path_str            = pathp(PathE::MainSrc,".swcrc").display().to_string();
    let mut is_clientside         = false;
    let mut client_side_file_type = ClientSideFileTypeE::Isnt;
    let mut rel_file_path         = PathBuf::new();
    let mut prefix_out_path       = PathBuf::new();

    if abs_file_path.starts_with(path(PathE::ClientSrc)) {
        is_clientside         = true;
        client_side_file_type = get_client_side_file_type(abs_file_path, false);
        rel_file_path         = abs_file_path.strip_prefix(path(PathE::ClientSrc)).unwrap().to_path_buf();
        prefix_out_path       = path(PathE::ClientOutputDev);

    } else if abs_file_path.starts_with(path(PathE::InstanceClientSrc)) {
        is_clientside         = true;
        client_side_file_type = get_client_side_file_type(abs_file_path, true);
        rel_file_path         = abs_file_path.strip_prefix(path(PathE::InstanceClientSrc)).unwrap().to_path_buf();
        prefix_out_path       = path(PathE::InstanceClientOutputDev);

    } else if abs_file_path.starts_with(path(PathE::ServerSrc)) {
        is_clientside = false;
        rel_file_path = abs_file_path.strip_prefix(path(PathE::ServerSrc)).unwrap().to_path_buf();
        prefix_out_path = path(PathE::ServerOutput);

    } else if abs_file_path.starts_with(path(PathE::InstanceServerSrc)) {
        is_clientside = false;
        rel_file_path = abs_file_path.strip_prefix(path(PathE::InstanceServerSrc)).unwrap().to_path_buf();
        prefix_out_path = path(PathE::InstanceServerOutput);
    }


    if is_clientside {

        match client_side_file_type {

            ClientSideFileTypeE::Core => {
                return Ok(GetFileMetaResultT { action: FileActionE::Core });
            },

            ClientSideFileTypeE::Lazy => {

                let ext = abs_file_path.extension().unwrap().to_str().unwrap();

                if ext == "ts" {

                    let path_clone           = rel_file_path.clone();
                    let js_out               = path_clone.with_extension("js");
                    let js_out               = prefix_out_path.join(js_out);
                    let js_out               = js_out.to_str().unwrap();
                    let absolute_path_str    = abs_file_path.clone();
                    let absolute_path_str    = absolute_path_str.to_string_lossy();

                    let _swc_cmd = Command::new("npx").args(["swc", &absolute_path_str, "-o", &js_out, "--config-file", &swcrc_path_str]).output().expect("swc chucked an error at update_file");

                } else if ext == "html" || ext == "css" {
                    let file_out_path = prefix_out_path.join(rel_file_path);
                    std::fs::copy(&abs_file_path, &file_out_path)?;
                }

                return Ok(GetFileMetaResultT { action: FileActionE::None })
            },

            ClientSideFileTypeE::ThirdParty => {
                return Ok(GetFileMetaResultT { action: FileActionE::ThirdParty });
            },

            ClientSideFileTypeE::Isnt => {
                return Ok(GetFileMetaResultT { action: FileActionE::None });
            }
        }
    }

    else { // is server side

        let ext = abs_file_path.extension().unwrap().to_str().unwrap();

        if ext == "ts" {
            let path_clone           = rel_file_path.clone();
            let js_out               = path_clone.with_extension("js");
            let js_out               = prefix_out_path.join(js_out);
            let js_out               = js_out.to_str().unwrap();
            let absolute_path_str    = abs_file_path.clone();
            let absolute_path_str    = absolute_path_str.to_string_lossy();

            let _swc_cmd             = Command::new("npx").args(["swc", &absolute_path_str, "-o", &js_out, "--config-file", &swcrc_path_str]).output().expect("swc chucked an error at update_file");
        }

        return Ok(GetFileMetaResultT { action: FileActionE::None })
    }
}




fn get_client_side_file_type(abs_file_path: &PathBuf, is_instance:bool) -> ClientSideFileTypeE {

    let beg = if is_instance { path(PathE::InstanceClientSrc) } else { path(PathE::ClientSrc) };

    if abs_file_path.starts_with(beg.clone().join("lazy")) {
        return ClientSideFileTypeE::Lazy;
    } else if abs_file_path.starts_with(beg.clone().join("thirdparty")) {
        return ClientSideFileTypeE::ThirdParty;
    } else {
        return ClientSideFileTypeE::Core;
    }
}



