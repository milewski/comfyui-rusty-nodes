use comfy_builder_core::prelude::*;
use serde_json::Value;
use std::error::Error;

#[derive(NodeInput)]
pub struct Input {
    #[tooltip = "JavaScript Object Notation (JSON) Pointer."]
    #[placeholder = "/path/0/key"]
    path: String,

    #[tooltip = "The JSON document from which to extract the value."]
    json: String,
}

#[derive(NodeOutput)]
pub struct Output {
    #[tooltip = "The extracted value."]
    string: String,
}

#[node(
    category = "Rusty Nodes / Json",
    description = "Extract a specific key from a JSON object using a JSON Pointer notation."
)]
pub struct JsonPathExtractor;

impl Node for JsonPathExtractor {
    type In = Input;
    type Out = Output;
    type Error = Box<dyn Error + Send + Sync>;

    fn execute(&self, input: Self::In) -> Result<Self::Out, Self::Error> {
        Ok(Output {
            string: serde_json::from_str::<Value>(&input.json)?
                .pointer(input.path.as_str())
                .and_then(|value| match value {
                    Value::Null => None,
                    Value::String(string) => Some(string.clone()),
                    _ => Some(value.to_string()),
                })
                .unwrap_or_default(),
        })
    }
}
