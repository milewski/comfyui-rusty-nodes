use comfy_builder_core::prelude::*;
use std::error::Error;

#[derive(NodeInput)]
pub struct Input {
    #[tooltip = "A List of List of Images."]
    images: Vec<Vec<Image<f32>>>,
}

#[derive(NodeOutput)]
pub struct Output {
    #[tooltip = "The collapsed image output."]
    images: Vec<Image<f32>>,
}

#[node(
    category = "Rusty Nodes / Utility",
    description = "Collapse a [[Images]] into [Image]."
)]
pub struct CollapseImageList;

impl Node for CollapseImageList {
    type In = Input;
    type Out = Output;
    type Error = Box<dyn Error + Send + Sync>;

    fn execute(&self, input: Self::In) -> Result<Self::Out, Self::Error> {
        Ok(Output {
            images: input.images.into_iter().flatten().collect(),
        })
    }
}
