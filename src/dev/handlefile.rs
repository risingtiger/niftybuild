use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

use crate::common_helperfuncs::copy_if_changed;
use crate::common_helperfuncs::path;
use crate::common_helperfuncs::pathp;
use crate::common_helperfuncs::run_swc_cli;
use crate::common_helperfuncs::PathE;

enum ClientSideFileTypeE {
    Core,
    Lazy,
    ThirdParty,
}

struct CompileJob {
    src: PathBuf,
    dest: PathBuf,
}

pub fn file_changed(abs_file_path: &PathBuf) -> Result<()> {
    let swcrc_path_str = pathp(PathE::TMPDirConfigs, "swcrc").display().to_string();

    let client_src = path(PathE::ClientSrc);
    let instance_client_src = path(PathE::InstanceClientSrc);
    let server_src = path(PathE::ServerSrc);
    let instance_server_src = path(PathE::InstanceServerSrc);

    if abs_file_path.starts_with(&client_src) {
        let rel_file_path = abs_file_path.strip_prefix(&client_src)?.to_path_buf();
        let client_side_file_type = get_client_side_file_type(abs_file_path, false);
        handle_client_file(
            abs_file_path,
            &rel_file_path,
            path(PathE::ClientOutputDev),
            false,
            client_side_file_type,
            &swcrc_path_str,
        )?;
    } else if abs_file_path.starts_with(&instance_client_src) {
        let rel_file_path = abs_file_path
            .strip_prefix(&instance_client_src)?
            .to_path_buf();
        let client_side_file_type = get_client_side_file_type(abs_file_path, true);
        handle_client_file(
            abs_file_path,
            &rel_file_path,
            path(PathE::InstanceClientOutputDev),
            true,
            client_side_file_type,
            &swcrc_path_str,
        )?;
    } else if abs_file_path.starts_with(&server_src) {
        let rel_file_path = abs_file_path.strip_prefix(&server_src)?.to_path_buf();
        handle_server_file(
            abs_file_path,
            &rel_file_path,
            path(PathE::ServerOutput),
            false,
            &swcrc_path_str,
        )?;
    } else if abs_file_path.starts_with(&instance_server_src) {
        let rel_file_path = abs_file_path
            .strip_prefix(&instance_server_src)?
            .to_path_buf();
        handle_server_file(
            abs_file_path,
            &rel_file_path,
            path(PathE::InstanceServerOutput),
            true,
            &swcrc_path_str,
        )?;
    }

    Ok(())
}

fn handle_client_file(
    abs_file_path: &PathBuf,
    rel_file_path: &PathBuf,
    prefix_out_path: PathBuf,
    is_instance: bool,
    client_side_file_type: ClientSideFileTypeE,
    swcrc_path: &str,
) -> Result<()> {
    match client_side_file_type {
        ClientSideFileTypeE::Core => {
            handle_client_core_file(
                abs_file_path,
                rel_file_path,
                &prefix_out_path,
                is_instance,
                swcrc_path,
            )?;
        }
        ClientSideFileTypeE::Lazy => {
            handle_lazy_file(abs_file_path, rel_file_path, &prefix_out_path, swcrc_path)?;
        }
        ClientSideFileTypeE::ThirdParty => {
            handle_thirdparty_file(is_instance)?;
        }
    }

    Ok(())
}

fn handle_client_core_file(
    abs_file_path: &PathBuf,
    rel_file_path: &PathBuf,
    prefix_out_path: &PathBuf,
    is_instance: bool,
    swcrc_path: &str,
) -> Result<()> {
    let ext = extension(abs_file_path);

    if ext == "ts" {
        if rel_file_path == Path::new("main.ts") {
            refresh_main_js_after_change(is_instance, swcrc_path)?;
        } else if is_defs_ts(rel_file_path) {
            handle_client_defs_file(abs_file_path, rel_file_path, is_instance, swcrc_path)?;
        } else if is_core_ts_build_input(rel_file_path, is_instance) {
            compile_ts_to_output(abs_file_path, rel_file_path, prefix_out_path, swcrc_path)?;

            if !is_instance && rel_file_path == Path::new("sw.ts") {
                super::apply_devappversion_sw()?;
            }
        }

        return Ok(());
    }

    if rel_file_path == Path::new("app.webmanifest") || rel_file_path == Path::new("index.html") {
        super::refresh_manifest_and_indexhtml()?;
    } else if rel_file_path == Path::new("main.json") {
        refresh_main_js_after_change(false, swcrc_path)?;
    } else if is_core_css_build_input(rel_file_path, is_instance) {
        copy_file_to_output(abs_file_path, rel_file_path, prefix_out_path)?;
    }

    Ok(())
}

