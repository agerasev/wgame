#![forbid(unsafe_code)]

pub type Path = str;
pub type PathBuf = String;

#[cfg(feature = "std")]
pub async fn read_bytes(path: impl AsRef<Path>) -> anyhow::Result<Vec<u8>> {
    Ok(async_fs::read(path.as_ref()).await?)
}

#[cfg(feature = "std")]
pub async fn read_string(path: impl AsRef<Path>) -> anyhow::Result<String> {
    Ok(async_fs::read_to_string(path.as_ref()).await?)
}

#[cfg(feature = "web")]
mod web {
    use anyhow::{Context, Error, Result, anyhow, bail};
    use js_sys::{ArrayBuffer, Uint8Array};
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{Request, RequestInit, RequestMode, Response};

    fn value_to_error(value: JsValue) -> Error {
        anyhow!("{value:?}")
    }

    async fn request(path: &str) -> Result<Response> {
        let opts = RequestInit::new();
        opts.set_method("GET");
        opts.set_mode(RequestMode::Cors);

        let request = Request::new_with_str_and_init(path, &opts).map_err(value_to_error)?;

        /*
        request
            .headers()
            .set("Accept", "application/json")?;
        */

        let window = web_sys::window().unwrap();
        let resp_value = JsFuture::from(window.fetch_with_request(&request))
            .await
            .map_err(value_to_error)?;

        // `resp_value` is a `Response` object.
        assert!(resp_value.is_instance_of::<Response>());
        let resp: Response = resp_value.dyn_into().unwrap();

        let status = resp.status();
        log::info!("Status: {status}");
        if status != 200 {
            bail!("Status {}: {}", status, resp.status_text())
        }

        Ok(resp)
    }

    async fn request_bytes(path: &str) -> Result<Vec<u8>> {
        let resp = request(path).await?;

        // Convert this other `Promise` into a rust `Future`.
        let buffer_value = JsFuture::from(resp.array_buffer().map_err(value_to_error)?)
            .await
            .map_err(value_to_error)?;

        assert!(buffer_value.is_instance_of::<ArrayBuffer>());
        let buffer: ArrayBuffer = buffer_value.dyn_into().unwrap();

        let byte_array =
            Uint8Array::new_with_byte_offset_and_length(&buffer, 0, buffer.byte_length());

        Ok(byte_array.to_vec())
    }

    async fn request_string(path: &str) -> Result<String> {
        let resp = request(path).await?;

        // Convert this other `Promise` into a rust `Future`.
        let string_value = JsFuture::from(resp.text().map_err(value_to_error)?)
            .await
            .map_err(value_to_error)?;

        let string = string_value.as_string().unwrap();

        Ok(string)
    }

    pub async fn read_bytes(path: impl AsRef<Path>) -> Result<Vec<u8>> {
        request_bytes(path.as_ref())
            .await
            .context("Request bytes error")
    }

    pub async fn read_string(path: impl AsRef<Path>) -> Result<String> {
        request_string(path.as_ref())
            .await
            .context("Request string error")
    }
}

#[cfg(feature = "web")]
pub use web::{read_bytes, read_string};
