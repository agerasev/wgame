//! Explicit GPU-to-CPU snapshots; no persistent CPU mirror or upload path.
use futures::channel::oneshot;
use half::f16;
use rgb::Rgba;
use wgame_gfx::{Offscreen, Target};
use wgame_image::Image;

/// GPU snapshot allocation, transfer, or mapping failure.
#[derive(Debug, thiserror::Error)]
pub enum ReadbackError {
    #[error("snapshot exceeds the device's buffer size limit")]
    TooLarge,
    #[error("GPU readback failed: {0}")]
    Poll(#[from] wgpu::PollError),
    #[error("GPU buffer mapping failed: {0}")]
    Map(#[from] wgpu::BufferAsyncError),
    #[error("mapped GPU buffer is unavailable: {0}")]
    Range(#[from] wgpu::MapRangeError),
    #[error("GPU snapshot completion was cancelled")]
    Cancelled,
    #[error("could not drive GPU readback: {0}")]
    Driver(String),
}

pub(crate) async fn readback(target: &mut Offscreen) -> Result<Image<Rgba<f16>>, ReadbackError> {
    let (width, height) = target.size();
    let format = target.state().format();
    let bytes_per_pixel = if format == wgpu::TextureFormat::Rgba16Float {
        8
    } else {
        4
    };
    let row = width
        .checked_mul(bytes_per_pixel)
        .ok_or(ReadbackError::TooLarge)?;
    let stride = row.checked_add(255).ok_or(ReadbackError::TooLarge)? / 256 * 256;
    let length = u64::from(stride) * u64::from(height);
    if length > target.state().device().limits().max_buffer_size || usize::try_from(length).is_err()
    {
        return Err(ReadbackError::TooLarge);
    }
    let device = target.state().device().clone();
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("render texture snapshot"),
        size: length,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let texture = target.texture().clone();
    target.encoder().copy_texture_to_buffer(
        texture.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(stride),
                rows_per_image: None,
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
    let submission = target.submit();
    let (send, receive) = oneshot::channel();
    buffer
        .slice(..)
        .map_async(wgpu::MapMode::Read, move |result| {
            let _ = send.send(result);
        });
    // Cancelling the future releases its buffer, even if a mapping callback is pending.
    struct Unmap(wgpu::Buffer);
    impl Drop for Unmap {
        fn drop(&mut self) {
            self.0.unmap();
        }
    }
    let _unmap = Unmap(buffer.clone());
    wait(&device, submission, receive).await?;
    let mapped = buffer.slice(..).get_mapped_range()?;
    let pixels = mapped
        .chunks(stride as usize)
        .flat_map(|row_data| row_data[..row as usize].chunks(bytes_per_pixel as usize))
        .map(|pixel| straight_alpha(decode(pixel, format), format.is_srgb()))
        .collect::<Vec<_>>();
    Ok(Image::with_data((width, height), pixels))
}

fn decode(bytes: &[u8], format: wgpu::TextureFormat) -> Rgba<f16> {
    use wgpu::TextureFormat::*;
    if format == Rgba16Float {
        let channel = |i| f16::from_bits(u16::from_le_bytes([bytes[i], bytes[i + 1]]));
        Rgba::new(channel(0), channel(2), channel(4), channel(6))
    } else {
        let channel = |i| f16::from_f32(bytes[i] as f32 / 255.0);
        if matches!(format, Bgra8Unorm | Bgra8UnormSrgb) {
            Rgba::new(channel(2), channel(1), channel(0), channel(3))
        } else {
            Rgba::new(channel(0), channel(1), channel(2), channel(3))
        }
    }
}

fn straight_alpha(pixel: Rgba<f16>, srgb: bool) -> Rgba<f16> {
    let alpha = pixel.a.to_f32();
    if alpha == 0.0 {
        return Rgba::new(f16::ZERO, f16::ZERO, f16::ZERO, pixel.a);
    }
    if alpha == 1.0 && !srgb {
        return pixel;
    }
    let channel = |c: f16| {
        let mut value = c.to_f32();
        if srgb {
            value = if value <= 0.04045 {
                value / 12.92
            } else {
                ((value + 0.055) / 1.055).powf(2.4)
            };
        }
        value /= alpha;
        f16::from_f32(value)
    };
    Rgba::new(
        channel(pixel.r),
        channel(pixel.g),
        channel(pixel.b),
        pixel.a,
    )
}

#[cfg(not(target_arch = "wasm32"))]
async fn wait(
    device: &wgpu::Device,
    submission: wgpu::SubmissionIndex,
    mapped: oneshot::Receiver<Result<(), wgpu::BufferAsyncError>>,
) -> Result<(), ReadbackError> {
    let device = device.clone();
    let (send, receive) = oneshot::channel();
    std::thread::Builder::new()
        .name("wgame-readback".into())
        .spawn(move || {
            let result = device.poll(wgpu::PollType::Wait {
                submission_index: Some(submission),
                timeout: Some(std::time::Duration::from_secs(30)),
            });
            let _ = send.send(result);
        })
        .map_err(|error| ReadbackError::Driver(error.to_string()))?;
    receive.await.map_err(|_| ReadbackError::Cancelled)??;
    mapped.await.map_err(|_| ReadbackError::Cancelled)??;
    Ok(())
}

#[cfg(target_arch = "wasm32")]
async fn wait(
    device: &wgpu::Device,
    _submission: wgpu::SubmissionIndex,
    mut mapped: oneshot::Receiver<Result<(), wgpu::BufferAsyncError>>,
) -> Result<(), ReadbackError> {
    let start = web_time::Instant::now();
    loop {
        device.poll(wgpu::PollType::Poll)?;
        if let Some(result) = mapped.try_recv().map_err(|_| ReadbackError::Cancelled)? {
            result?;
            return Ok(());
        }
        if start.elapsed().as_secs() >= 30 {
            return Err(ReadbackError::Driver("snapshot timed out".into()));
        }
        let window = web_sys::window()
            .ok_or_else(|| ReadbackError::Driver("browser window required".into()))?;
        // Yield to the browser rather than spinning microtasks while GPU work completes.
        let delay = js_sys::Promise::new(&mut |resolve, reject| {
            if let Err(error) =
                window.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 4)
            {
                let _ = reject.call1(&js_sys::JsString::from("readback timer"), &error);
            }
        });
        wasm_bindgen_futures::JsFuture::from(delay)
            .await
            .map_err(|error| ReadbackError::Driver(format!("{error:?}")))?;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn snapshot_channels_preserve_bgra_order_and_float_values() {
        assert_eq!(
            decode(&[0, 128, 255, 255], wgpu::TextureFormat::Bgra8Unorm),
            Rgba::new(f16::ONE, f16::from_f32(128.0 / 255.0), f16::ZERO, f16::ONE)
        );
        let input = [f16::from_f32(-0.5), f16::from_f32(2.0), f16::ZERO, f16::ONE];
        let bytes: Vec<_> = input
            .iter()
            .flat_map(|v| v.to_bits().to_le_bytes())
            .collect();
        assert_eq!(
            decode(&bytes, wgpu::TextureFormat::Rgba16Float),
            Rgba::new(input[0], input[1], input[2], input[3])
        );
    }
}
