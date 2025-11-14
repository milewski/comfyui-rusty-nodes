use comfy_builder_core::prelude::*;
use std::error::Error;
use std::fs;
use std::path::Path;

#[derive(NodeInput)]
struct Input {
    file: String,
    move_to: String,
}

#[derive(NodeOutput)]
pub struct Output {
    output: String,
}

#[node(category = "Rusty Nodes / Utility", description = "Move file.")]
struct MoveFile;

impl Node for MoveFile {
    type In = Input;
    type Out = Output;
    type Error = Box<dyn Error + Send + Sync>;

    fn execute(&self, input: Self::In) -> Result<Self::Out, Self::Error> {
        let source = Path::new(&input.file);
        let destination_directory = Path::new(&input.move_to);

        fs::create_dir_all(destination_directory)?;

        let file_name = source.file_name().ok_or("Source path has no filename")?;
        let destination_file = destination_directory.join(file_name);

        fs::rename(&source, &destination_file)?;

        Ok(Output {
            output: destination_file.to_string_lossy().to_string(),
        })
    }
}
