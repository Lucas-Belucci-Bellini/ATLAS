//! Browser HTTP/JSON client for ATLAS (WASM).
//!
//! Uses the browser Fetch API with a cancellable request timeout. The client
//! returns parsed JSON as JsValue and deliberately does not impose a
//! serialization framework.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use js_sys::Reflect;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{AbortController, Request, RequestInit, Response, Window};

/// Client for a same-origin or cross-origin HTTP API.
///
/// Cross-origin servers must explicitly allow the frontend origin with CORS.
/// This client does not add authentication credentials automatically.
#[derive(Debug, Clone)]
pub struct HttpClient {
    base_url: String,
    timeout_ms: u32,
}

impl HttpClient {
    /// Creates a client for an absolute HTTP(S) base URL.
    ///
    /// The default request timeout is 10 seconds.
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            timeout_ms: 10_000,
        }
    }

    /// Sets the per-request timeout in milliseconds.
    ///
    /// A zero timeout is rejected by get_json.
    pub fn with_timeout_ms(mut self, timeout_ms: u32) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    /// Performs a GET request and parses a successful response as JSON.
    ///
    /// Returns the status and request ID alongside the parsed JSON body.
    /// Network errors, non-2xx responses, invalid JSON and timeout expiration
    /// are returned as errors.
    pub async fn get_json(&self, path: &str) -> Result<HttpResponse, JsValue> {
        if self.timeout_ms == 0 {
            return Err(JsValue::from_str("timeout must be greater than zero"));
        }

        let base = self.base_url.trim();
        if !(base.starts_with("http://") || base.starts_with("https://")) {
            return Err(JsValue::from_str(
                "base URL must start with http:// or https://",
            ));
        }

        let path = path.trim();
        if path.is_empty() || path.contains("://") {
            return Err(JsValue::from_str(
                "path must be a non-empty relative API path",
            ));
        }

        let url = format!(
            "{}/{}",
            base.trim_end_matches('/'),
            path.trim_start_matches('/')
        );

        fetch_json_with_timeout(&url, self.timeout_ms).await
    }
}

/// Parsed browser response metadata and JSON body.
#[derive(Debug, Clone)]
pub struct HttpResponse {
    /// HTTP status code for the successful response.
    pub status: u16,
    /// Request ID returned by the server, when available.
    pub request_id: Option<String>,
    /// Parsed JSON body.
    pub body: JsValue,
}

impl HttpResponse {
    /// Reads a required string field from a route-specific JSON object.
    pub fn string_field(value: &JsValue, field: &str) -> Result<String, JsValue> {
        let field_value = Reflect::get(value, &JsValue::from_str(field))?;
        field_value
            .as_string()
            .ok_or_else(|| JsValue::from_str(&format!("field '{field}' must be a string")))
    }
}

/// Parsed top-level fields of the shared successful HTTP/JSON envelope.
#[derive(Debug, Clone)]
pub struct ApiEnvelope {
    /// Route-specific JSON payload.
    pub data: JsValue,
    /// Contract version from meta.api_version.
    pub api_version: String,
}

impl ApiEnvelope {
    /// Parses the data/meta envelope used by the family HTTP/JSON contract.
    pub fn from_js(value: &JsValue) -> Result<Self, JsValue> {
        let data = Reflect::get(value, &JsValue::from_str("data"))?;
        if data.is_null() || data.is_undefined() {
            return Err(JsValue::from_str("response is missing data"));
        }

        let meta = Reflect::get(value, &JsValue::from_str("meta"))?;
        if meta.is_null() || meta.is_undefined() {
            return Err(JsValue::from_str("response is missing meta"));
        }

        let version = Reflect::get(&meta, &JsValue::from_str("api_version"))?;
        let api_version = version
            .as_string()
            .ok_or_else(|| JsValue::from_str("meta.api_version must be a string"))?;

        Ok(Self { data, api_version })
    }

    /// Reads a required string field from the envelope's data object.
    pub fn string_field(&self, field: &str) -> Result<String, JsValue> {
        HttpResponse::string_field(&self.data, field)
    }
}

/// Converts common browser/network rejection values into displayable text.
pub fn describe_js_error(error: &JsValue) -> String {
    if let Some(message) = error.as_string() {
        return message;
    }

    if let Ok(message) = Reflect::get(error, &JsValue::from_str("message")) {
        if let Some(message) = message.as_string() {
            return message;
        }
    }

    format!("{error:?}")
}

struct TimeoutGuard {
    window: Window,
    handle: i32,
    callback: Rc<RefCell<Option<Closure<dyn FnMut()>>>>,
}

impl Drop for TimeoutGuard {
    fn drop(&mut self) {
        let _ = self.window.clear_timeout_with_handle(self.handle);
        let _ = self.callback.borrow_mut().take();
    }
}

fn timeout_error_if_needed(
    timed_out: &Cell<bool>,
    timeout_ms: u32,
    original_error: JsValue,
) -> JsValue {
    if timed_out.get() {
        JsValue::from_str(&format!("request timed out after {timeout_ms} ms"))
    } else {
        original_error
    }
}

async fn fetch_json_with_timeout(url: &str, timeout_ms: u32) -> Result<HttpResponse, JsValue> {
    let window =
        web_sys::window().ok_or_else(|| JsValue::from_str("browser window is not available"))?;
    let controller = AbortController::new()?;
    let signal = controller.signal();

    let options = RequestInit::new();
    options.set_method("GET");
    options.set_signal(Some(&signal));

    let request = Request::new_with_str_and_init(url, &options)?;
    request.headers().set("Accept", "application/json")?;

    let timed_out = Rc::new(Cell::new(false));
    let timed_out_for_timer = timed_out.clone();
    let controller_for_timer = controller.clone();
    let timeout_callback = Closure::<dyn FnMut()>::new(move || {
        timed_out_for_timer.set(true);
        let _ = controller_for_timer.abort();
    });
    let timeout_callback = Rc::new(RefCell::new(Some(timeout_callback)));

    let timeout_handle = {
        let borrowed = timeout_callback.borrow();
        let callback = borrowed
            .as_ref()
            .ok_or_else(|| JsValue::from_str("request timeout callback is unavailable"))?;
        window.set_timeout_with_callback_and_timeout_and_arguments_0(
            callback.as_ref().unchecked_ref(),
            timeout_ms.min(i32::MAX as u32) as i32,
        )?
    };

    let _timeout_guard = TimeoutGuard {
        window: window.clone(),
        handle: timeout_handle,
        callback: timeout_callback,
    };

    let response_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|error| timeout_error_if_needed(&timed_out, timeout_ms, error))?;
    let response = response_value
        .dyn_into::<Response>()
        .map_err(|_| JsValue::from_str("fetch returned an invalid response object"))?;

    let status = response.status();
    let request_id = response.headers().get("x-request-id")?;
    if !response.ok() {
        let message = match request_id {
            Some(request_id) => format!("HTTP {status}; request ID: {request_id}"),
            None => format!("HTTP {status}"),
        };
        return Err(JsValue::from_str(&message));
    }

    let body = JsFuture::from(response.json()?)
        .await
        .map_err(|error| timeout_error_if_needed(&timed_out, timeout_ms, error))?;

    Ok(HttpResponse {
        status,
        request_id,
        body,
    })
}
