use comfy_builder_core::candle::{Device, Tensor, WithDType};
use comfy_builder_core::numpy::Element;
use comfy_builder_core::prelude::Image;
use image::{DynamicImage, ImageBuffer};
use pyo3::exceptions::PyValueError;
use std::error::Error;

pub fn tensor_to_image(
    tensor: Tensor,
    width: u32,
    height: u32,
    channels: u32,
) -> Result<DynamicImage, Box<dyn Error + Send + Sync>> {
    let buffer: Vec<f32> = tensor.flatten_all()?.to_vec1()?;

    if !(channels == 3 || channels == 4) {
        Err(PyValueError::new_err("Only RGB/RGBA supported"))?
    }

    let pixels: Vec<u8> = buffer
        .into_iter()
        .map(|value| (value.clamp(0.0, 1.0) * 255.0).round() as u8)
        .collect();

    match channels {
        3 => Ok(DynamicImage::ImageRgb8(
            ImageBuffer::from_raw(width, height, pixels).ok_or_else(|| PyValueError::new_err("Invalid Dimensions"))?,
        )),
        4 => Ok(DynamicImage::ImageRgba8(
            ImageBuffer::from_raw(width, height, pixels).ok_or_else(|| PyValueError::new_err("Invalid Dimensions"))?,
        )),
        _ => unreachable!(),
    }
}

pub fn image_to_tensor<T: WithDType + Element>(image: DynamicImage) -> Result<Image<T>, Box<dyn Error + Send + Sync>> {
    let width = image.width() as usize;
    let height = image.height() as usize;
    let channels = image.color().channel_count() as usize;
    let pixels: Vec<f32> = match channels {
        3 => image.to_rgb32f().to_vec(),
        4 => image.to_rgba32f().to_vec(),
        _ => Err(PyValueError::new_err(format!(
            "Unexpected number of channels, expected 3 or 4 but received {}",
            channels
        )))?,
    };

    let tensor = Tensor::from_vec(pixels, (1, height, width, channels), &Device::Cpu)?;

    Ok(Image::from_tensor(tensor))
}
