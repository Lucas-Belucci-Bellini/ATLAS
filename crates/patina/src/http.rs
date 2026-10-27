//! Browser HTTP/JSON client for ATLAS (WASM).
//!
//! The client uses the browser Fetch API and returns a JavaScript Promise so
//! it can run without a runtime dependency on a particular async executor.
//! Network failures, non-success status codes, invalid JSON and timeouts reject
//! the Promise. JSON values are returned as JsValue because this crate
//! intentionally does not impose a serialization framework.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use js_sys::{Object, Promise, Reflect};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
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
    /// The returned Promise resolves to an HttpResponse containing the parsed
    /// JSON body, status and x-request-id when present. The Promise is rejected
    /// for network errors, non-2xx HTTP responses, invalid JSON, or timeouts.
    pub fn get_json(&self, path: &str) -> Result<Promise, JsValue> {
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

        let url = format!("{}/{}", base.trim_end_matches('/'), path.trim_start_matches('/'));
        fetch_json_with_timeout(&url, self.timeout_ms)
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
    /// Decodes the object produced by HttpClient.
    pub fn from_js(value: &JsValue) -> Result<Self, JsValue> {
        let status_value = Reflect::get(value, &JsValue::from_str("status"))?;
        let status = status_value
            .as_f64()
            .filter(|status| status.fract() == 0.0 && (200.0..=599.0).contains(status))
            .ok_or_else(|| JsValue::from_str("response status must be a valid HTTP status"))?
            as u16;

        let request_id_value = Reflect::get(value, &JsValue::from_str("request_id"))?;
        let request_id = if request_id_value.is_null() || request_id_value.is_undefined() {
            None
        } else {
            Some(
                request_id_value
                    .as_string()
                    .ok_or_else(|| JsValue::from_str("response request_id must be a string"))?,
            )
        };

        let body = Reflect::get(value, &JsValue::from_str("body"))?;
        if body.is_null() || body.is_undefined() {
            return Err(JsValue::from_str("response is missing JSON body"));
        }

        Ok(Self { status, request_id, body })
    }
}

/// Parsed top-level fields of the shared successful HTTP/JSON envelope.
///
/// data remains a JsValue; callers decode route-specific fields explicitly.
#[derive(Debug, Clone)]
pub struct ApiEnvelope {
    /// Route-specific JSON payload.
    pub data: JsValue,
    /// Contract version from meta.api_version.
    pub api_version: String,
}

impl ApiEnvelope {
    /// Parses the { "data": ..., "meta": { "api_version": "..." } } envelope.
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

    /// Reads a required string field from a route-specific JSON object.
    pub fn string_field(value: &JsValue, field: &str) -> Result<String, JsValue> {
        let field_value = Reflect::get(value, &JsValue::from_str(field))?;
        field_value
            .as_string()
            .ok_or_else(|| JsValue::from_str(&format!("field {field} must be a string")))
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

fn fetch_json_with_timeout(url: &str, timeout_ms: u32) -> Result<Promise, JsValue> {
    let window = web_sys::window()
        .ok_or_else(|| JsValue::from_str("browser window is not available"))?;
    let controller = AbortController::new()?;
    let signal = controller.signal();

    let options = RequestInit::new();
    options.set_method("GET");
    options.set_signal(Some(&signal));

    let request = Request::new_with_str_and_init(url, &options)?;
    request.headers().set("Accept", "application/json")?;

    let fetch_promise = window.fetch_with_request(&request);
    let timed_out = Rc::new(Cell::new(false));
    let timed_out_for_timer = timed_out.clone();
    let controller_for_timer = controller.clone();

    let timeout_closure = Closure::<dyn FnMut()>::new(move || {
        timed_out_for_timer.set(true);
        let _ = controller_for_timer.abort();
    });
    let timeout_slot: Rc<RefCell<Option<Closure<dyn FnMut()>>>> =
        Rc::new(RefCell::new(Some(timeout_closure)));

    let timeout_handle = {
        let borrowed = timeout_slot.borrow();
        let callback = borrowed
            .as_ref()
            .ok_or_else(|| JsValue::from_str("request timeout callback is unavailable"))?;
        window.set_timeout_with_callback_and_timeout_and_arguments_0(
            callback.as_ref().unchecked_ref(),
            timeout_ms.min(i32::MAX as u32) as i32,
        )?
    };

    // Returning the Response.json() Promise chains body parsing without
    // requiring an async runtime dependency in this framework crate.
    let on_response = Closure::once_into_js(move |value: JsValue| -> JsValue {
        let response = match value.dyn_into::<Response>() {
            Ok(response) => response,
            Err(_) => {
                return Promise::reject(&JsValue::from_str(
                    "fetch returned an invalid response object",
                ))
                .into();
            }
        };

        let status = response.status();
        let request_id = match response.headers().get("x-request-id") {
            Ok(request_id) => request_id,
            Err(error) => return Promise::reject(&error).into(),
        };

        if !response.ok() {
            let message = match request_id {
                Some(request_id) => format!("HTTP {status}; request ID: {request_id}"),
                None => format!("HTTP {status}"),
            };
            return Promise::reject(&JsValue::from_str(&message)).into();
        }

        let json_promise = match response.json() {
            Ok(json_promise) => json_promise,
            Err(error) => return Promise::reject(&error).into(),
        };

        let on_body = Closure::once_into_js(move |body: JsValue| -> JsValue {
            let object = Object::new();
            let status_value = JsValue::from_f64(status as f64);
            let request_id_value = request_id
                .as_ref()
                .map(|id| JsValue::from_str(id))
                .unwrap_or(JsValue::NULL);

            let result = (|| -> Result<(), JsValue> {
                Reflect::set(
                    &object,
                    &JsValue::from_str("status"),
                    &status_value,
                )?;
                Reflect::set(
                    &object,
                    &JsValue::from_str("request_id"),
                    &request_id_value,
                )?;
                Reflect::set(&object, &JsValue::from_str("body"), &body)?;
                Ok(())
            })();

            match result {
                Ok(()) => object.into(),
                Err(error) => Promise::reject(&error).into(),
            }
        });
        let on_body_error = Closure::once_into_js(move |reason: JsValue| -> JsValue {
            Promise::reject(&reason).into()
        });
        json_promise
            .then2(on_body.unchecked_ref(), on_body_error.unchecked_ref())
            .into()
    });

    let on_fetch_error = Closure::once_into_js(move |reason: JsValue| -> JsValue {
        Promise::reject(&reason).into()
    });

    let response_chain = fetch_promise.then2(
        on_response.unchecked_ref(),
        on_fetch_error.unchecked_ref(),
    );

    let window_on_success: Window = window.clone();
    let timer_on_success = timeout_slot.clone();
    let on_success = Closure::once_into_js(move |value: JsValue| -> JsValue {
        let _ = window_on_success.clear_timeout_with_handle(timeout_handle);
        timer_on_success.borrow_mut().take();
        value
    });

    let window_on_error = window;
    let timer_on_error = timeout_slot;
    let timed_out_on_error = timed_out;
    let on_error = Closure::once_into_js(move |reason: JsValue| -> JsValue {
        let _ = window_on_error.clear_timeout_with_handle(timeout_handle);
        timer_on_error.borrow_mut().take();

        if timed_out_on_error.get() {
            Promise::reject(&JsValue::from_str(&format!(
                "request timed out after {timeout_ms} ms"
            )))
            .into()
        } else {
            Promise::reject(&reason).into()
        }
    });

    Ok(response_chain.then2(
        on_success.unchecked_ref(),
        on_error.unchecked_ref(),
    ))
}
