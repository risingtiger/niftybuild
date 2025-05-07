
use anyhow::Result;

use std::io::Write;
use std::fs::{self};
use std::path::Path;
use brotli::CompressorWriter;
use glob::glob;

use crate::common_helperfuncs::path;
use crate::common_helperfuncs::PathE;



pub fn runit() -> Result<()> {

    let folder  = path(PathE::ClientOutputDist);

    let glob_str = &format!("{}/**/*", folder.to_str().unwrap());

    for entry in glob(glob_str)? {
        let path = entry.unwrap();

        if path.is_file() {

            match path.extension().and_then(|ext| ext.to_str()) {
                Some("js") | Some("css") => {
                    let ext = path.extension().unwrap().to_str().unwrap();
                    let metadata = fs::metadata(&path)?;
                    let size = metadata.len();

                    if size > 4000 {
                        let input = fs::read_to_string(&path)?;
                        process_brotli(&path, &input, ext)?;
                    }
                },
                _ => {}
            }
        }
    }

    Ok(())
}




fn process_brotli(file_out_path:&Path, input: &String, extension:&str) -> Result<()> {

    let input:&[u8] = input.as_bytes();

    let mut writer = CompressorWriter::new(
        Vec::new(),
        4096, /* buffer size */
        11,   /* quality */
        22    /* lgwin */
    );
    writer.write_all(input).unwrap();
    let x = writer.into_inner();
    let postfix = extension.to_string() + ".br";
    fs::write(file_out_path.with_extension(postfix), &x).unwrap();

    Ok(())
}


