use comfy_builder_core::prelude::*;
use serde_json::Value;
use std::error::Error;

#[derive(NodeInput)]
pub struct Input {
    #[multiline = true]
    #[placeholder = "{ \"key\": \"value\" },..."]
    #[tooltip = "Any JSON string, in any format."]
    json: String,
}

#[derive(NodeOutput)]
pub struct Output {
    #[tooltip = "The output value."]
    string: Vec<String>,
}

#[node(
    category = "Rusty Nodes / Json",
    description = "Parse the input as a JSON string and return the parsed value; if it’s an array, return the list."
)]
pub struct JsonLoader;

impl Node for JsonLoader {
    type In = Input;
    type Out = Output;
    type Error = Box<dyn Error + Send + Sync>;

    fn execute(&self, input: Self::In) -> Result<Self::Out, Self::Error> {
        let value = serde_json::from_str::<Value>(&input.json)?;
        let value: Vec<Value> = value
            .as_array()
            .map(|array| array.to_vec())
            .unwrap_or_else(|| vec![value]);

        Ok(Output {
            string: value.into_iter().map(|value| value.to_string()).collect(),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
}
