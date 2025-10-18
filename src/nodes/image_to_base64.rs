use crate::utilities::tensor_to_image;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use comfy_builder_core::candle::IndexOp;
use comfy_builder_core::prelude::*;
use image::ImageFormat;
use std::error::Error;
use std::io::Cursor;

#[derive(Debug, Enum)]
enum Format {
    #[display_name = "webp"]
    WebP,

    #[display_name = "png"]
    Png,

    #[display_name = "jpeg"]
    Jpeg,

    #[display_name = "gif"]
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
struct Input {
    #[tooltip = "Image to encode. Alpha channel is preserved if present."]
    image: Image<f32>,

    #[tooltip = "Add MIME header `data:image/<fmt>;base64` to the result. Set false to return raw Base64."]
    #[default = true]
    include_header: bool,

    #[tooltip = "Target image format for Base64 output."]
    #[default = "webp"]
    format: Format,
}

#[derive(NodeOutput)]
struct Output {
    #[tooltip = "Base64 string of the image."]
    base64: String,
}

#[node(
    category = "Rusty Nodes / Image",
    description = "Convert an image into a Base64‑encoded string."
)]
struct ImageToBase64;

impl Node for ImageToBase64 {
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
            buffer
        };

        let encoded = STANDARD.encode(bytes.into_inner());
        let header = if input.include_header {
            format!("data:{};base64,{}", format.to_mime_type(), encoded)
        } else {
            encoded.to_string()
        };

        Ok(Output { base64: header })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::utilities::image_to_tensor;
    use comfy_builder_core::run_node;

    macro_rules! generate_test {
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

    generate_test!(
        png,
        node => ImageToBase64,
        image_path => "./src/fixtures/1x1.png",
        cases => [
            { include_header: true,  format: Format::Png,  base64: "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAIAAACQd1PeAAAAD0lEQVR4AQEEAPv/AP8AAAMBAQCNHeWCAAAAAElFTkSuQmCC" },
            { include_header: false, format: Format::Png,  base64: "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAIAAACQd1PeAAAAD0lEQVR4AQEEAPv/AP8AAAMBAQCNHeWCAAAAAElFTkSuQmCC" },
        ]
    );

    generate_test!(
        webp,
        node => ImageToBase64,
        image_path => "./src/fixtures/1x1.png",
        cases => [
            { include_header: true,  format: Format::WebP,  base64: "data:image/webp;base64,UklGRhoAAABXRUJQVlA4TA4AAAAvAAAAAM1VICIC0f+IBA==" },
            { include_header: false, format: Format::WebP,  base64: "UklGRhoAAABXRUJQVlA4TA4AAAAvAAAAAM1VICIC0f+IBA==" },
        ]
    );

    generate_test!(
        jpeg,
        node => ImageToBase64,
        image_path => "./src/fixtures/1x1.png",
        cases => [
            { include_header: true,  format: Format::Jpeg,  base64: "data:image/jpeg;base64,/9j/4AAQSkZJRgABAgAAAQABAAD/wAARCAABAAEDAREAAhEBAxEB/9sAQwAIBgYHBgUIBwcHCQkICgwUDQwLCwwZEhMPFB0aHx4dGhwcICQuJyAiLCMcHCg3KSwwMTQ0NB8nOT04MjwuMzQy/9sAQwEJCQkMCwwYDQ0YMiEcITIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIy/8QAHwAAAQUBAQEBAQEAAAAAAAAAAAECAwQFBgcICQoL/8QAtRAAAgEDAwIEAwUFBAQAAAF9AQIDAAQRBRIhMUEGE1FhByJxFDKBkaEII0KxwRVS0fAkM2JyggkKFhcYGRolJicoKSo0NTY3ODk6Q0RFRkdISUpTVFVWV1hZWmNkZWZnaGlqc3R1dnd4eXqDhIWGh4iJipKTlJWWl5iZmqKjpKWmp6ipqrKztLW2t7i5usLDxMXGx8jJytLT1NXW19jZ2uHi4+Tl5ufo6erx8vP09fb3+Pn6/8QAHwEAAwEBAQEBAQEBAQAAAAAAAAECAwQFBgcICQoL/8QAtREAAgECBAQDBAcFBAQAAQJ3AAECAxEEBSExBhJBUQdhcRMiMoEIFEKRobHBCSMzUvAVYnLRChYkNOEl8RcYGRomJygpKjU2Nzg5OkNERUZHSElKU1RVVldYWVpjZGVmZ2hpanN0dXZ3eHl6goOEhYaHiImKkpOUlZaXmJmaoqOkpaanqKmqsrO0tba3uLm6wsPExcbHyMnK0tPU1dbX2Nna4uPk5ebn6Onq8vP09fb3+Pn6/9oADAMBAAIRAxEAPwDi6+ZP3E//2Q==" },
            { include_header: false, format: Format::Jpeg,  base64: "/9j/4AAQSkZJRgABAgAAAQABAAD/wAARCAABAAEDAREAAhEBAxEB/9sAQwAIBgYHBgUIBwcHCQkICgwUDQwLCwwZEhMPFB0aHx4dGhwcICQuJyAiLCMcHCg3KSwwMTQ0NB8nOT04MjwuMzQy/9sAQwEJCQkMCwwYDQ0YMiEcITIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIy/8QAHwAAAQUBAQEBAQEAAAAAAAAAAAECAwQFBgcICQoL/8QAtRAAAgEDAwIEAwUFBAQAAAF9AQIDAAQRBRIhMUEGE1FhByJxFDKBkaEII0KxwRVS0fAkM2JyggkKFhcYGRolJicoKSo0NTY3ODk6Q0RFRkdISUpTVFVWV1hZWmNkZWZnaGlqc3R1dnd4eXqDhIWGh4iJipKTlJWWl5iZmqKjpKWmp6ipqrKztLW2t7i5usLDxMXGx8jJytLT1NXW19jZ2uHi4+Tl5ufo6erx8vP09fb3+Pn6/8QAHwEAAwEBAQEBAQEBAQAAAAAAAAECAwQFBgcICQoL/8QAtREAAgECBAQDBAcFBAQAAQJ3AAECAxEEBSExBhJBUQdhcRMiMoEIFEKRobHBCSMzUvAVYnLRChYkNOEl8RcYGRomJygpKjU2Nzg5OkNERUZHSElKU1RVVldYWVpjZGVmZ2hpanN0dXZ3eHl6goOEhYaHiImKkpOUlZaXmJmaoqOkpaanqKmqsrO0tba3uLm6wsPExcbHyMnK0tPU1dbX2Nna4uPk5ebn6Onq8vP09fb3+Pn6/9oADAMBAAIRAxEAPwDi6+ZP3E//2Q==" },
        ]
    );

    generate_test!(
        gif,
        node => ImageToBase64,
        image_path => "./src/fixtures/1x1.png",
        cases => [
            { include_header: true,  format: Format::Gif,  base64: "data:image/gif;base64,R0lGODlhAQABAIAAAAAAAAAAACH5BAgAAAAALAAAAAABAAEAgP8AAAAAAAICRAEAOw==" },
            { include_header: false, format: Format::Gif,  base64: "R0lGODlhAQABAIAAAAAAAAAAACH5BAgAAAAALAAAAAABAAEAgP8AAAAAAAICRAEAOw==" },
        ]
    );
}
