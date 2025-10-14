use comfy_builder_core::candle;
use comfy_builder_core::candle::shape::ShapeWithOneHole;
use comfy_builder_core::candle::{Device, IndexOp};
use comfy_builder_core::prelude::*;
use rayon::prelude::*;
use resize::Pixel::{GrayF32, RGBF32};
use resize::{PixelFormat, Type};
use std::error::Error;
use std::ops::Deref;

#[derive(Debug, Default, Clone, Enum)]
enum Interpolation {
    #[default]
    #[label = "lanczos3"]
    Lanczos3,

    #[label = "point"]
    Point,

    #[label = "triangle"]
    Triangle,

    #[label = "catrom"]
    Catrom,

    #[label = "mitchell"]
    Mitchell,

    #[label = "bspline"]
    BSpline,

    #[label = "gaussian"]
    Gaussian,
}

impl From<Interpolation> for Type {
    fn from(value: Interpolation) -> Self {
        match value {
            Interpolation::Lanczos3 => Type::Lanczos3,
            Interpolation::Point => Type::Point,
            Interpolation::Triangle => Type::Triangle,
            Interpolation::Catrom => Type::Catrom,
            Interpolation::Mitchell => Type::Mitchell,
            Interpolation::BSpline => Type::BSpline,
            Interpolation::Gaussian => Type::Gaussian,
        }
    }
}

#[derive(Debug, NodeInput)]
pub struct Input {
    width: usize,
    height: usize,
    image: Image<f32>,
    mask: Option<Mask<f32>>,
    interpolation: Interpolation,
}

#[derive(NodeOutput)]
pub struct Output {
    image: Image<f32>,
    mask: Option<Mask<f32>>,
    width: usize,
    height: usize,
}

#[node(category = "Rusty Nodes / Image")]
pub struct ResizeImage;

impl<'a> Node<'a> for ResizeImage {
    type In = Input;
    type Out = Output;
    type Error = Box<dyn Error + Send + Sync>;

    fn execute(&self, input: Self::In) -> Result<Self::Out, Self::Error> {
        let mask = if let Some(mask) = input.mask {
            let (batch, mask_height, mask_width) = mask.dims3()?;

            let mask = self.resize_parallel::<1, Mask<f32>, _, _, _>(
                &mask,
                batch,
                mask_width,
                mask_height,
                input.width,
                input.height,
                input.interpolation.clone(),
                || GrayF32,
                |batch, width, height, _| (batch, height, width),
            )?;

            Some(mask)
        } else {
            None
        };

        let (batch, height, width, _) = input.image.dims4()?;

        let image = self.resize_parallel::<3, Image<f32>, _, _, _>(
            &input.image,
            batch,
            width,
            height,
            input.width,
            input.height,
            input.interpolation.clone(),
            || RGBF32,
            |batch, width, height, channels| (batch, height, width, channels),
        )?;

        Ok(Output {
            image,
            mask,
            width: input.width,
            height: input.height,
        })
    }
}

impl ResizeImage {
    #[allow(clippy::too_many_arguments)]
    fn resize_parallel<'a, const CHANNELS: usize, Output, Input, Format, Shape>(
        &self,
        image: &Input,
        batch: usize,
        width: usize,
        height: usize,
        target_width: usize,
        target_height: usize,
        interpolation: Interpolation,
        format: fn() -> Format,
        get_shape: fn(batch: usize, width: usize, height: usize, channels: usize) -> Shape,
    ) -> Result<Output, candle::Error>
    where
        Input: IndexOp<usize> + Send + Sync,
        Output: Deref<Target = Input>,
        Output: TryFrom<(Vec<f32>, Shape, &'a Device), Error = candle::Error>,
        Format: PixelFormat,
        Shape: ShapeWithOneHole,
    {
        let output_pixels_per_image = target_width * target_height * CHANNELS;

        let resized_data: Vec<Vec<f32>> = (0..batch)
            .into_par_iter()
            .flat_map(|batch| image.i(batch))
            .flat_map(|tensor| tensor.flatten_all().and_then(|data| data.to_vec1()))
            .flat_map(|data| {
                self.resize::<CHANNELS, Format>(
                    &data,
                    width,
                    height,
                    target_width,
                    target_height,
                    interpolation.clone().into(),
                    format(),
                )
            })
            .collect();

        let mut data = Vec::with_capacity(batch * output_pixels_per_image);

        for chunk in resized_data {
            data.extend_from_slice(&chunk);
        }

        Output::try_from((
            data,
            get_shape(batch, target_height, target_width, CHANNELS),
            &Device::Cpu,
        ))
    }

    #[allow(clippy::too_many_arguments)]
    fn resize<const CHANNELS: usize, Format: PixelFormat>(
        &self,
        input: &[f32],
        origin_width: usize,
        origin_height: usize,
        target_width: usize,
        target_height: usize,
        r#type: Type,
        format: Format,
    ) -> resize::Result<Vec<f32>> {
        let output_pixels_per_image = target_width * target_height * CHANNELS;
        let mut output = vec![0.0f32; output_pixels_per_image];

        let mut resizer = resize::new(
            origin_width,
            origin_height,
            target_width,
            target_height,
            format,
            r#type,
        )?;

        let input_rgb: &[Format::InputPixel] = unsafe {
            std::slice::from_raw_parts(
                input.as_ptr() as *const Format::InputPixel,
                input.len() / CHANNELS,
            )
        };

        let output_rgb: &mut [Format::OutputPixel] = unsafe {
            std::slice::from_raw_parts_mut(
                output.as_mut_ptr() as *mut Format::OutputPixel,
                output.len() / CHANNELS,
            )
        };

        resizer.resize(input_rgb, output_rgb)?;

        Ok(output)
    }
}
