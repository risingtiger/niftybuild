
use std::fs::File;
use std::os::unix::fs::PermissionsExt;
use std::io::Write;
use std::path::PathBuf;
use std::path::Path;
use anyhow::Result;
use std::fs;
use std::env;
use std::os::unix::fs::symlink;

use crate::common_helperfuncs::PathE;
use crate::common_helperfuncs::path;





pub fn initit(instance:&str) -> Result<()> {
    if instance.len() < 3 {
        eprintln!("Error: Instance name must be at least 3 characters long");
        std::process::exit(1);
    }

    let config_file = find_config_file()?;

    let _ = setinstance(instance, &config_file)?;

    handle_defs_symlinks(instance, &config_file)?;

    write_esbuild_config_file()?;
    write_swcrc_file()?;

    Ok(())
}




pub fn setinstance(instance: &str, config_file: &String) -> Result<()> {

    let content = fs::read_to_string(&config_file).unwrap_or_default();
    
    // just update the one line for NIFTY_INSTANCE and leave the rest as is
    let updated_content = content.lines()
        .map(|line| {
            if line.starts_with("export NIFTY_INSTANCE=") {
                format!("export NIFTY_INSTANCE={}", instance)
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<String>>()
        .join("\n");

    fs::write(&config_file, &updated_content)?;

    let config_basename = config_file.split('/').last().unwrap_or_default();
    
    println!("Remember to run 'source ~/{}' to apply changes", config_basename);

    Ok(())
}




fn handle_defs_symlinks(instance: &str, config_file: &String) -> Result<()> {

    let content = fs::read_to_string(&config_file).unwrap_or_default();

    let p = format!("export NIFTY_INSTANCE_{}_SERVER_DIR=", instance.to_uppercase());
    let server = &content.lines()
        .find(|line| line.starts_with(&p))
        .and_then(|line| line.split('=').nth(1))
        .unwrap_or("")
        .trim_matches('"');

    let p = format!("export NIFTY_INSTANCE_{}_CLIENT_DIR=", instance.to_uppercase());
    let client = &content.lines()
        .find(|line| line.starts_with(&p))
        .and_then(|line| line.split('=').nth(1))
        .unwrap_or("")
        .trim_matches('"');

    let instance_server_path = PathBuf::from(server);
    let instance_client_path = PathBuf::from(client);


    let server_path          = path(PathE::ServerSrc);
    let client_path          = path(PathE::ClientSrc);

    symlink(Path::new(&server_path).join("defs.ts"), Path::new(&client_path).join("defs_server_symlink.ts")).unwrap_or_default();
    symlink(Path::new(&server_path).join("defs.ts"), Path::new(&instance_server_path).join("defs_server_symlink.ts")).unwrap_or_default();
    symlink(Path::new(&server_path).join("defs.ts"), Path::new(&instance_client_path).join("defs_server_symlink.ts")).unwrap_or_default();
    symlink(Path::new(&instance_server_path).join("defs.ts"), Path::new(&instance_client_path).join("defs_instance_server_symlink.ts")).unwrap_or_default();
    symlink(Path::new(&client_path).join("defs.ts"), Path::new(&instance_client_path).join("defs_client_symlink.ts")).unwrap_or_default();

    Ok(())
}




pub fn find_config_file() -> Result<String> {
    let home_dir = env::var("HOME").expect("HOME environment variable not set");
    let shell = env::var("SHELL").unwrap_or_default();
    
    // Check if we're on macOS or Linux
    let os_type = if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        "unknown"
    };
    
    // Try to determine the shell type from the SHELL env var
    let shell_type = if shell.contains("zsh") {
        "zsh"
    } else if shell.contains("bash") {
        "bash"
    } else if shell.contains("fish") {
        "fish"
    } else {
        "unknown"
    };
    
    // Check for existing config files in order of preference
    let possible_configs = match (os_type, shell_type) {
        ("macos", "zsh") => vec![
            format!("{}/{}", home_dir, ".zprofile"),
            format!("{}/{}", home_dir, ".zshrc"),
            format!("{}/{}", home_dir, ".profile"),
        ],
        ("macos", "bash") => vec![
            format!("{}/{}", home_dir, ".bash_profile"),
            format!("{}/{}", home_dir, ".bashrc"),
            format!("{}/{}", home_dir, ".profile"),
        ],
        ("linux", "zsh") => vec![
            format!("{}/{}", home_dir, ".zshrc"),
            format!("{}/{}", home_dir, ".profile"),
        ],
        ("linux", "bash") => vec![
            format!("{}/{}", home_dir, ".bashrc"),
            format!("{}/{}", home_dir, ".profile"),
        ],
        ("linux", "fish") => vec![
            format!("{}/{}", home_dir, ".config/fish/config.fish"),
        ],
        _ => vec![
            format!("{}/{}", home_dir, ".profile"),
            format!("{}/{}", home_dir, ".bashrc"),
        ],
    };
    
    // Find the first config file that exists
    for config in possible_configs {
        if fs::metadata(&config).is_ok() {
            return Ok(config);
        }
    }
    
    // If no config file exists, default to .profile and create it if needed
    let default_config = format!("{}/{}", home_dir, ".profile");
    if fs::metadata(&default_config).is_err() {
        fs::write(&default_config, "# Environment variables for Nifty\n")?;
    }
    
    Ok(default_config)
}




fn write_esbuild_config_file() -> Result<()> {

    let tmp_path = path(PathE::TMPDir).to_string_lossy().to_string();

    let esbuild_config = format!(r#"
        import * as esbuild from 'esbuild';
        import {{ env }} from 'process';
        import path from 'path';
        import fs from 'fs';

        const files_instructions_path_str = '{}filestobundle.json'
        const files_instructions          = JSON.parse(fs.readFileSync(files_instructions_path_str, 'utf8'))


        const outdir_str  = files_instructions.pop();
        const entryPoints = files_instructions;


        const buildOptions = {{
            entryPoints,
            platform: 'browser',
            bundle: true, 
            outdir: outdir_str,
            target: 'esnext',
            minify: false, 
            sourcemap: false, 
            loader: {{}},
            define: {{}},
            loader: {{
                '.js': 'ts',
            }},
        }};


        esbuild
            .build(buildOptions)
            .then(() => {{
            }})
            .catch((error) => {{
                console.error('Build failed:', error);
                process.exit(1);
            }});
    "#, tmp_path);

    let config_dir = path(PathE::TMPDirConfigs);
    fs::create_dir_all(&config_dir)?;
    let config_file_path = config_dir.join("esbuild.config.mjs");
    
    let mut file = File::create(&config_file_path)?;
    file.write_all(esbuild_config.as_bytes())?;
    
    let mut perms = fs::metadata(&config_file_path)?.permissions();
    perms.set_mode(0o755); // rwxr-xr-x
    fs::set_permissions(&config_file_path, perms)?;
    
    println!("Created esbuild config at: {}", config_file_path.display());
    
    Ok(())
}




fn write_swcrc_file() -> Result<()> {

    let swcrc = r#"
        {
          "jsc": {
            "parser": {
              "syntax": "typescript",
              "tsx": false,
              "decorators": true,
              "dynamicImport": true
            },
            "target": "esnext",
            "transform": {},
            "externalHelpers": false
          },
          "module": {
            "type": "es6",
            "strict": true,
            "lazy": false,
            "noInterop": false
          },
          "sourceMaps": false,
          "minify": false
        }
    "#;

    let config_dir = path(PathE::TMPDirConfigs);
    fs::create_dir_all(&config_dir)?;
    let swcrc_path = config_dir.join("swcrc");
    
    let mut file = File::create(&swcrc_path)?;
    file.write_all(swcrc.as_bytes())?;
    
    println!("Created swcrc at: {}", swcrc_path.display());
    
    Ok(())
}











