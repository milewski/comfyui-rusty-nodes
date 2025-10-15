use crate::utilities::image_to_tensor;
use base64::Engine;
use comfy_builder_core::prelude::*;
use pyo3::exceptions::PyValueError;
use std::error::Error;

#[derive(NodeInput)]
pub struct Input {
    #[multiline = true]
    #[placeholder = "data:image/png;base64,..."]
    #[tooltip = "Base64‑encoded image data."]
    image: String,
}

#[derive(NodeOutput)]
pub struct Output {
    #[tooltip = "Base64‑encoded image data."]
    image: Image<f32>,
}

#[node(
    category = "Rusty Nodes / Image",
    description = "Convert a Base64‑encoded string into an image."
)]
pub struct Base64ToImage;

impl<'a> Node<'a> for Base64ToImage {
    type In = Input;
    type Out = Output;
    type Error = Box<dyn Error + Send + Sync>;

    fn execute(&self, input: Self::In) -> Result<Self::Out, Self::Error> {
        let data = strip_data_url_prefix(&input.image)
            .map_err(|error| PyValueError::new_err(format!("Invalid data provided: {}", error)))?;

        let image_bytes = base64::engine::general_purpose::STANDARD
            .decode(data)
            .map_err(|error| {
                PyValueError::new_err(format!(
                    "Could not decode the base64 string, are you sure it is valid? \nError: {error}"
                ))
            })?;

        Ok(Output {
            image: image_to_tensor(image::load_from_memory(&image_bytes)?)?,
        })
    }
}

/// Helper that strips the typical `data:<mime>;base64,` prefix.
/// Returns an error if the input is not valid Base‑64 after the prefix.
fn strip_data_url_prefix(data: &str) -> Result<&str, Box<dyn Error>> {
    if let Some(comma) = data.find(',') {
        Ok(&data[comma + 1..])
    } else {
        Ok(data)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use comfy_builder_core::run_node;

    #[test]
    fn base64_to_image_with_header() {
        let output = run_node!(
            Base64ToImage,
            Input {
                image:
                    "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAIAAACQd1PeAAAAD0lEQVR4AQEEAPv/AP8AAAMBAQCNHeWCAAAAAElFTkSuQmCC"
                        .to_string()
            }
        );

        assert_eq!(output.image.dims(), [1, 1, 1, 3])
    }

    #[test]
    fn base64_to_image_without_header() {
        let output = run_node!(
            Base64ToImage,
            Input {
                image:
                    "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAIAAACQd1PeAAAAD0lEQVR4AQEEAPv/AP8AAAMBAQCNHeWCAAAAAElFTkSuQmCC"
                        .to_string()
            }
        );

        assert_eq!(output.image.dims(), [1, 1, 1, 3])
    }

    #[test]
    fn base64_to_image_without_header_error() {
        let output = run_node!(
            Base64ToImage,
            Input {
                image: "invalid".to_string()
            },
            return
        );

        assert!(output.is_err());
    }
}
