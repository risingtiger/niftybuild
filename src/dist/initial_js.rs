
use anyhow::Result;

use std::fs::{self}; //, File
use std::path::{Path, PathBuf};
use minifier::js::minify;
use walkdir::WalkDir;

use super::ProcessedStatsT;

use crate::common_helperfuncs::PathE;
use crate::common_helperfuncs::path;
use crate::common_helperfuncs::pathp;




pub fn runit(stats: &mut ProcessedStatsT) -> Result<()> {

    let client_output_dev_str        = path(PathE::ClientOutputDev);
    let client_output_dev_path       = Path::new(&client_output_dev_str);
    let mut file_paths: Vec<PathBuf> = Vec::new();

    for entry in WalkDir::new(client_output_dev_path).into_iter().filter_entry(|e| is_skip_entry(e)).filter_map(|e| e.ok()) {

        let path = entry.path();

        if entry.file_type().is_file() {
            file_paths.push(path.to_path_buf());
        }
    }

    file_paths.into_iter().for_each(|p| {
        let _ = process_file(&p, stats);
    });

    Ok(())
}




fn process_file(file_in_path:&PathBuf, stats:&mut ProcessedStatsT) -> Result<()> {

    let extension      = file_in_path.extension().unwrap_or_default();

    if extension == "js" {
        let _ = process_js(file_in_path, stats);
    } else if extension == "html" {
        stats.html_files_count += 1;
    } else if extension == "css" {
        stats.css_files_count += 1;
    } else {
    }

    Ok(())
}




fn process_js(file_in_path:&PathBuf, stats:&mut ProcessedStatsT) -> Result<()> {

    let js_file_str = fs::read_to_string(file_in_path).unwrap_or_else(|_| String::from(" "));

    if 
        file_in_path.components().any(|component| component.as_os_str() == "lazy") && 
        (file_in_path.components().any(|component| component.as_os_str() == "views" || component.as_os_str() == "components")   ) 
    {
        let combined_str = process_js_html_css_combined(&js_file_str, &file_in_path)?;
        let minified_str = process_minify_js(&combined_str).unwrap();
        let _            = write_js(&file_in_path, &minified_str);
    }

    else {
        let minified_str = process_minify_js(&js_file_str).unwrap();
        let _            = write_js(&file_in_path, &minified_str);
    }

    stats.js_files_count += 1;
    stats.lines_of_js += js_file_str.lines().count() as u32;

    Ok(())
}




fn process_js_html_css_combined(js_file_str: &String, file_in_path: &Path) -> Result<String> {

    let css_file_str  = fs::read_to_string(file_in_path.with_extension("css")).unwrap_or_else(|_| String::from(" "));
    let html_file_str = fs::read_to_string(file_in_path.with_extension("html")).unwrap_or_else(|_| String::from(" "));

    let import_statements = process_js_parts_import_statements(file_in_path);

    let mut updated_js_file_str = String::with_capacity(
        js_file_str.len() + css_file_str.len() + 20 + 56 + html_file_str.len() + import_statements.len(),
    );
    
    let mut html_replacement_str = String::with_capacity( html_file_str.len() + 56);
    let mut css_replacement_str  = String::with_capacity( css_file_str.len() + 20);

    if file_in_path.components().any(|component| component.as_os_str() == "views") {
        html_replacement_str.push_str("<link rel='stylesheet' href='/assets/main.css'>");
    }

    html_replacement_str.push_str(&html_file_str);

    css_replacement_str.push_str("<style>");
    css_replacement_str.push_str(&css_file_str);
    css_replacement_str.push_str("</style>");

    // Insert import statements at the beginning, then the modified js content
    updated_js_file_str.push_str(&import_statements);
    updated_js_file_str.push_str(&js_file_str.replace("{--css--}", &css_replacement_str).replace("{--html--}", &html_replacement_str));

    Ok(updated_js_file_str)
}




fn process_js_parts_import_statements(file_in_path: &Path) -> String {

    let Some(current_dir) = file_in_path.parent() else {
        return String::new();
    };
    
    let parts_dir = current_dir.join("parts");
    
    if !parts_dir.exists() || !parts_dir.is_dir() {
        return String::new();
    }
    
    let Ok(entries) = fs::read_dir(&parts_dir) else {
        return String::new();
    };
    
    let mut import_statements = String::new();
    
    for entry in entries {
        let Ok(entry) = entry else { continue; };
        
        let entry_path = entry.path();
        if !entry_path.is_dir() {
            continue;
        }
        
        let Some(dir_name) = entry_path.file_name() else { continue; };
        let Some(dir_name_str) = dir_name.to_str() else { continue; };
        
        let js_file_name = format!("{}.js", dir_name_str);
        let js_file_path = entry_path.join(&js_file_name);
        
        if !js_file_path.exists() {
            continue;
        }
        
        let formated_string = format!("import './parts/{}/{}';      \n", dir_name_str, js_file_name);
        let formated_string_literal = formated_string.as_str();
        println!(&formated_string_literal);
        import_statements.push_str(&formated_string_literal);
    }
    
    import_statements
}




fn process_minify_js(js_str: &String) -> Result<String> {

    let minified_js = minify(js_str);
    let minified_js_string = minified_js.to_string();
    Ok(minified_js_string)
}




fn write_js(file_in_path: &Path, js_str: &String) -> Result<()> {

    let prefix_to_cut_str  = path(PathE::ClientOutputDev);
    let tmpdir_str         = pathp(PathE::TMPDir, "files/");
    let prefix_to_cut_path = Path::new(&prefix_to_cut_str);
    let output_dir_str     = Path::new(&tmpdir_str);

    let stripped_path      = file_in_path.strip_prefix(prefix_to_cut_path)?;
    let new_path           = output_dir_str.join(stripped_path);

    fs::create_dir_all(new_path.parent().unwrap())?;
    fs::write(new_path, js_str)?;

    Ok(())
}




fn is_skip_entry( entry: &walkdir::DirEntry) -> bool {

    if entry.file_name() == "media" {
        return false;
    }

    return true;
}




