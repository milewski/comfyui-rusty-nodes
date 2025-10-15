use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use comfy_builder_core::candle::{IndexOp, Tensor};
use comfy_builder_core::prelude::*;
use image::{DynamicImage, ImageBuffer, ImageFormat};
use pyo3::exceptions::PyValueError;
use std::error::Error;
use std::io::Cursor;

#[derive(Debug, Enum)]
enum Format {
    #[label = "webp"]
    WebP,

    #[label = "png"]
    Png,

    #[label = "jpeg"]
    Jpeg,

    #[label = "gif"]
    Gif,
}

impl From<Format> for ImageFormat {
    fn from(value: Format) -> Self {
        match value {
            Format::Jpeg => ImageFormat::Jpeg,
            Format::Gif => ImageFormat::Gif,
            Format::Png => ImageFormat::Png,
            Format::WebP => ImageFormat::WebP,
        }
    }
}

#[derive(NodeInput)]
pub struct Input {
    #[tooltip = "The image to be encoded, it can contain alpha channel."]
    image: Image<f32>,

    #[default = true]
    #[tooltip = "if true the output will include the 'data:;base64;' header instructions"]
    include_header: bool,

    #[default = "webp"]
    #[tooltip = "the desired formate to be encoded as base64"]
    format: Format,
}

#[derive(NodeOutput)]
pub struct Output {
    base64: String,
}

#[node(display_name = "Image To Base64", category = "Rusty Nodes / Image")]
pub struct ImageToBase64;

impl<'a> Node<'a> for ImageToBase64 {
    type In = Input;
    type Out = Output;
    type Error = Box<dyn Error + Send + Sync>;

    fn execute(&self, input: Self::In) -> Result<Self::Out, Self::Error> {
        let shape = input.image.dims();
        let height = shape[1] as u32;
        let width = shape[2] as u32;
        let channels = shape[3] as u32;

        let image = tensor_to_image(input.image.i(0)?, width, height, channels)?;

        let format: ImageFormat = input.format.into();
        let bytes = {
            let mut buffer = Cursor::new(Vec::new());
            image.write_to(&mut buffer, format)?;
            buffer.into_inner()
        };

        let encoded = STANDARD.encode(&bytes);
        let header = if input.include_header {
            format!("data:{};base64,{}", format.to_mime_type(), encoded)
        } else {
            format!("{}", encoded)
        };

        Ok(Output { base64: header })
    }
}

