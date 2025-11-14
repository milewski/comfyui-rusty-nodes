use comfy_builder_core::prelude::*;
use std::error::Error;
use std::fs;

#[derive(NodeInput)]
struct Input {
    from: String,
    to: String,
}

#[derive(NodeOutput)]
pub struct Output {
    output: String,
}

#[node(category = "Rusty Nodes / Utility", description = "Rename a file.")]
struct RenameFile;

impl Node for RenameFile {
    type In = Input;
    type Out = Output;
    type Error = Box<dyn Error + Send + Sync>;

    fn execute(&self, input: Self::In) -> Result<Self::Out, Self::Error> {
        fs::rename(input.from, &input.to)?;

        Ok(Output { output: input.to })
    }
}