fn handle_lazy_file(
    abs_file_path: &PathBuf,
    rel_file_path: &PathBuf,
    prefix_out_path: &PathBuf,
    swcrc_path: &str,
) -> Result<()> {
    if extension(abs_file_path) == "ts" {
        compile_ts_to_output(abs_file_path, rel_file_path, prefix_out_path, swcrc_path)?;
    } else {
        copy_file_to_output(abs_file_path, rel_file_path, prefix_out_path)?;
    }

    Ok(())
}

fn handle_thirdparty_file(is_instance: bool) -> Result<()> {
    let (src, dest) = if is_instance {
        (
            pathp(PathE::InstanceClientSrc, "thirdparty/"),
            pathp(PathE::InstanceClientOutputDev, "thirdparty/"),
        )
    } else {
        (
            pathp(PathE::ClientSrc, "thirdparty/"),
            pathp(PathE::ClientOutputDev, "thirdparty/"),
        )
    };

    super::thirdparty::handle_thirdparty_js(&src, &dest)
}

fn handle_server_file(
    abs_file_path: &PathBuf,
    rel_file_path: &PathBuf,
    prefix_out_path: PathBuf,
    is_instance: bool,
    swcrc_path: &str,
) -> Result<()> {
    if extension(abs_file_path) != "ts" {
        return Ok(());
    }

    if is_defs_ts(rel_file_path) {
        handle_server_defs_file(abs_file_path, rel_file_path, is_instance, swcrc_path)?;
    } else {
        compile_ts_to_output(abs_file_path, rel_file_path, &prefix_out_path, swcrc_path)?;

        if !is_instance && rel_file_path == Path::new("index.ts") {
            super::server::postprocess_indexjs()?;
        }
    }

    Ok(())
}

fn handle_client_defs_file(
    abs_file_path: &PathBuf,
    rel_file_path: &PathBuf,
    is_instance: bool,
    swcrc_path: &str,
) -> Result<()> {
    let mut jobs = vec![compile_job_for_rel(
        abs_file_path,
        rel_file_path,
        if is_instance {
            path(PathE::InstanceClientOutputDev)
        } else {
            path(PathE::ClientOutputDev)
        },
    )];

    if !is_instance && rel_file_path == Path::new("defs.ts") {
        let instance_symlink = pathp(PathE::InstanceClientSrc, "defs_client_symlink.ts");
        if instance_symlink.exists() {
            jobs.push(compile_job_for_rel(
                &instance_symlink,
                &PathBuf::from("defs_client_symlink.ts"),
                path(PathE::InstanceClientOutputDev),
            ));
        }
    }

    compile_jobs_in_parallel(jobs, swcrc_path)
}

fn handle_server_defs_file(
    abs_file_path: &PathBuf,
    rel_file_path: &PathBuf,
    is_instance: bool,
    swcrc_path: &str,
) -> Result<()> {
    let mut jobs = vec![compile_job_for_rel(
        abs_file_path,
        rel_file_path,
        if is_instance {
            path(PathE::InstanceServerOutput)
        } else {
            path(PathE::ServerOutput)
        },
    )];

    if !is_instance && rel_file_path == Path::new("defs.ts") {
        let client_symlink = pathp(PathE::ClientSrc, "defs_server_symlink.ts");
        if client_symlink.exists() {
            jobs.push(compile_job_for_rel(
                &client_symlink,
                &PathBuf::from("defs_server_symlink.ts"),
                path(PathE::ClientOutputDev),
            ));
        }

        let instance_client_symlink = pathp(PathE::InstanceClientSrc, "defs_server_symlink.ts");
        if instance_client_symlink.exists() {
            jobs.push(compile_job_for_rel(
                &instance_client_symlink,
                &PathBuf::from("defs_server_symlink.ts"),
                path(PathE::InstanceClientOutputDev),
            ));
        }
    } else if is_instance && rel_file_path == Path::new("defs.ts") {
        let instance_client_symlink =
            pathp(PathE::InstanceClientSrc, "defs_instance_server_symlink.ts");
        if instance_client_symlink.exists() {
            jobs.push(compile_job_for_rel(
                &instance_client_symlink,
                &PathBuf::from("defs_instance_server_symlink.ts"),
                path(PathE::InstanceClientOutputDev),
            ));
        }
    }

    compile_jobs_in_parallel(jobs, swcrc_path)
}