fn tensor_to_image(
    tensor: Tensor,
    width: u32,
    height: u32,
    channels: u32,
) -> Result<DynamicImage, Box<dyn Error + Send + Sync>> {
    let buffer: Vec<f32> = tensor.flatten_all()?.to_vec1()?;

    if (channels == 3 || channels == 4) == false {
        Err(PyValueError::new_err("Only RGB/RGBA supported"))?
    }

    let pixels: Vec<u8> = buffer
        .into_iter()
        .map(|value| (value.clamp(0.0, 1.0) * 255.0).round() as u8)
        .collect();

    match channels {
        3 => Ok(DynamicImage::ImageRgb8(
            ImageBuffer::from_raw(width, height, pixels)
                .ok_or_else(|| PyValueError::new_err("Invalid Dimensions"))?,
        )),
        4 => Ok(DynamicImage::ImageRgba8(
            ImageBuffer::from_raw(width, height, pixels)
                .ok_or_else(|| PyValueError::new_err("Invalid Dimensions"))?,
        )),
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::nodes::base64_image::image_to_tensor;
    use comfy_builder_core::run_node;

    macro_rules! generate_node_tests {
        (
            $fn_name:ident,
            node => $node:ident,
            image_path => $image_path:expr,
            cases => [
                $( { include_header: $include_header:expr, format: $format:expr, base64: $expected_base64:expr } ),* $(,)?
            ]
        ) => {
            #[test]
            fn $fn_name() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
                let image = image_to_tensor::<f32>(
                    image::ImageReader::open($image_path)?.decode()?
                )?;

                $(
                    let input = Input {
                        image: image.clone(),
                        include_header: $include_header,
                        format: $format,
                    };

                    let output = run_node!($node, input);

                    assert_eq!(
                        output.base64,
                        $expected_base64,
                        "Mismatch for include_header={} format={:?}",
                        $include_header,
                        $format
                    );
                )*

                Ok(())
            }
        };
    }

    generate_node_tests!(
        test_png,
        node => ImageToBase64,
        image_path => "./src/fixtures/2x2.png",
        cases => [
            { include_header: true,  format: Format::Png,  base64: "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAIAAACQd1PeAAAAD0lEQVR4AQEEAPv/AP8AAAMBAQCNHeWCAAAAAElFTkSuQmCC" },
            { include_header: false, format: Format::Png,  base64: "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAIAAACQd1PeAAAAD0lEQVR4AQEEAPv/AP8AAAMBAQCNHeWCAAAAAElFTkSuQmCC" },
        ]
    );

    generate_node_tests!(
        test_webp,
        node => ImageToBase64,
        image_path => "./src/fixtures/2x2.png",
        cases => [
            { include_header: true,  format: Format::WebP,  base64: "data:image/webp;base64,UklGRhoAAABXRUJQVlA4TA4AAAAvAAAAAM1VICIC0f+IBA==" },
            { include_header: false, format: Format::WebP,  base64: "UklGRhoAAABXRUJQVlA4TA4AAAAvAAAAAM1VICIC0f+IBA==" },
        ]
    );

    generate_node_tests!(
        test_jpeg,
        node => ImageToBase64,
        image_path => "./src/fixtures/2x2.png",
        cases => [
            { include_header: true,  format: Format::Jpeg,  base64: "data:image/jpeg;base64,/9j/4AAQSkZJRgABAgAAAQABAAD/wAARCAABAAEDAREAAhEBAxEB/9sAQwAIBgYHBgUIBwcHCQkICgwUDQwLCwwZEhMPFB0aHx4dGhwcICQuJyAiLCMcHCg3KSwwMTQ0NB8nOT04MjwuMzQy/9sAQwEJCQkMCwwYDQ0YMiEcITIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIy/8QAHwAAAQUBAQEBAQEAAAAAAAAAAAECAwQFBgcICQoL/8QAtRAAAgEDAwIEAwUFBAQAAAF9AQIDAAQRBRIhMUEGE1FhByJxFDKBkaEII0KxwRVS0fAkM2JyggkKFhcYGRolJicoKSo0NTY3ODk6Q0RFRkdISUpTVFVWV1hZWmNkZWZnaGlqc3R1dnd4eXqDhIWGh4iJipKTlJWWl5iZmqKjpKWmp6ipqrKztLW2t7i5usLDxMXGx8jJytLT1NXW19jZ2uHi4+Tl5ufo6erx8vP09fb3+Pn6/8QAHwEAAwEBAQEBAQEBAQAAAAAAAAECAwQFBgcICQoL/8QAtREAAgECBAQDBAcFBAQAAQJ3AAECAxEEBSExBhJBUQdhcRMiMoEIFEKRobHBCSMzUvAVYnLRChYkNOEl8RcYGRomJygpKjU2Nzg5OkNERUZHSElKU1RVVldYWVpjZGVmZ2hpanN0dXZ3eHl6goOEhYaHiImKkpOUlZaXmJmaoqOkpaanqKmqsrO0tba3uLm6wsPExcbHyMnK0tPU1dbX2Nna4uPk5ebn6Onq8vP09fb3+Pn6/9oADAMBAAIRAxEAPwDi6+ZP3E//2Q==" },
            { include_header: false, format: Format::Jpeg,  base64: "/9j/4AAQSkZJRgABAgAAAQABAAD/wAARCAABAAEDAREAAhEBAxEB/9sAQwAIBgYHBgUIBwcHCQkICgwUDQwLCwwZEhMPFB0aHx4dGhwcICQuJyAiLCMcHCg3KSwwMTQ0NB8nOT04MjwuMzQy/9sAQwEJCQkMCwwYDQ0YMiEcITIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIy/8QAHwAAAQUBAQEBAQEAAAAAAAAAAAECAwQFBgcICQoL/8QAtRAAAgEDAwIEAwUFBAQAAAF9AQIDAAQRBRIhMUEGE1FhByJxFDKBkaEII0KxwRVS0fAkM2JyggkKFhcYGRolJicoKSo0NTY3ODk6Q0RFRkdISUpTVFVWV1hZWmNkZWZnaGlqc3R1dnd4eXqDhIWGh4iJipKTlJWWl5iZmqKjpKWmp6ipqrKztLW2t7i5usLDxMXGx8jJytLT1NXW19jZ2uHi4+Tl5ufo6erx8vP09fb3+Pn6/8QAHwEAAwEBAQEBAQEBAQAAAAAAAAECAwQFBgcICQoL/8QAtREAAgECBAQDBAcFBAQAAQJ3AAECAxEEBSExBhJBUQdhcRMiMoEIFEKRobHBCSMzUvAVYnLRChYkNOEl8RcYGRomJygpKjU2Nzg5OkNERUZHSElKU1RVVldYWVpjZGVmZ2hpanN0dXZ3eHl6goOEhYaHiImKkpOUlZaXmJmaoqOkpaanqKmqsrO0tba3uLm6wsPExcbHyMnK0tPU1dbX2Nna4uPk5ebn6Onq8vP09fb3+Pn6/9oADAMBAAIRAxEAPwDi6+ZP3E//2Q==" },
        ]
    );

    generate_node_tests!(
        test_gif,
        node => ImageToBase64,
        image_path => "./src/fixtures/2x2.png",
        cases => [
            { include_header: true,  format: Format::Gif,  base64: "data:image/gif;base64,R0lGODlhAQABAIAAAAAAAAAAACH5BAgAAAAALAAAAAABAAEAgP8AAAAAAAICRAEAOw==" },
            { include_header: false, format: Format::Gif,  base64: "R0lGODlhAQABAIAAAAAAAAAAACH5BAgAAAAALAAAAAABAAEAgP8AAAAAAAICRAEAOw==" },
        ]
    );
}
