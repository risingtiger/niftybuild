
use anyhow::Result;
use std::fs;
use std::path::Path;
use minifier::js::minify;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::{SourceType, GetSpan};
use oxc_ast::ast::{ClassElement, PropertyKey, Statement};

use crate::common_helperfuncs::{PathE, pathp};



pub fn runit() -> Result<()> {

    let views_dirs = vec![
        pathp(PathE::TMPDirFiles, "lazy/views/"),
        pathp(PathE::InstanceClientOutputTMP, "lazy/views/"),
    ];

    let mut extracted_loads: Vec<(String, String)> = Vec::new();

    for views_dir in views_dirs {

        if !views_dir.exists() { 
            continue; 
        }

        for entry in fs::read_dir(&views_dir)? {

            let entry = entry?;
            let view_folder = entry.path();

            if !view_folder.is_dir() { 
                continue; 
            }

            let folder_name = view_folder.file_name().unwrap().to_str().unwrap();
            let js_file = view_folder.join(format!("{}.js", folder_name));

            if !js_file.exists() { 
                continue; 
            }

            if let Some((func_name, func_body)) = extract_static_load(&js_file, folder_name)? {
                extracted_loads.push((func_name, func_body));
            }
        }
    }

    if extracted_loads.is_empty() {
        println!("No static load functions found to extract");
        return Ok(());
    }

    let mut output = String::new();
    let mut export_names: Vec<String> = Vec::new();

    for (name, body) in &extracted_loads {
        output.push_str(&format!("const {}={};\n", name, body));
        export_names.push(name.clone());
    }

    output.push_str(&format!("export default{{{}}};", export_names.join(",")));

    let minified = minify(&output).to_string();

    let output_path = pathp(PathE::ServerOutput, "viewloads.js");
    fs::write(&output_path, &minified)?;

    println!("Generated viewloads.js with {} load functions", extracted_loads.len());

    Ok(())
}




fn extract_static_load(js_file: &Path, folder_name: &str) -> Result<Option<(String, String)>> {

    let source = fs::read_to_string(js_file)?;

    let allocator = Allocator::default();
    let source_type = SourceType::from_path(js_file).unwrap_or_default();
    let parser_return = Parser::new(&allocator, &source, source_type).parse();

    if parser_return.errors.len() > 0 {
        eprintln!("Parse errors in {}: {:?}", js_file.display(), parser_return.errors);
    }

    for stmt in parser_return.program.body.iter() {

        if let Statement::ClassDeclaration(class_decl) = stmt {

            for element in class_decl.body.body.iter() {

                if let ClassElement::PropertyDefinition(prop_def) = element {

                    if !prop_def.r#static {
                        continue;
                    }

                    let is_load = match &prop_def.key {
                        PropertyKey::StaticIdentifier(ident) => ident.name == "load",
                        _ => false,
                    };

                    if !is_load {
                        continue;
                    }

                    if let Some(value) = &prop_def.value {
                        let start = value.span().start as usize;
                        let end = value.span().end as usize;
                        let func_body = &source[start..end];

                        let func_name = format!("load_{}", folder_name.to_lowercase());

                        return Ok(Some((func_name, func_body.to_string())));
                    }
                }
            }
        }
    }

    Ok(None)
}