fn refresh_main_js_after_change(changed_instance_main: bool, swcrc_path: &str) -> Result<()> {
    let client_main_src = pathp(PathE::ClientSrc, "main.ts");
    let client_main_out = pathp(PathE::ClientOutputDev, "main.js");
    let instance_main_src = pathp(PathE::InstanceClientSrc, "main.ts");
    let instance_main_out = pathp(PathE::InstanceClientOutputDev, "main.js");

    let mut jobs = vec![CompileJob {
        src: client_main_src,
        dest: client_main_out,
    }];

    if changed_instance_main || !instance_main_out.exists() {
        jobs.push(CompileJob {
            src: instance_main_src,
            dest: instance_main_out,
        });
    }

    compile_jobs_in_parallel(jobs, swcrc_path)?;
    super::handle_mainjs_n_json()
}

fn compile_ts_to_output(
    abs_file_path: &PathBuf,
    rel_file_path: &PathBuf,
    prefix_out_path: &PathBuf,
    swcrc_path: &str,
) -> Result<()> {
    let job = compile_job_for_rel(abs_file_path, rel_file_path, prefix_out_path.to_path_buf());
    compile_single_ts(&job.src, &job.dest, swcrc_path)
}

fn compile_job_for_rel(
    abs_file_path: &PathBuf,
    rel_file_path: &PathBuf,
    prefix_out_path: PathBuf,
) -> CompileJob {
    let mut js_rel_path = rel_file_path.clone();
    js_rel_path.set_extension("js");

    CompileJob {
        src: abs_file_path.clone(),
        dest: prefix_out_path.join(js_rel_path),
    }
}

fn compile_jobs_in_parallel(jobs: Vec<CompileJob>, swcrc_path: &str) -> Result<()> {
    let handles = jobs
        .into_iter()
        .map(|job| {
            let swcrc_path = swcrc_path.to_string();
            std::thread::spawn(move || compile_single_ts(&job.src, &job.dest, &swcrc_path))
        })
        .collect::<Vec<_>>();

    for handle in handles {
        handle
            .join()
            .map_err(|e| anyhow::anyhow!("SWC file thread panicked: {:?}", e))??;
    }

    Ok(())
}

fn compile_single_ts(
    abs_file_path: &PathBuf,
    js_out_path: &PathBuf,
    swcrc_path: &str,
) -> Result<()> {
    if let Some(dir) = js_out_path.parent() {
        std::fs::create_dir_all(dir)?;
    }

    let parent_dir = abs_file_path.parent().with_context(|| {
        format!(
            "failed to determine parent directory for {}",
            abs_file_path.display()
        )
    })?;
    let js_out = js_out_path.to_str().with_context(|| {
        format!(
            "failed to convert output path {} to string",
            js_out_path.display()
        )
    })?;

    run_swc_cli(
        parent_dir,
        vec![
            abs_file_path.to_string_lossy().to_string(),
            String::from("-o"),
            String::from(js_out),
            String::from("--config-file"),
            String::from(swcrc_path),
        ],
    )
    .with_context(|| {
        format!(
            "failed to compile changed TypeScript file {} to {}",
            abs_file_path.display(),
            js_out_path.display()
        )
    })
}

fn copy_file_to_output(
    abs_file_path: &PathBuf,
    rel_file_path: &PathBuf,
    prefix_out_path: &PathBuf,
) -> Result<()> {
    let file_out_path = prefix_out_path.join(rel_file_path);
    copy_if_changed(abs_file_path, &file_out_path).map(|_| ())
}

fn get_client_side_file_type(abs_file_path: &PathBuf, is_instance: bool) -> ClientSideFileTypeE {
    let beg = if is_instance {
        path(PathE::InstanceClientSrc)
    } else {
        path(PathE::ClientSrc)
    };

    if abs_file_path.starts_with(beg.clone().join("lazy")) {
        ClientSideFileTypeE::Lazy
    } else if abs_file_path.starts_with(beg.clone().join("thirdparty")) {
        ClientSideFileTypeE::ThirdParty
    } else {
        ClientSideFileTypeE::Core
    }
}

fn is_core_ts_build_input(rel_file_path: &Path, is_instance: bool) -> bool {
    rel_file_path == Path::new("main.ts")
        || (!is_instance
            && (rel_file_path == Path::new("sw.ts")
                || rel_file_path == Path::new("shared_worker.ts")))
        || rel_file_path.starts_with("alwaysload")
}

fn is_core_css_build_input(rel_file_path: &Path, is_instance: bool) -> bool {
    (!is_instance
        && (rel_file_path == Path::new("index.css") || rel_file_path == Path::new("main.css")))
        || (is_instance && rel_file_path == Path::new("icons.css"))
}

fn is_defs_ts(rel_file_path: &Path) -> bool {
    rel_file_path
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .map(|file_name| file_name.starts_with("defs") && file_name.ends_with(".ts"))
        .unwrap_or(false)
}

fn extension(path: &Path) -> &str {
    path.extension().and_then(|ext| ext.to_str()).unwrap_or("")
}
