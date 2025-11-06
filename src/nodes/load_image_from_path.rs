use crate::utilities::image_to_tensor;
use comfy_builder_core::prelude::*;
use std::env::current_dir;
use std::error::Error;
use std::fs;

#[derive(NodeInput)]
pub struct Input {
    #[tooltip = "Path to the image file relative to current working directory."]
    #[placeholder = "ComfyUI/input/my-image.jpg"]
    filename: String,
}

#[derive(NodeOutput)]
pub struct Output {
    #[tooltip = "The loaded image."]
    image: Image<f32>,
}

#[node(
    category = "Rusty Nodes / Image",
    description = "Load an image from a given file path relative to current working directory."
)]
pub struct LoadImageFromPath;

impl Node for LoadImageFromPath {
    type In = Input;
    type Out = Output;
    type Error = Box<dyn Error + Send + Sync>;

    fn execute(&self, input: Self::In) -> Result<Self::Out, Self::Error> {
        let path = current_dir()?.join(input.filename);
        let bytes = fs::read(&path).map_err(|error| format!("{} ({:?})", error, path))?;

        Ok(Output {
            image: image_to_tensor(image::load_from_memory(bytes.as_slice())?)?,
        })
    }
}
