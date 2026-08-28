//! Client for the Addis Hotel Booking API.
//!
//! The dashboard is a pure client-side app, so every call is a browser `fetch`
//! carrying a JWT. [`session`] owns the tokens and persists them to
//! `localStorage`; [`request`] attaches the access token and transparently
//! retries once through `/accounts/auth/token/refresh/` when the API reports
//! the token expired.
//!
//! # Wire format
//!
//! The published OpenAPI schema documents many responses as an untyped `{}`,
//! and describes pagination as plain DRF (`{count, next, previous, results}`).
//! Neither matches the service, which always wraps:
//!
//! ```json
//! { "success": true, "message": "…", "data": … ,
//!   "meta": { "count": 0, "page": 1, "pages": 1, "page_size": 10 } }
//! ```
//!
//! and on failure:
//!
//! ```json
//! { "success": false, "code": "VALIDATION_ERROR", "message": "…",
//!   "errors": { "field": ["…"] }, "status_code": 400 }
//! ```
//!
//! Every type here was verified against the running backend, not the schema.

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use wasm_bindgen::JsCast;

pub const API_BASE: &str = "https://addisapi.pastaromatour.com/api/v1";

const ACCESS_KEY: &str = "addis.access";
const REFRESH_KEY: &str = "addis.refresh";
const USER_KEY: &str = "addis.user";
const ORG_KEY: &str = "addis.org";

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub struct ApiError {
    pub status: u16,
    /// The API's `code`, e.g. `VALIDATION_ERROR` or `AUTHENTICATION_FAILED`.
    pub code: String,
    pub message: String,
    /// Per-field validation messages, flattened to one string per field.
    pub fields: Vec<(String, String)>,
}

impl ApiError {
    fn local(message: impl Into<String>) -> Self {
        Self {
            status: 0,
            code: "CLIENT_ERROR".into(),
            message: message.into(),
            fields: Vec::new(),
        }
    }

    pub fn is_unauthorized(&self) -> bool {
        self.status == 401
    }

    pub fn is_forbidden(&self) -> bool {
        self.status == 403
    }

    /// The message for a specific field, if the API rejected one.
    pub fn field(&self, name: &str) -> Option<&str> {
        self.fields
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }

    /// First field message, whichever field it is — handy for a form banner
    /// when the caller does not know which field the API will object to.
    pub fn any_field(&self) -> Option<String> {
        self.fields.first().map(|(k, v)| {
            if k == "non_field_errors" {
                v.clone()
            } else {
                format!("{k}: {v}")
            }
        })
    }

    /// Message to show a user: the field detail when there is one, else the
    /// envelope's message.
    pub fn detail(&self) -> String {
        self.any_field().unwrap_or_else(|| self.message.clone())
    }

    fn parse(status: u16, body: &str) -> Self {
        let Ok(v) = serde_json::from_str::<serde_json::Value>(body) else {
            return Self {
                status,
                code: "UNKNOWN".into(),
                message: format!("Unexpected response from the server (HTTP {status})."),
                fields: Vec::new(),
            };
        };
        let message = v
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("Something went wrong.")
            .to_string();
        let code = v
            .get("code")
            .and_then(|c| c.as_str())
            .unwrap_or("UNKNOWN")
            .to_string();
        let fields = v
            .get("errors")
            .and_then(|e| e.as_object())
            .map(|obj| {
                obj.iter()
                    .filter(|(k, _)| *k != "code" && *k != "messages")
                    .map(|(k, val)| {
                        let text = match val {
                            serde_json::Value::Array(a) => a
                                .iter()
                                .filter_map(|x| x.as_str())
                                .collect::<Vec<_>>()
                                .join(" "),
                            serde_json::Value::String(s) => s.clone(),
                            other => other.to_string(),
                        };
                        (k.clone(), text)
                    })
                    .collect()
            })
            .unwrap_or_default();
        Self {
            status,
            code,
            message,
            fields,
        }
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

pub type ApiResult<T> = Result<T, ApiError>;

// ---------------------------------------------------------------------------
// Pagination
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Meta {
    #[serde(default)]
    pub count: u32,
    #[serde(default = "one")]
    pub page: u32,
    #[serde(default = "one")]
    pub pages: u32,
    #[serde(default)]
    pub page_size: u32,
}

fn one() -> u32 {
    1
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub meta: Meta,
}

impl<T> Page<T> {
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
    pub fn len(&self) -> usize {
        self.items.len()
    }
}

/// Query-string pairs. Blank values are dropped, so callers can push
/// unconditionally and let the API apply its own defaults.
pub type Params = Vec<(&'static str, String)>;

// ---------------------------------------------------------------------------
// Session
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct RoleRef {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub uuid: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub is_system_role: bool,
}

/// One entry of the user's `assigned_organizations` — the hotels this account
/// may act on, and with which role.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct AssignedOrg {
    #[serde(default)]
    pub account_id: i64,
    #[serde(default)]
    pub organization_id: i64,
    #[serde(default)]
    pub organization_uuid: String,
    #[serde(default)]
    pub organization_name: String,
    #[serde(default)]
    pub role: Option<RoleRef>,
}

/// The signed-in user.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct User {
    #[serde(default)]
    pub id: Option<i64>,
    #[serde(default)]
    pub uuid: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub first_name: Option<String>,
    #[serde(default)]
    pub last_name: Option<String>,
    #[serde(default)]
    pub user_type: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub city: Option<String>,
    #[serde(default)]
    pub address_1: Option<String>,
    #[serde(default)]
    pub address_2: Option<String>,
    #[serde(default)]
    pub postal_code: Option<String>,
    #[serde(default)]
    pub profile_picture: Option<String>,
    #[serde(default)]
    pub is_superuser: bool,
    #[serde(default)]
    pub date_joined: Option<String>,
    #[serde(default)]
    pub assigned_organizations: Vec<AssignedOrg>,
    #[serde(default)]
    pub permissions: Vec<String>,
}

impl User {
    /// Best available display name, falling back to the email local part.
    pub fn display_name(&self) -> String {
        let full = [self.first_name.as_deref(), self.last_name.as_deref()]
            .into_iter()
            .flatten()
            .filter(|s| !s.trim().is_empty())
            .collect::<Vec<_>>()
            .join(" ");
        if !full.is_empty() {
            return full;
        }
        self.email
            .as_deref()
            .and_then(|e| e.split('@').next())
            .unwrap_or("Account")
            .to_string()
    }

    pub fn initials(&self) -> String {
        self.display_name()
            .split_whitespace()
            .filter_map(|w| w.chars().next())
            .take(2)
            .collect::<String>()
            .to_uppercase()
    }

    /// The role the API assigned inside the active hotel, when there is one.
    pub fn org_role(&self) -> Option<&str> {
        self.assigned_organizations
            .first()
            .and_then(|o| o.role.as_ref())
            .map(|r| r.name.as_str())
    }

    /// Human-readable role for the topbar.
    pub fn role_label(&self) -> String {
        if let Some(r) = self.org_role() {
            return r.to_string();
        }
        match self.user_type.as_deref() {
            Some("superadmin") => "Superadmin".into(),
            Some("organization_admin") => "Administrator".into(),
            Some("general_customer") => "Customer".into(),
            _ if self.is_superuser => "Superadmin".into(),
            _ => "Staff".into(),
        }
    }

    pub fn is_superadmin(&self) -> bool {
        self.is_superuser || self.user_type.as_deref() == Some("superadmin")
    }

    /// Whether the API granted a permission code (e.g. `rooms.manage`).
    /// Superadmins are treated as holding everything.
    pub fn can(&self, code: &str) -> bool {
        self.is_superadmin() || self.permissions.iter().any(|p| p == code)
    }
}

pub mod session {
    //! Token, user and active-hotel storage.
    //!
    //! `localStorage` can throw outright (private mode, blocked site data), so
    //! every access is fallible and simply degrades to an unauthenticated
    //! session rather than panicking the app.

    use super::{User, ACCESS_KEY, ORG_KEY, REFRESH_KEY, USER_KEY};

    fn store() -> Option<web_sys::Storage> {
        web_sys::window()?.local_storage().ok()?
    }

    fn get(key: &str) -> Option<String> {
        store()?.get_item(key).ok()?.filter(|s| !s.is_empty())
    }

    fn set(key: &str, value: &str) {
        if let Some(s) = store() {
            let _ = s.set_item(key, value);
        }
    }

    fn remove(key: &str) {
        if let Some(s) = store() {
            let _ = s.remove_item(key);
        }
    }

    pub fn access_token() -> Option<String> {
        get(ACCESS_KEY)
    }

    pub fn refresh_token() -> Option<String> {
        get(REFRESH_KEY)
    }

    pub fn is_authenticated() -> bool {
        access_token().is_some()
    }

    pub fn user() -> Option<User> {
        serde_json::from_str(&get(USER_KEY)?).ok()
    }

    pub fn set_tokens(access: &str, refresh: Option<&str>) {
        set(ACCESS_KEY, access);
        if let Some(r) = refresh {
            set(REFRESH_KEY, r);
        }
    }

    pub fn set_user(user: &User) {
        if let Ok(s) = serde_json::to_string(user) {
            set(USER_KEY, &s);
        }
        // Adopt the first hotel the API assigned unless the operator already
        // picked one; a fresh admin has none until they create a property.
        if active_org().is_none() {
            if let Some(o) = user.assigned_organizations.first() {
                set_active_org(o.organization_id);
            }
        }
    }

    /// Id of the hotel the dashboard is currently acting on. Org-scoped
    /// endpoints (`/organizations/{id}/…`) need it in the path.
    pub fn active_org() -> Option<i64> {
        get(ORG_KEY)?.parse().ok()
    }

    pub fn set_active_org(id: i64) {
        set(ORG_KEY, &id.to_string());
    }

    pub fn clear() {
        remove(ACCESS_KEY);
        remove(REFRESH_KEY);
        remove(USER_KEY);
        remove(ORG_KEY);
    }
}

/// The active hotel id, or a `CLIENT_ERROR` telling the operator to create one.
///
/// Every org-scoped path needs this, and a brand-new admin account genuinely
/// has no hotel yet, so it is a real error state rather than an assertion.
pub fn require_org() -> ApiResult<i64> {
    session::active_org().ok_or_else(|| {
        ApiError::local("No hotel is selected yet. Create your property under Hotel Profile first.")
    })
}

// ---------------------------------------------------------------------------
// Transport
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Method {
    Get,
    Post,
    Patch,
    Put,
    Delete,
}

impl Method {
    fn as_str(self) -> &'static str {
        match self {
            Method::Get => "GET",
            Method::Post => "POST",
            Method::Patch => "PATCH",
            Method::Put => "PUT",
            Method::Delete => "DELETE",
        }
    }
    fn gloo(self) -> gloo_net::http::Method {
        match self {
            Method::Get => gloo_net::http::Method::GET,
            Method::Post => gloo_net::http::Method::POST,
            Method::Patch => gloo_net::http::Method::PATCH,
            Method::Put => gloo_net::http::Method::PUT,
            Method::Delete => gloo_net::http::Method::DELETE,
        }
    }
}

/// Whether a call should carry the access token and be retried after refresh.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Auth {
    Required,
    None,
}

/// What to send as the request body.
enum Body<'a> {
    Empty,
    Json(&'a serde_json::Value),
    /// `multipart/form-data`; the browser sets the boundary itself, so no
    /// `Content-Type` header must be attached.
    Form(&'a web_sys::FormData),
}

async fn send_raw(
    method: Method,
    path: &str,
    body: Body<'_>,
    auth: Auth,
) -> ApiResult<(u16, String)> {
    let url = format!("{API_BASE}{path}");
    let mut req = gloo_net::http::RequestBuilder::new(&url).method(method.gloo());

    if auth == Auth::Required {
        if let Some(token) = session::access_token() {
            req = req.header("Authorization", &format!("Bearer {token}"));
        }
    }

    let sent = match body {
        Body::Empty => req.send().await,
        Body::Json(json) => req
            .header("Content-Type", "application/json")
            .json(json)
            .map_err(|e| ApiError::local(format!("could not encode request: {e}")))?
            .send()
            .await,
        Body::Form(form) => req
            .body(form)
            .map_err(|e| ApiError::local(format!("could not attach upload: {e}")))?
            .send()
            .await,
    };

    let res = sent.map_err(|e| {
        ApiError::local(format!(
            "Could not reach the API ({} {path}). {e}",
            method.as_str()
        ))
    })?;

    let status = res.status();
    let text = res
        .text()
        .await
        .map_err(|e| ApiError::local(format!("could not read response: {e}")))?;
    Ok((status, text))
}

/// Exchanges the stored refresh token for a new access token.
///
/// Returns `false` when there is nothing to refresh with or the API rejects it,
/// in which case the caller should surface the original 401.
async fn try_refresh() -> bool {
    let Some(refresh) = session::refresh_token() else {
        return false;
    };
    let body = serde_json::json!({ "refresh": refresh });
    let Ok((status, text)) = send_raw(
        Method::Post,
        "/accounts/auth/token/refresh/",
        Body::Json(&body),
        Auth::None,
    )
    .await
    else {
        return false;
    };
    if !(200..300).contains(&status) {
        return false;
    }
    let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) else {
        return false;
    };
    let payload = v.get("data").unwrap_or(&v);
    match payload.get("access").and_then(|a| a.as_str()) {
        Some(access) => {
            session::set_tokens(access, payload.get("refresh").and_then(|r| r.as_str()));
            true
        }
        None => false,
    }
}

/// Sends a request, retrying once through the refresh endpoint on a 401.
async fn send_with_retry(
    method: Method,
    path: &str,
    body: Body<'_>,
    auth: Auth,
) -> ApiResult<(u16, String)> {
    let first = send_raw(method, path, copy_body(&body), auth).await?;
    if first.0 == 401 && auth == Auth::Required && try_refresh().await {
        return send_raw(method, path, body, auth).await;
    }
    Ok(first)
}

/// `Body` holds only references, so re-sending is a shallow copy.
fn copy_body<'a>(body: &Body<'a>) -> Body<'a> {
    match body {
        Body::Empty => Body::Empty,
        Body::Json(v) => Body::Json(v),
        Body::Form(f) => Body::Form(f),
    }
}

/// The envelope's `data`, as raw JSON.
pub async fn request_value(
    method: Method,
    path: &str,
    body: Option<serde_json::Value>,
    auth: Auth,
) -> ApiResult<serde_json::Value> {
    let b = match &body {
        Some(v) => Body::Json(v),
        None => Body::Empty,
    };
    let (status, text) = send_with_retry(method, path, b, auth).await?;
    unwrap_data(status, &text)
}

fn unwrap_data(status: u16, text: &str) -> ApiResult<serde_json::Value> {
    if !(200..300).contains(&status) {
        return Err(ApiError::parse(status, text));
    }
    if text.trim().is_empty() {
        return Ok(serde_json::Value::Null);
    }
    let v: serde_json::Value = serde_json::from_str(text)
        .map_err(|e| ApiError::local(format!("could not decode response: {e}")))?;
    Ok(v.get("data").cloned().unwrap_or(v))
}

/// Typed single-object request.
pub async fn request<T: DeserializeOwned>(
    method: Method,
    path: &str,
    body: Option<serde_json::Value>,
    auth: Auth,
) -> ApiResult<T> {
    let data = request_value(method, path, body, auth).await?;
    serde_json::from_value(data)
        .map_err(|e| ApiError::local(format!("unexpected response shape: {e}")))
}

/// A collection that the API returns unpaginated (`data` is a bare array).
pub async fn request_list<T: DeserializeOwned>(path: &str, params: &Params) -> ApiResult<Vec<T>> {
    let data = request_value(Method::Get, &with_query(path, params), None, Auth::Required).await?;
    if data.is_null() {
        return Ok(Vec::new());
    }
    serde_json::from_value(data).map_err(|e| ApiError::local(format!("unexpected list shape: {e}")))
}

/// Typed paginated request, preserving `meta`.
pub async fn request_page<T: DeserializeOwned>(path: &str, params: &Params) -> ApiResult<Page<T>> {
    let path = with_query(path, params);
    let (status, text) = send_with_retry(Method::Get, &path, Body::Empty, Auth::Required).await?;
    if !(200..300).contains(&status) {
        return Err(ApiError::parse(status, &text));
    }
    let v: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| ApiError::local(format!("could not decode response: {e}")))?;
    let items = serde_json::from_value::<Vec<T>>(
        v.get("data").cloned().unwrap_or(serde_json::Value::Array(vec![])),
    )
    .map_err(|e| ApiError::local(format!("unexpected list shape: {e}")))?;
    let meta = v
        .get("meta")
        .and_then(|m| serde_json::from_value::<Meta>(m.clone()).ok())
        .unwrap_or(Meta {
            count: items.len() as u32,
            page: 1,
            pages: 1,
            page_size: items.len() as u32,
        });
    Ok(Page { items, meta })
}

/// Uploads a `multipart/form-data` body (photos, spreadsheets, icons).
pub async fn request_form<T: DeserializeOwned>(
    method: Method,
    path: &str,
    form: &web_sys::FormData,
) -> ApiResult<T> {
    let (status, text) = send_with_retry(method, path, Body::Form(form), Auth::Required).await?;
    let data = unwrap_data(status, &text)?;
    serde_json::from_value(data)
        .map_err(|e| ApiError::local(format!("unexpected response shape: {e}")))
}

fn with_query(path: &str, params: &Params) -> String {
    let qs = params
        .iter()
        .filter(|(_, v)| !v.trim().is_empty())
        .map(|(k, v)| format!("{k}={}", urlencode(v)))
        .collect::<Vec<_>>()
        .join("&");
    if qs.is_empty() {
        path.to_string()
    } else if path.contains('?') {
        format!("{path}&{qs}")
    } else {
        format!("{path}?{qs}")
    }
}

/// Percent-encodes a query value. Small hand-rolled version so the crate does
/// not need a URL dependency for the handful of characters that matter here.
fn urlencode(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            b' ' => "+".to_string(),
            other => format!("%{other:02X}"),
        })
        .collect()
}

/// Fetches a file endpoint with the access token and hands it to the browser
/// as a download.
///
/// The CSV endpoints (`/guests/export/`, the bulk-upload template) are behind
/// `Authorization`, so a plain `<a download>` cannot reach them — the bytes have
/// to come through `fetch` and go back out as a blob URL.
pub async fn download(path: &str, filename: &str) -> ApiResult<()> {
    let url = format!("{API_BASE}{path}");
    let mut req = gloo_net::http::RequestBuilder::new(&url).method(gloo_net::http::Method::GET);
    if let Some(token) = session::access_token() {
        req = req.header("Authorization", &format!("Bearer {token}"));
    }
    let res = req
        .send()
        .await
        .map_err(|e| ApiError::local(format!("Could not reach the API. {e}")))?;

    if !(200..300).contains(&res.status()) {
        let status = res.status();
        let text = res.text().await.unwrap_or_default();
        return Err(ApiError::parse(status, &text));
    }

    let bytes = res
        .binary()
        .await
        .map_err(|e| ApiError::local(format!("could not read the file: {e}")))?;

    let array = js_sys::Uint8Array::from(bytes.as_slice());
    let parts = js_sys::Array::new();
    parts.push(&array);
    let opts = web_sys::BlobPropertyBag::new();
    opts.set_type("text/csv");
    let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(&parts, &opts)
        .map_err(|_| ApiError::local("could not build the download"))?;
    let href = web_sys::Url::create_object_url_with_blob(&blob)
        .map_err(|_| ApiError::local("could not build the download"))?;

    let doc = web_sys::window()
        .and_then(|w| w.document())
        .ok_or_else(|| ApiError::local("no document"))?;
    let a = doc
        .create_element("a")
        .map_err(|_| ApiError::local("no document"))?
        .unchecked_into::<web_sys::HtmlAnchorElement>();
    a.set_href(&href);
    a.set_download(filename);
    a.click();
    let _ = web_sys::Url::revoke_object_url(&href);
    Ok(())
}


// ---------------------------------------------------------------------------
// Formatting helpers
// ---------------------------------------------------------------------------

/// `12345` -> `"12,345"`.
pub fn thousands(n: i64) -> String {
    let neg = n < 0;
    let s = n.abs().to_string();
    let mut out = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    let mut res: String = out.chars().rev().collect();
    if neg {
        res.insert(0, '-');
    }
    res
}

/// A decimal string from the API (`"10292.50"`) as `"10,292.50"`.
/// Blank, null or unparseable input becomes an em dash.
pub fn money(raw: Option<&str>) -> String {
    match raw.and_then(|s| s.trim().parse::<f64>().ok()) {
        Some(n) => money_f(n),
        None => "—".into(),
    }
}

pub fn money_f(n: f64) -> String {
    let whole = n.trunc() as i64;
    let cents = ((n.abs().fract() * 100.0).round()) as i64;
    format!("{}.{:02}", thousands(whole), cents.min(99))
}

/// Rounds to whole units, for tiles where cents are noise.
pub fn money_round(raw: Option<&str>) -> String {
    match raw.and_then(|s| s.trim().parse::<f64>().ok()) {
        Some(n) => thousands(n.round() as i64),
        None => "—".into(),
    }
}

/// `"2026-08-28"` -> `"28 Aug 2026"`. Anything unexpected passes through.
pub fn pretty_date(raw: Option<&str>) -> String {
    let Some(s) = raw.map(str::trim).filter(|s| !s.is_empty()) else {
        return "—".into();
    };
    let date = s.split('T').next().unwrap_or(s);
    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() != 3 {
        return s.to_string();
    }
    let months = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    match (parts[1].parse::<usize>(), parts[2].parse::<u32>()) {
        (Ok(m), Ok(d)) if (1..=12).contains(&m) => {
            format!("{:02} {} {}", d, months[m - 1], parts[0])
        }
        _ => s.to_string(),
    }
}

/// `"2026-08-27T18:10:05.393278Z"` -> `"27 Aug 2026, 18:10"`.
pub fn pretty_datetime(raw: Option<&str>) -> String {
    let Some(s) = raw.map(str::trim).filter(|s| !s.is_empty()) else {
        return "—".into();
    };
    let mut it = s.splitn(2, 'T');
    let date = pretty_date(it.next());
    match it.next() {
        Some(time) => {
            let hm: String = time.chars().take(5).collect();
            format!("{date}, {hm}")
        }
        None => date,
    }
}

// ---------------------------------------------------------------------------
// Authentication
// ---------------------------------------------------------------------------

/// Locates the token pair in a login/register/OTP response and stores it.
fn adopt_tokens(data: &serde_json::Value) -> ApiResult<()> {
    let tokens = data
        .get("tokens")
        .or_else(|| data.get("token"))
        .unwrap_or(data);
    let access = tokens
        .get("access")
        .or_else(|| tokens.get("access_token"))
        .and_then(|a| a.as_str())
        .ok_or_else(|| {
            ApiError::local("Signed in, but the server did not return an access token.")
        })?;
    let refresh = tokens
        .get("refresh")
        .or_else(|| tokens.get("refresh_token"))
        .and_then(|r| r.as_str());
    session::set_tokens(access, refresh);
    Ok(())
}

/// Stores the user carried by an auth response, falling back to `/accounts/me/`.
async fn adopt_user(data: &serde_json::Value) -> User {
    let user = match data.get("user") {
        Some(u) => serde_json::from_value::<User>(u.clone()).unwrap_or_default(),
        None => me().await.unwrap_or_default(),
    };
    session::set_user(&user);
    user
}

/// Signs in with an email **or** phone number and stores the resulting tokens.
pub async fn login(identifier: &str, password: &str) -> ApiResult<User> {
    let body = serde_json::json!({
        "identifier": identifier.trim(),
        "password": password,
    });
    let data = request_value(Method::Post, "/accounts/auth/login/", Some(body), Auth::None).await?;
    adopt_tokens(&data)?;
    Ok(adopt_user(&data).await)
}

/// Fields the signup form collects. `user_type` defaults to
/// `organization_admin`, which is what a hotel signing up needs.
#[derive(Debug, Clone, Default, Serialize)]
pub struct RegisterPayload {
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub phone: String,
    pub user_type: String,
}

/// Creates an account. The API returns a token pair straight away, so this also
/// signs the new admin in.
pub async fn register(payload: &RegisterPayload) -> ApiResult<User> {
    let body = serde_json::to_value(payload)
        .map_err(|e| ApiError::local(format!("could not encode signup: {e}")))?;
    let data = request_value(
        Method::Post,
        "/accounts/auth/register/",
        Some(body),
        Auth::None,
    )
    .await?;
    adopt_tokens(&data)?;
    Ok(adopt_user(&data).await)
}

/// The signed-in user's profile. Refreshes the cached copy as a side effect so
/// permissions and hotel assignments stay current.
pub async fn me() -> ApiResult<User> {
    let user: User = request(Method::Get, "/accounts/me/", None, Auth::Required).await?;
    session::set_user(&user);
    Ok(user)
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct ProfileUpdate {
    pub first_name: String,
    pub last_name: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub phone: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub address_1: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub city: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub postal_code: String,
}

pub async fn update_profile(payload: &ProfileUpdate) -> ApiResult<User> {
    let body = serde_json::to_value(payload)
        .map_err(|e| ApiError::local(format!("could not encode profile: {e}")))?;
    let user: User = request(Method::Patch, "/accounts/me/", Some(body), Auth::Required).await?;
    session::set_user(&user);
    Ok(user)
}

pub async fn change_password(old: &str, new: &str) -> ApiResult<()> {
    let body = serde_json::json!({ "old_password": old, "new_password": new });
    request_value(
        Method::Post,
        "/accounts/auth/password/change/",
        Some(body),
        Auth::Required,
    )
    .await?;
    Ok(())
}

/// Requests a password-reset OTP by email or SMS.
pub async fn request_password_reset(identifier: &str) -> ApiResult<()> {
    let body = serde_json::json!({ "identifier": identifier.trim() });
    request_value(
        Method::Post,
        "/accounts/auth/password/reset/request/",
        Some(body),
        Auth::None,
    )
    .await?;
    Ok(())
}

/// Completes a reset with the code the API sent out.
pub async fn confirm_password_reset(
    identifier: &str,
    otp_code: &str,
    new_password: &str,
) -> ApiResult<()> {
    let body = serde_json::json!({
        "identifier": identifier.trim(),
        "otp_code": otp_code.trim(),
        "new_password": new_password,
    });
    request_value(
        Method::Post,
        "/accounts/auth/password/reset/confirm/",
        Some(body),
        Auth::None,
    )
    .await?;
    Ok(())
}

/// Clears the local session. The API is stateless for JWTs, so there is
/// nothing to revoke server-side.
pub fn logout() {
    session::clear();
}

// ---------------------------------------------------------------------------
// Organizations (the hotel itself)
// ---------------------------------------------------------------------------

/// A hotel as it appears in `GET /organizations/` — the property switcher.
#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct OrgSummary {
    pub id: i64,
    #[serde(default)]
    pub uuid: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub role_id: Option<i64>,
    #[serde(default)]
    pub role_name: String,
    #[serde(default)]
    pub city: Option<String>,
    #[serde(default)]
    pub country: Option<String>,
    #[serde(default)]
    pub currency: Option<String>,
    #[serde(default)]
    pub star_rating: Option<f32>,
    #[serde(default)]
    pub logo: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct OrgPhoto {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub image_url: String,
    #[serde(default)]
    pub caption: Option<String>,
    #[serde(default)]
    pub is_logo: bool,
    #[serde(default)]
    pub display_order: i32,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct OrgContact {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub phone_number: Option<String>,
    #[serde(default)]
    pub is_primary: bool,
}

/// Property rules. Every field is optional on the wire — the API seeds a row
/// with defaults when the hotel is created and leaves the rest null.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct OrgPolicies {
    #[serde(default)]
    pub id: Option<i64>,
    #[serde(default)]
    pub checkin_time: Option<String>,
    #[serde(default)]
    pub checkout_time: Option<String>,
    #[serde(default)]
    pub children_allowed: Option<bool>,
    #[serde(default)]
    pub children_age: Option<i32>,
    #[serde(default)]
    pub extrabed_available: Option<bool>,
    #[serde(default)]
    pub pet_allowed: Option<bool>,
    #[serde(default)]
    pub smoking_allowed: Option<bool>,
    #[serde(default)]
    pub non_smoking_property: Option<bool>,
    #[serde(default)]
    pub government_id_required: Option<bool>,
    #[serde(default)]
    pub minimum_checkin_age: Option<i32>,
    #[serde(default)]
    pub event: Option<String>,
    #[serde(default)]
    pub bank_card_allow: Option<bool>,
    #[serde(default)]
    pub online_transaction_allow: Option<bool>,
    #[serde(default)]
    pub bank_payment_allow: Option<bool>,
    #[serde(default)]
    pub parties_or_event_allowed: Option<bool>,
    #[serde(default)]
    pub public_note: Option<String>,
}

/// Full hotel profile. Also used as the create/update payload — `Serialize`
/// skips the read-only nested collections.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Organization {
    #[serde(default, skip_serializing)]
    pub id: i64,
    #[serde(default, skip_serializing)]
    pub uuid: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    /// A decimal string when read back, a number when written.
    #[serde(default, skip_serializing)]
    pub star_rating: Option<String>,
    #[serde(default)]
    pub currency: Option<String>,
    #[serde(default)]
    pub business_registration_number: Option<String>,
    #[serde(default)]
    pub tin_number: Option<String>,
    #[serde(default)]
    pub vat_number: Option<String>,
    #[serde(default)]
    pub established_year: Option<i32>,
    #[serde(default)]
    pub address_line: Option<String>,
    #[serde(default)]
    pub latitude: Option<f64>,
    #[serde(default)]
    pub longitude: Option<f64>,
    #[serde(default)]
    pub city: Option<String>,
    #[serde(default)]
    pub postal_code: Option<String>,
    #[serde(default)]
    pub country: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub phone_number: Option<String>,
    #[serde(default)]
    pub whatsapp: Option<String>,
    #[serde(default)]
    pub website_url: Option<String>,
    #[serde(default)]
    pub fb_link: Option<String>,
    #[serde(default)]
    pub instagram_link: Option<String>,
    #[serde(default)]
    pub x_link: Option<String>,
    #[serde(default)]
    pub youtube_channel_link: Option<String>,
    #[serde(default)]
    pub linkedin_url: Option<String>,
    #[serde(default, skip_serializing)]
    pub photos: Vec<OrgPhoto>,
    #[serde(default, skip_serializing)]
    pub policies: Option<OrgPolicies>,
    #[serde(default, skip_serializing)]
    pub contacts: Vec<OrgContact>,
}

impl Organization {
    pub fn stars(&self) -> f32 {
        self.star_rating
            .as_deref()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0)
    }
    pub fn currency_code(&self) -> &str {
        self.currency.as_deref().unwrap_or("ETB")
    }
    /// `"Kazanchis, Addis Ababa, Ethiopia"`, skipping whatever is null.
    pub fn location(&self) -> String {
        [
            self.address_line.as_deref(),
            self.city.as_deref(),
            self.country.as_deref(),
        ]
        .into_iter()
        .flatten()
        .filter(|s| !s.trim().is_empty())
        .collect::<Vec<_>>()
        .join(", ")
    }
    pub fn logo_url(&self) -> Option<String> {
        self.photos
            .iter()
            .find(|p| p.is_logo && !p.image_url.is_empty())
            .map(|p| p.image_url.clone())
    }
}

pub async fn list_organizations() -> ApiResult<Vec<OrgSummary>> {
    request_list("/organizations/", &Vec::new()).await
}

pub async fn get_organization(id: i64) -> ApiResult<Organization> {
    request(
        Method::Get,
        &format!("/organizations/{id}/"),
        None,
        Auth::Required,
    )
    .await
}

/// The hotel the dashboard is acting on.
pub async fn active_organization() -> ApiResult<Organization> {
    get_organization(require_org()?).await
}

/// Creates the hotel and adopts it as the active property.
pub async fn create_organization(
    org: &Organization,
    star_rating: f32,
) -> ApiResult<Organization> {
    let mut body = serde_json::to_value(org)
        .map_err(|e| ApiError::local(format!("could not encode hotel: {e}")))?;
    if let Some(map) = body.as_object_mut() {
        map.insert("star_rating".into(), serde_json::json!(star_rating));
    }
    let created: Organization =
        request(Method::Post, "/organizations/", Some(body), Auth::Required).await?;
    session::set_active_org(created.id);
    // The token only carries hotel permissions once the account exists, so pull
    // a fresh profile before the caller navigates into a scoped page.
    let _ = me().await;
    Ok(created)
}

pub async fn update_organization(
    id: i64,
    org: &Organization,
    star_rating: Option<f32>,
) -> ApiResult<Organization> {
    let mut body = serde_json::to_value(org)
        .map_err(|e| ApiError::local(format!("could not encode hotel: {e}")))?;
    if let (Some(map), Some(stars)) = (body.as_object_mut(), star_rating) {
        map.insert("star_rating".into(), serde_json::json!(stars));
    }
    request(
        Method::Patch,
        &format!("/organizations/{id}/"),
        Some(body),
        Auth::Required,
    )
    .await
}

pub async fn get_policies() -> ApiResult<OrgPolicies> {
    let org = require_org()?;
    request(
        Method::Get,
        &format!("/organizations/{org}/policies/"),
        None,
        Auth::Required,
    )
    .await
}

pub async fn update_policies(policies: &OrgPolicies) -> ApiResult<OrgPolicies> {
    let org = require_org()?;
    let body = serde_json::to_value(policies)
        .map_err(|e| ApiError::local(format!("could not encode policies: {e}")))?;
    request(
        Method::Patch,
        &format!("/organizations/{org}/policies/"),
        Some(body),
        Auth::Required,
    )
    .await
}

pub async fn list_gallery() -> ApiResult<Vec<OrgPhoto>> {
    let org = require_org()?;
    request_list(&format!("/organizations/{org}/gallery/"), &Vec::new()).await
}

/// Adds a gallery photo by URL. The endpoint is multipart and also accepts a
/// binary `image`, but the dashboard only ever links to hosted images.
pub async fn add_gallery_photo(
    image_url: &str,
    caption: &str,
    is_logo: bool,
) -> ApiResult<OrgPhoto> {
    let org = require_org()?;
    let form = web_sys::FormData::new()
        .map_err(|_| ApiError::local("could not build the upload"))?;
    let _ = form.append_with_str("image_url", image_url.trim());
    if !caption.trim().is_empty() {
        let _ = form.append_with_str("caption", caption.trim());
    }
    let _ = form.append_with_str("is_logo", if is_logo { "true" } else { "false" });
    request_form(
        Method::Post,
        &format!("/organizations/{org}/gallery/"),
        &form,
    )
    .await
}

pub async fn delete_gallery_photo(photo_id: i64) -> ApiResult<()> {
    let org = require_org()?;
    request_value(
        Method::Delete,
        &format!("/organizations/{org}/gallery/{photo_id}/"),
        None,
        Auth::Required,
    )
    .await?;
    Ok(())
}

/// Persists a new gallery order. `ids` must be in the order to display.
pub async fn reorder_gallery(ids: &[i64]) -> ApiResult<()> {
    let org = require_org()?;
    let order: Vec<serde_json::Value> = ids
        .iter()
        .enumerate()
        .map(|(i, id)| serde_json::json!({ "id": id, "display_order": i as i32 + 1 }))
        .collect();
    let body = serde_json::json!({ "photos": order });
    request_value(
        Method::Patch,
        &format!("/organizations/{org}/gallery/reorder/"),
        Some(body),
        Auth::Required,
    )
    .await?;
    Ok(())
}

pub async fn list_contacts() -> ApiResult<Vec<OrgContact>> {
    let org = require_org()?;
    request_list(&format!("/organizations/{org}/contacts/"), &Vec::new()).await
}

pub async fn create_contact(contact: &OrgContact) -> ApiResult<OrgContact> {
    let org = require_org()?;
    let body = serde_json::json!({
        "name": contact.name.trim(),
        "title": contact.title.clone().unwrap_or_default(),
        "email": contact.email.clone().unwrap_or_default(),
        "phone_number": contact.phone_number.clone().unwrap_or_default(),
        "is_primary": contact.is_primary,
    });
    request(
        Method::Post,
        &format!("/organizations/{org}/contacts/"),
        Some(body),
        Auth::Required,
    )
    .await
}

pub async fn delete_contact(contact_id: i64) -> ApiResult<()> {
    let org = require_org()?;
    request_value(
        Method::Delete,
        &format!("/organizations/{org}/contacts/{contact_id}/"),
        None,
        Auth::Required,
    )
    .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Dashboard
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct DashHotel {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub city: Option<String>,
    #[serde(default)]
    pub country: Option<String>,
    #[serde(default)]
    pub address: Option<String>,
    #[serde(default)]
    pub star_rating: f32,
    #[serde(default)]
    pub currency: Option<String>,
    #[serde(default)]
    pub logo: Option<String>,
    #[serde(default)]
    pub is_verified: bool,
}

impl DashHotel {
    pub fn location(&self) -> String {
        [
            self.address.as_deref(),
            self.city.as_deref(),
            self.country.as_deref(),
        ]
        .into_iter()
        .flatten()
        .filter(|s| !s.trim().is_empty())
        .collect::<Vec<_>>()
        .join(", ")
    }
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct DashMetrics {
    #[serde(default)]
    pub new_reservations_today: u32,
    #[serde(default)]
    pub check_ins_today: u32,
    #[serde(default)]
    pub rooms_available_today: u32,
    #[serde(default)]
    pub total_rooms: u32,
    #[serde(default)]
    pub occupied_rooms_today: u32,
    #[serde(default)]
    pub occupancy_rate_today: f32,
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct DashToday {
    #[serde(default)]
    pub total_bookings: u32,
    #[serde(default)]
    pub total_revenue: Option<String>,
    #[serde(default)]
    pub total_revenue_formatted: Option<String>,
    #[serde(default)]
    pub check_ins: u32,
    #[serde(default)]
    pub check_outs: u32,
}

/// A row of the dashboard's recent-bookings feed. Flatter than the reservations
/// list: the guest and room are inlined rather than nested.
#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct DashReservation {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub booking_reference: String,
    #[serde(default)]
    pub guest_name: String,
    #[serde(default)]
    pub guest_phone: Option<String>,
    #[serde(default)]
    pub guest_email: Option<String>,
    #[serde(default)]
    pub check_in_date: Option<String>,
    #[serde(default)]
    pub check_out_date: Option<String>,
    #[serde(default)]
    pub number_of_nights: u32,
    #[serde(default)]
    pub guest_count: u32,
    #[serde(default)]
    pub room_number: Option<String>,
    #[serde(default)]
    pub room_type: Option<String>,
    #[serde(default)]
    pub room_name: Option<String>,
    #[serde(default)]
    pub booking_status: String,
    #[serde(default)]
    pub total_amount: Option<String>,
    #[serde(default)]
    pub currency: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct DashSummary {
    #[serde(default)]
    pub date: Option<String>,
    #[serde(default)]
    pub hotel: DashHotel,
    #[serde(default)]
    pub metrics: DashMetrics,
    #[serde(default)]
    pub today_summary: DashToday,
    #[serde(default)]
    pub recent_reservations: Vec<DashReservation>,
}

/// `date` is optional; the API defaults to the property's local today.
pub async fn dashboard_summary(date: Option<&str>) -> ApiResult<DashSummary> {
    let params: Params = vec![("date", date.unwrap_or_default().to_string())];
    request(
        Method::Get,
        &with_query("/dashboard/summary/", &params),
        None,
        Auth::Required,
    )
    .await
}

// ---------------------------------------------------------------------------
// Amenities
// ---------------------------------------------------------------------------

pub const AMENITY_CATEGORIES: &[&str] = &[
    "Essentials",
    "Leisure & Wellness",
    "Dining",
    "Services",
    "Rooms & Accessibility",
    "Business",
    "Custom",
];

#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct Amenity {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub icon_url: Option<String>,
    #[serde(default)]
    pub is_custom: bool,
}

impl Amenity {
    pub fn category_str(&self) -> &str {
        self.category.as_deref().filter(|s| !s.is_empty()).unwrap_or("Custom")
    }
}

/// Amenity catalogue for the current hotel. Returns `[]` for an unauthenticated
/// caller, so this is dashboard-only.
pub async fn list_amenities() -> ApiResult<Vec<Amenity>> {
    request_list("/amenities/", &Vec::new()).await
}

/// Creates a hotel-specific amenity. The endpoint is multipart because it also
/// accepts an icon upload.
pub async fn create_amenity(name: &str, category: &str) -> ApiResult<Amenity> {
    let form = web_sys::FormData::new()
        .map_err(|_| ApiError::local("could not build the request"))?;
    let _ = form.append_with_str("name", name.trim());
    let _ = form.append_with_str("category", category);
    request_form(Method::Post, "/amenities/create/", &form).await
}

// ---------------------------------------------------------------------------
// Rooms
// ---------------------------------------------------------------------------

pub const ROOM_TYPES: &[&str] = &[
    "Standard Room",
    "Deluxe Room",
    "Twin Room",
    "Family Room",
    "Executive Suite",
    "Single Room",
];
pub const ROOM_STATUSES: &[&str] = &["Available", "Occupied", "Reserved", "Maintenance"];
pub const BED_TYPES: &[&str] = &[
    "Single Bed",
    "Double Bed",
    "Queen Bed",
    "King Bed",
    "Super King Bed",
    "California King Bed",
    "Sofa Bed",
    "Bunk Bed",
    "Extra Bed",
    "Crib / Baby Cot",
    "Other",
];
/// The API's `EventType` choices. Note there is no "Maintenance" value — a
/// room out of service is `Blocked` with a note.
pub const EVENT_TYPES: &[&str] = &["Blocked", "Unavailable", "Upcoming"];

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct RoomRow {
    pub id: i64,
    #[serde(default)]
    pub uuid: String,
    #[serde(default)]
    pub room_number: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub room_type: Option<String>,
    #[serde(default)]
    pub floor: Option<i32>,
    #[serde(default)]
    pub guest_capacity: u32,
    #[serde(default)]
    pub price_per_night: Option<String>,
    #[serde(default)]
    pub price_after_discount: Option<String>,
    #[serde(default)]
    pub discount_percent_per_night: Option<i32>,
    #[serde(default)]
    pub breakfast_included: bool,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub amenities_preview: Vec<String>,
    #[serde(default)]
    pub photos_count: u32,
}

impl RoomRow {
    pub fn status_str(&self) -> &str {
        self.status.as_deref().unwrap_or("Available")
    }
    /// Nightly rate actually charged, rounded, with thousands separators.
    pub fn price_display(&self) -> String {
        money_round(
            self.price_after_discount
                .as_deref()
                .or(self.price_per_night.as_deref()),
        )
    }
    pub fn full_price_display(&self) -> String {
        money_round(self.price_per_night.as_deref())
    }
    pub fn has_discount(&self) -> bool {
        self.discount_percent_per_night.unwrap_or(0) > 0
    }
    pub fn type_str(&self) -> &str {
        self.room_type.as_deref().unwrap_or("—")
    }
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct RoomStats {
    #[serde(default)]
    pub total_rooms: u32,
    #[serde(default)]
    pub available: u32,
    #[serde(default)]
    pub occupied: u32,
    #[serde(default)]
    pub reserved: u32,
    #[serde(default)]
    pub maintenance: u32,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Bed {
    #[serde(default, skip_serializing)]
    pub id: i64,
    #[serde(default)]
    pub bed_type: String,
    #[serde(default)]
    pub number_of_beds: u32,
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct RoomPhoto {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub image_url: String,
    #[serde(default)]
    pub caption: Option<String>,
    #[serde(default)]
    pub display_order: i32,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct RoomPolicy {
    #[serde(default, skip_serializing)]
    pub id: Option<i64>,
    #[serde(default)]
    pub checkin_time: Option<String>,
    #[serde(default)]
    pub checkout_time: Option<String>,
    #[serde(default)]
    pub children_allowed: Option<bool>,
    #[serde(default)]
    pub children_age: Option<i32>,
    #[serde(default)]
    pub extrabed_available: Option<bool>,
    #[serde(default)]
    pub pet_allowed: Option<bool>,
    #[serde(default)]
    pub smoking_allowed: Option<bool>,
    #[serde(default)]
    pub non_smoking_property: Option<bool>,
    #[serde(default)]
    pub government_id_required: Option<bool>,
    #[serde(default)]
    pub minimum_checkin_age: Option<i32>,
    #[serde(default)]
    pub parties_or_event_allowed: Option<bool>,
    #[serde(default)]
    pub public_note: Option<String>,
}

/// A block on a room's calendar: maintenance, an owner stay, a held range.
#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct SpecialEvent {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub room_id: i64,
    #[serde(default)]
    pub room_number: Option<String>,
    #[serde(default)]
    pub event_type: Option<String>,
    #[serde(default)]
    pub event_start_date: Option<String>,
    #[serde(default)]
    pub event_finished_date: Option<String>,
    #[serde(default)]
    pub is_active: bool,
    #[serde(default)]
    pub note: Option<String>,
}

/// `GET /rooms/{id}/`. Note the payload carries both a null `amenities` and a
/// populated `room_amenities`; only the latter is real.
#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct RoomDetail {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub uuid: String,
    #[serde(default)]
    pub organization_id: i64,
    #[serde(default)]
    pub room_number: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub room_type: Option<String>,
    #[serde(default)]
    pub floor: Option<i32>,
    #[serde(default)]
    pub guest_capacity: u32,
    #[serde(default)]
    pub price_per_night: Option<String>,
    #[serde(default)]
    pub discount_percent_per_night: Option<i32>,
    #[serde(default)]
    pub price_after_discount: Option<String>,
    #[serde(default)]
    pub breakfast_included: bool,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub beds: Vec<Bed>,
    #[serde(default)]
    pub room_amenities: Vec<Amenity>,
    #[serde(default)]
    pub photos: Vec<RoomPhoto>,
    #[serde(default)]
    pub special_events: Vec<SpecialEvent>,
    #[serde(default)]
    pub policy: Option<RoomPolicy>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

impl RoomDetail {
    pub fn status_str(&self) -> &str {
        self.status.as_deref().unwrap_or("Available")
    }
    pub fn type_str(&self) -> &str {
        self.room_type.as_deref().unwrap_or("—")
    }
    pub fn price_display(&self) -> String {
        money_round(
            self.price_after_discount
                .as_deref()
                .or(self.price_per_night.as_deref()),
        )
    }
    pub fn bed_summary(&self) -> String {
        if self.beds.is_empty() {
            return "Not configured".into();
        }
        self.beds
            .iter()
            .map(|b| match b.number_of_beds {
                0 | 1 => format!("1 {}", b.bed_type),
                n => format!("{n} {}s", b.bed_type),
            })
            .collect::<Vec<_>>()
            .join(", ")
    }
    pub fn amenity_ids(&self) -> Vec<i64> {
        self.room_amenities.iter().map(|a| a.id).collect()
    }
    pub fn primary_image(&self) -> Option<String> {
        self.photos
            .iter()
            .min_by_key(|p| p.display_order)
            .map(|p| p.image_url.clone())
            .filter(|u| !u.is_empty())
    }
}

pub async fn list_rooms(status: Option<&str>, search: &str) -> ApiResult<Page<RoomRow>> {
    let mut params: Params = vec![("page_size", "100".into())];
    if let Some(s) = status.filter(|s| *s != "All") {
        params.push(("status", s.to_string()));
    }
    params.push(("search", search.trim().to_string()));
    request_page("/rooms/", &params).await
}

/// All rooms, ignoring filters — for pickers that need the whole inventory.
pub async fn all_rooms() -> ApiResult<Vec<RoomRow>> {
    Ok(request_page::<RoomRow>(
        "/rooms/",
        &vec![("page_size", "100".into()), ("ordering", "room_number".into())],
    )
    .await?
    .items)
}

pub async fn room_stats() -> ApiResult<RoomStats> {
    request(Method::Get, "/rooms/statistics/", None, Auth::Required).await
}

pub async fn get_room(id: i64) -> ApiResult<RoomDetail> {
    request(
        Method::Get,
        &format!("/rooms/{id}/"),
        None,
        Auth::Required,
    )
    .await
}

/// Looks a room up by the number shown in the URL.
///
/// The routes are keyed on the human room number (`/rooms/101`) rather than the
/// database id, so the id has to be resolved through a search first.
pub async fn get_room_by_number(number: &str) -> ApiResult<RoomDetail> {
    let page = request_page::<RoomRow>(
        "/rooms/",
        &vec![("search", number.to_string()), ("page_size", "100".into())],
    )
    .await?;
    let hit = page
        .items
        .iter()
        .find(|r| r.room_number == number)
        .or_else(|| page.items.first())
        .ok_or_else(|| ApiError {
            status: 404,
            code: "NOT_FOUND".into(),
            message: format!("No room numbered {number} in this property."),
            fields: Vec::new(),
        })?;
    get_room(hit.id).await
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct NewRoom {
    pub room_number: String,
    pub name: String,
    pub room_type: String,
    pub floor: i32,
    pub guest_capacity: u32,
    pub price_per_night: String,
    pub discount_percent_per_night: i32,
    pub breakfast_included: bool,
    pub status: String,
    pub amenity_ids: Vec<i64>,
    pub beds: Vec<Bed>,
}

pub async fn create_room(room: &NewRoom) -> ApiResult<RoomRow> {
    let body =
        serde_json::to_value(room).map_err(|e| ApiError::local(format!("encode room: {e}")))?;
    request(Method::Post, "/rooms/", Some(body), Auth::Required).await
}

pub async fn update_room(id: i64, room: &NewRoom) -> ApiResult<RoomDetail> {
    let body =
        serde_json::to_value(room).map_err(|e| ApiError::local(format!("encode room: {e}")))?;
    request(
        Method::Patch,
        &format!("/rooms/{id}/"),
        Some(body),
        Auth::Required,
    )
    .await
}

/// Patches a single field, for inline edits (rate changes, discounts).
pub async fn patch_room(id: i64, body: serde_json::Value) -> ApiResult<RoomDetail> {
    request(
        Method::Patch,
        &format!("/rooms/{id}/"),
        Some(body),
        Auth::Required,
    )
    .await
}

pub async fn delete_room(id: i64) -> ApiResult<()> {
    request_value(
        Method::Delete,
        &format!("/rooms/{id}/"),
        None,
        Auth::Required,
    )
    .await?;
    Ok(())
}

pub async fn set_room_status(id: i64, status: &str) -> ApiResult<()> {
    let body = serde_json::json!({ "status": status });
    request_value(
        Method::Patch,
        &format!("/rooms/{id}/status/"),
        Some(body),
        Auth::Required,
    )
    .await?;
    Ok(())
}

pub async fn sync_room_amenities(id: i64, amenity_ids: &[i64]) -> ApiResult<()> {
    let body = serde_json::json!({ "amenity_ids": amenity_ids });
    request_value(
        Method::Post,
        &format!("/rooms/{id}/amenities/sync/"),
        Some(body),
        Auth::Required,
    )
    .await?;
    Ok(())
}

pub async fn sync_room_beds(id: i64, beds: &[Bed]) -> ApiResult<()> {
    let body = serde_json::json!({ "beds": beds });
    request_value(
        Method::Post,
        &format!("/rooms/{id}/beds/"),
        Some(body),
        Auth::Required,
    )
    .await?;
    Ok(())
}

pub async fn update_room_policy(id: i64, policy: &RoomPolicy) -> ApiResult<RoomPolicy> {
    let body = serde_json::to_value(policy)
        .map_err(|e| ApiError::local(format!("encode policy: {e}")))?;
    request(
        Method::Patch,
        &format!("/rooms/{id}/policies/"),
        Some(body),
        Auth::Required,
    )
    .await
}

pub async fn add_room_photo(id: i64, image_url: &str, caption: &str) -> ApiResult<RoomPhoto> {
    let form = web_sys::FormData::new()
        .map_err(|_| ApiError::local("could not build the upload"))?;
    let _ = form.append_with_str("image_url", image_url.trim());
    if !caption.trim().is_empty() {
        let _ = form.append_with_str("caption", caption.trim());
    }
    request_form(Method::Post, &format!("/rooms/{id}/photos/"), &form).await
}

pub async fn delete_room_photo(room_id: i64, photo_id: i64) -> ApiResult<()> {
    request_value(
        Method::Delete,
        &format!("/rooms/{room_id}/photos/{photo_id}/"),
        None,
        Auth::Required,
    )
    .await?;
    Ok(())
}

pub async fn list_special_events(room_id: i64) -> ApiResult<Vec<SpecialEvent>> {
    request_list(&format!("/rooms/{room_id}/special-events/"), &Vec::new()).await
}

pub async fn create_special_event(
    room_id: i64,
    event_type: &str,
    start: &str,
    end: &str,
    note: &str,
) -> ApiResult<SpecialEvent> {
    let body = serde_json::json!({
        "event_type": event_type,
        "event_start_date": start,
        "event_finished_date": end,
        "note": note,
    });
    request(
        Method::Post,
        &format!("/rooms/{room_id}/special-events/"),
        Some(body),
        Auth::Required,
    )
    .await
}

pub async fn delete_special_event(event_id: i64) -> ApiResult<()> {
    request_value(
        Method::Delete,
        &format!("/rooms/special-events/{event_id}/"),
        None,
        Auth::Required,
    )
    .await?;
    Ok(())
}

/// Ends a block now rather than deleting the record, keeping the history.
pub async fn end_special_event(event_id: i64) -> ApiResult<()> {
    request_value(
        Method::Post,
        &format!("/rooms/special-events/{event_id}/end/"),
        None,
        Auth::Required,
    )
    .await?;
    Ok(())
}

/// Blocks a date range across one or more rooms. `end_date` is inclusive.
pub async fn block_dates(
    room_ids: &[i64],
    event_type: &str,
    start: &str,
    end: &str,
    note: &str,
) -> ApiResult<()> {
    let mut body = serde_json::json!({
        "event_type": event_type,
        "start_date": start,
        "end_date": end,
        "note": note,
    });
    if let Some(map) = body.as_object_mut() {
        if room_ids.len() == 1 {
            map.insert("room_id".into(), serde_json::json!(room_ids[0]));
        } else {
            map.insert("room_ids".into(), serde_json::json!(room_ids));
        }
    }
    request_value(Method::Post, "/rooms/block-dates/", Some(body), Auth::Required).await?;
    Ok(())
}

// ---- Bulk upload ----------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct BulkPreviewRow {
    #[serde(default)]
    pub row_number: u32,
    #[serde(default)]
    pub room_number: Option<String>,
    #[serde(default)]
    pub room_type: Option<String>,
    #[serde(default)]
    pub floor: Option<i32>,
    #[serde(default)]
    pub guest_capacity: Option<u32>,
    #[serde(default)]
    pub price_per_night: Option<f64>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub amenities: Vec<String>,
    #[serde(default)]
    pub validation_status: String,
    #[serde(default)]
    pub message: Option<String>,
}

impl BulkPreviewRow {
    pub fn is_error(&self) -> bool {
        self.validation_status.eq_ignore_ascii_case("error")
    }
    pub fn is_warning(&self) -> bool {
        self.validation_status.eq_ignore_ascii_case("warning")
    }
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct BulkValidation {
    #[serde(default)]
    pub rows_detected: u32,
    #[serde(default)]
    pub valid: u32,
    #[serde(default)]
    pub warnings: u32,
    #[serde(default)]
    pub errors: u32,
    #[serde(default)]
    pub preview_rows: Vec<BulkPreviewRow>,
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct BulkImportResult {
    #[serde(default)]
    pub total_rows: u32,
    #[serde(default)]
    pub imported_count: u32,
    #[serde(default)]
    pub updated_count: u32,
    #[serde(default)]
    pub skipped_count: u32,
    #[serde(default)]
    pub failed_count: u32,
    #[serde(default)]
    pub errors: Vec<String>,
}

/// Import switches, shared by validate and import so the preview matches what
/// the commit will actually do.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BulkOptions {
    pub skip_duplicates: bool,
    pub update_existing: bool,
    pub default_available: bool,
    pub validate_amenities: bool,
}

impl Default for BulkOptions {
    fn default() -> Self {
        Self {
            skip_duplicates: true,
            update_existing: false,
            default_available: true,
            validate_amenities: true,
        }
    }
}

fn bulk_form(file: &web_sys::File, opts: BulkOptions) -> ApiResult<web_sys::FormData> {
    let form = web_sys::FormData::new()
        .map_err(|_| ApiError::local("could not build the upload"))?;
    form.append_with_blob_and_filename("file", file, &file.name())
        .map_err(|_| ApiError::local("could not attach the spreadsheet"))?;
    let b = |v: bool| if v { "true" } else { "false" };
    let _ = form.append_with_str("skip_duplicates", b(opts.skip_duplicates));
    let _ = form.append_with_str("update_existing", b(opts.update_existing));
    let _ = form.append_with_str("default_available", b(opts.default_available));
    let _ = form.append_with_str("validate_amenities", b(opts.validate_amenities));
    Ok(form)
}

/// Dry run: parses the spreadsheet and reports per-row diagnostics.
pub async fn validate_bulk_rooms(
    file: &web_sys::File,
    opts: BulkOptions,
) -> ApiResult<BulkValidation> {
    let form = bulk_form(file, opts)?;
    request_form(Method::Post, "/rooms/bulk-upload/validate/", &form).await
}

/// Commits the spreadsheet in a single transaction.
pub async fn import_bulk_rooms(
    file: &web_sys::File,
    opts: BulkOptions,
) -> ApiResult<BulkImportResult> {
    let form = bulk_form(file, opts)?;
    request_form(Method::Post, "/rooms/bulk-upload/import/", &form).await
}

pub async fn download_bulk_template() -> ApiResult<()> {
    download(
        "/rooms/bulk-upload/template/?format=csv",
        "rooms_upload_template.csv",
    )
    .await
}

// ---- Availability calendar ------------------------------------------------

/// One blocking entry on a room's row.
///
/// The API repeats an identical entry once per blocked night and never says
/// *which* night, so the date span has to be read off `check_in_date` /
/// `check_out_date` instead. See [`CalendarRoom::blocked_days`].
#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct CalendarDay {
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub status_code: String,
    #[serde(default)]
    pub is_available: bool,
    #[serde(default)]
    pub check_in_date: Option<String>,
    #[serde(default)]
    pub check_out_date: Option<String>,
    #[serde(default)]
    pub guest_name: Option<String>,
    #[serde(default)]
    pub guest_phone: Option<String>,
    #[serde(default)]
    pub reservation_id: Option<i64>,
    #[serde(default)]
    pub event_id: Option<i64>,
    #[serde(default)]
    pub event_type: Option<String>,
    #[serde(default)]
    pub note: Option<String>,
    #[serde(default)]
    pub block_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct CalendarRoom {
    #[serde(default)]
    pub room_id: i64,
    #[serde(default)]
    pub room_number: String,
    #[serde(default)]
    pub room_name: String,
    #[serde(default)]
    pub room_type: Option<String>,
    #[serde(default)]
    pub floor: Option<i32>,
    #[serde(default)]
    pub guest_capacity: u32,
    #[serde(default)]
    pub price_per_night: Option<f64>,
    #[serde(default)]
    pub price_after_discount: Option<f64>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub days: Vec<CalendarDay>,
}

impl CalendarRoom {
    /// Expands the sparse, undated `days` list into `(day-of-month, entry)`
    /// pairs for the requested month.
    ///
    /// Entries are deduplicated by reservation or event id — the API sends one
    /// copy per night with no date attached, so the span comes from the entry's
    /// own dates and the duplicates are redundant. Check-out day is left free,
    /// matching how the backend counts availability.
    pub fn blocked_days(&self, year: i32, month: u32) -> Vec<(u32, CalendarDay)> {
        let mut seen: Vec<(Option<i64>, Option<i64>, String)> = Vec::new();
        let mut out: Vec<(u32, CalendarDay)> = Vec::new();

        for d in &self.days {
            let key = (
                d.reservation_id,
                d.event_id,
                d.check_in_date.clone().unwrap_or_default(),
            );
            if seen.contains(&key) {
                continue;
            }
            seen.push(key);

            let Some(start) = d.check_in_date.as_deref().and_then(parse_ymd) else {
                continue;
            };
            let end = d
                .check_out_date
                .as_deref()
                .and_then(parse_ymd)
                .unwrap_or(start);

            // A block ("Blocked"/"Unavailable") covers its end date; a stay
            // frees the room on checkout morning.
            let inclusive_end = d.reservation_id.is_none();
            for day in 1..=days_in_month(year, month) {
                let cell = (year, month, day);
                let after_start = cell >= start;
                let before_end = if inclusive_end { cell <= end } else { cell < end };
                if after_start && before_end {
                    out.push((day, d.clone()));
                }
            }
        }
        out
    }
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct CalendarMatrix {
    #[serde(default)]
    pub year: i32,
    #[serde(default)]
    pub month: u32,
    #[serde(default)]
    pub month_name: String,
    #[serde(default)]
    pub total_days: u32,
    #[serde(default)]
    pub total_rooms: u32,
    #[serde(default)]
    pub total_slots: u32,
    #[serde(default)]
    pub occupancy_rate: f32,
    #[serde(default)]
    pub available_slots: u32,
    #[serde(default)]
    pub reserved_slots: u32,
    #[serde(default)]
    pub blocked_slots: u32,
    #[serde(default)]
    pub unavailable_slots: u32,
    #[serde(default)]
    pub matrix: Vec<CalendarRoom>,
}

/// `"2026-08-28"` -> `(2026, 8, 28)`.
fn parse_ymd(s: &str) -> Option<(i32, u32, u32)> {
    let date = s.split('T').next()?;
    let mut it = date.split('-');
    Some((
        it.next()?.parse().ok()?,
        it.next()?.parse().ok()?,
        it.next()?.parse().ok()?,
    ))
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 => 29,
        2 => 28,
        _ => 30,
    }
}

pub async fn calendar_matrix(
    year: i32,
    month: u32,
    room_type: &str,
    search: &str,
) -> ApiResult<CalendarMatrix> {
    let params: Params = vec![
        ("year", year.to_string()),
        ("month", month.to_string()),
        ("page_size", "100".into()),
        (
            "room_type",
            if room_type == "All" { String::new() } else { room_type.to_string() },
        ),
        ("search", search.trim().to_string()),
    ];
    request(
        Method::Get,
        &with_query("/rooms/calendar-matrix/", &params),
        None,
        Auth::Required,
    )
    .await
}

// ---------------------------------------------------------------------------
// Reservations
// ---------------------------------------------------------------------------

pub const BOOKING_STATUSES: &[&str] = &[
    "New",
    "Pending",
    "Confirmed",
    "In-house",
    "Completed",
    "Cancelled",
    "Rejected",
    "No show",
];
pub const PAYMENT_METHODS: &[&str] = &[
    "Cash",
    "Card",
    "Bank transfer",
    "Mobile payment",
    "Online payment",
    "Other",
];
pub const PAYMENT_STATUSES: &[&str] = &["Unpaid", "Partially paid", "Paid", "Refunded"];
pub const ROOM_CONDITIONS: &[&str] = &[
    "Good",
    "Needs cleaning",
    "Damaged",
    "Maintenance required",
];

/// The room, as embedded in a reservation.
#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct ResRoom {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub room_number: String,
    #[serde(default)]
    pub floor: Option<i32>,
    #[serde(default)]
    pub room_type: Option<String>,
    #[serde(default)]
    pub price_per_night: Option<String>,
    #[serde(default)]
    pub price_after_discount: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
}

/// The guest, as embedded in a reservation. Note the field is `phone_number`
/// here but `phone`/`display_phone` on the guests endpoints.
#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct ResGuest {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub phone_number: Option<String>,
    #[serde(default)]
    pub id_passport_no: Option<String>,
    #[serde(default)]
    pub nationality: Option<String>,
    #[serde(default)]
    pub is_vip: bool,
}

/// The financial settlement written at checkout. This is the only place the API
/// records money taken, so it is also the payments ledger.
#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct Completion {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub actual_check_in_at: Option<String>,
    #[serde(default)]
    pub actual_check_out_at: Option<String>,
    #[serde(default)]
    pub number_of_nights: u32,
    #[serde(default)]
    pub room_charge: Option<String>,
    #[serde(default)]
    pub extra_bed_charge: Option<String>,
    #[serde(default)]
    pub pet_charge: Option<String>,
    #[serde(default)]
    pub additional_charge: Option<String>,
    #[serde(default)]
    pub additional_charge_note: Option<String>,
    #[serde(default)]
    pub damage_charge: Option<String>,
    #[serde(default)]
    pub discount_amount: Option<String>,
    #[serde(default)]
    pub tax_amount: Option<String>,
    #[serde(default)]
    pub total_amount: Option<String>,
    #[serde(default)]
    pub amount_paid: Option<String>,
    #[serde(default)]
    pub outstanding_amount: Option<String>,
    #[serde(default)]
    pub payment_status: Option<String>,
    #[serde(default)]
    pub payment_method: Option<String>,
    #[serde(default)]
    pub payment_reference: Option<String>,
    #[serde(default)]
    pub room_condition: Option<String>,
    #[serde(default)]
    pub completion_note: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
}

impl Completion {
    pub fn total(&self) -> f64 {
        num(self.total_amount.as_deref())
    }
    pub fn paid(&self) -> f64 {
        num(self.amount_paid.as_deref())
    }
    pub fn outstanding(&self) -> f64 {
        num(self.outstanding_amount.as_deref())
    }
    pub fn status_str(&self) -> &str {
        self.payment_status.as_deref().unwrap_or("Unpaid")
    }
    pub fn method_str(&self) -> &str {
        self.payment_method
            .as_deref()
            .filter(|s| !s.is_empty())
            .unwrap_or("—")
    }
}

fn num(raw: Option<&str>) -> f64 {
    raw.and_then(|s| s.trim().parse::<f64>().ok()).unwrap_or(0.0)
}

/// An entry of the reservation timeline.
#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct Activity {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub actor_name: Option<String>,
    #[serde(default, rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub note: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
}

/// A reservation. The list and detail endpoints return the same shape; only
/// `completions` and `activities` are detail-only.
#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct Reservation {
    pub id: i64,
    #[serde(default)]
    pub uuid: String,
    #[serde(default)]
    pub booking_reference: String,
    #[serde(default)]
    pub reference_display: String,
    #[serde(default)]
    pub organization: i64,
    #[serde(default)]
    pub organization_name: String,
    #[serde(default)]
    pub room: ResRoom,
    #[serde(default)]
    pub guest: ResGuest,
    #[serde(default)]
    pub check_in_date: Option<String>,
    #[serde(default)]
    pub check_out_date: Option<String>,
    #[serde(default)]
    pub number_of_nights: u32,
    #[serde(default)]
    pub guest_count: u32,
    #[serde(default)]
    pub pet_presence: bool,
    #[serde(default)]
    pub baby_presence: bool,
    #[serde(default)]
    pub extra_bed: u32,
    #[serde(default)]
    pub extra_bed_per_night_price: Option<String>,
    #[serde(default)]
    pub booking_status: String,
    #[serde(default)]
    pub cancellation_reason: Option<String>,
    #[serde(default)]
    pub guest_note: Option<String>,
    #[serde(default)]
    pub organization_note: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub completions: Vec<Completion>,
    #[serde(default)]
    pub activities: Vec<Activity>,
}

impl Reservation {
    pub fn reference(&self) -> String {
        if !self.reference_display.is_empty() {
            self.reference_display.clone()
        } else {
            self.booking_reference.clone()
        }
    }
    pub fn guest_name(&self) -> String {
        if self.guest.name.trim().is_empty() {
            "—".into()
        } else {
            self.guest.name.clone()
        }
    }
    pub fn guest_phone(&self) -> String {
        self.guest.phone_number.clone().unwrap_or_default()
    }
    pub fn guest_email(&self) -> String {
        self.guest.email.clone().unwrap_or_default()
    }
    pub fn guest_initials(&self) -> String {
        self.guest
            .name
            .split_whitespace()
            .filter_map(|w| w.chars().next())
            .take(2)
            .collect::<String>()
            .to_uppercase()
    }
    pub fn room_label(&self) -> String {
        if self.room.room_number.is_empty() {
            "—".into()
        } else {
            self.room.room_number.clone()
        }
    }
    pub fn room_type(&self) -> &str {
        self.room.room_type.as_deref().unwrap_or("—")
    }
    pub fn status_str(&self) -> &str {
        if self.booking_status.is_empty() {
            "New"
        } else {
            &self.booking_status
        }
    }
    pub fn stay_dates(&self) -> String {
        format!(
            "{} → {}",
            pretty_date(self.check_in_date.as_deref()),
            pretty_date(self.check_out_date.as_deref())
        )
    }
    /// The settlement, when the stay has been checked out.
    pub fn completion(&self) -> Option<&Completion> {
        self.completions.first()
    }
    /// Amount actually settled, or the estimated stay value when the guest has
    /// not checked out yet.
    pub fn amount(&self) -> f64 {
        match self.completion() {
            Some(c) => c.total(),
            None => {
                let rate = num(self
                    .room
                    .price_after_discount
                    .as_deref()
                    .or(self.room.price_per_night.as_deref()));
                rate * self.number_of_nights as f64
            }
        }
    }
    pub fn is_settled(&self) -> bool {
        self.completion().is_some()
    }
    /// Which lifecycle actions the current status allows.
    pub fn can_confirm(&self) -> bool {
        matches!(self.status_str(), "New" | "Pending")
    }
    pub fn can_reject(&self) -> bool {
        matches!(self.status_str(), "New" | "Pending")
    }
    pub fn can_check_in(&self) -> bool {
        self.status_str() == "Confirmed"
    }
    pub fn can_check_out(&self) -> bool {
        self.status_str() == "In-house"
    }
    pub fn can_cancel(&self) -> bool {
        matches!(self.status_str(), "New" | "Pending" | "Confirmed")
    }
}

/// Filters for `GET /reservations/`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ReservationQuery {
    /// `all | new | confirmed | in-house | completed | cancelled`
    pub status_tab: String,
    pub search: String,
    pub room_id: Option<i64>,
    pub start_date: String,
    pub end_date: String,
    pub ordering: String,
    pub page: u32,
    pub page_size: u32,
}

/// Maps a UI tab label to the API's `status_tab` value.
pub fn reservation_tab_param(label: &str) -> String {
    match label {
        "All" | "" => String::new(),
        "In-house" | "In House" => "in-house".into(),
        other => other.to_lowercase(),
    }
}

pub async fn list_reservations(q: &ReservationQuery) -> ApiResult<Page<Reservation>> {
    let mut params: Params = vec![
        (
            "page_size",
            if q.page_size == 0 { "100".into() } else { q.page_size.to_string() },
        ),
        ("status_tab", q.status_tab.clone()),
        ("search", q.search.trim().to_string()),
        ("start_date", q.start_date.clone()),
        ("end_date", q.end_date.clone()),
        ("ordering", q.ordering.clone()),
    ];
    if q.page > 1 {
        params.push(("page", q.page.to_string()));
    }
    if let Some(id) = q.room_id {
        params.push(("room_id", id.to_string()));
    }
    request_page("/reservations/", &params).await
}

pub async fn get_reservation(id: i64) -> ApiResult<Reservation> {
    request(
        Method::Get,
        &format!("/reservations/{id}/"),
        None,
        Auth::Required,
    )
    .await
}

/// Finds a reservation by its human booking reference (`HA-260828513`).
///
/// The guest routes are keyed on the reference, so the numeric id has to be
/// looked up through search first.
pub async fn get_reservation_by_reference(reference: &str) -> ApiResult<Reservation> {
    let page = list_reservations(&ReservationQuery {
        search: reference.to_string(),
        page_size: 100,
        ..Default::default()
    })
    .await?;
    let hit = page
        .items
        .iter()
        .find(|r| r.booking_reference == reference || r.reference_display == reference)
        .or_else(|| page.items.first())
        .ok_or_else(|| ApiError {
            status: 404,
            code: "NOT_FOUND".into(),
            message: format!("No reservation matching {reference}."),
            fields: Vec::new(),
        })?;
    get_reservation(hit.id).await
}

pub async fn reservation_activities(id: i64) -> ApiResult<Vec<Activity>> {
    request_list(&format!("/reservations/{id}/activities/"), &Vec::new()).await
}

/// Fields the front-desk booking form collects.
#[derive(Debug, Clone, Default, Serialize)]
pub struct NewReservation {
    pub organization_id: i64,
    pub room_id: i64,
    pub check_in_date: String,
    pub check_out_date: String,
    pub guest_count: u32,
    pub extra_bed: u32,
    pub pet_presence: bool,
    pub baby_presence: bool,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub guest_first_name: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub guest_last_name: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub guest_email: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub guest_phone: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub guest_note: String,
}

/// Creates a booking from the dashboard.
///
/// Note the API attaches the *calling* account as the guest when the request is
/// authenticated, so the `guest_*` name fields are advisory. Walk-ins entered
/// by staff therefore land under the staff member's guest record; the guest's
/// own details still reach the reservation note.
pub async fn create_reservation(res: &NewReservation) -> ApiResult<Reservation> {
    let body = serde_json::to_value(res)
        .map_err(|e| ApiError::local(format!("could not encode reservation: {e}")))?;
    request(Method::Post, "/reservations/", Some(body), Auth::Required).await
}

pub async fn confirm_reservation(id: i64, note: &str) -> ApiResult<Reservation> {
    let body = serde_json::json!({ "note": note });
    request(
        Method::Post,
        &format!("/reservations/{id}/confirm/"),
        Some(body),
        Auth::Required,
    )
    .await
}

pub async fn reject_reservation(id: i64, reason: &str, note: &str) -> ApiResult<Reservation> {
    let body = serde_json::json!({ "reason": reason, "note": note });
    request(
        Method::Post,
        &format!("/reservations/{id}/reject/"),
        Some(body),
        Auth::Required,
    )
    .await
}

pub async fn cancel_reservation(id: i64, reason: &str) -> ApiResult<Reservation> {
    let body = serde_json::json!({ "reason": reason });
    request(
        Method::Post,
        &format!("/reservations/{id}/cancel/"),
        Some(body),
        Auth::Required,
    )
    .await
}

pub async fn check_in(id: i64, id_passport_no: &str, note: &str) -> ApiResult<Reservation> {
    let body = serde_json::json!({ "id_passport_no": id_passport_no, "note": note });
    request(
        Method::Post,
        &format!("/reservations/{id}/check-in/"),
        Some(body),
        Auth::Required,
    )
    .await
}

/// The checkout form: charges, settlement and room condition.
#[derive(Debug, Clone, Default, Serialize)]
pub struct CheckoutPayload {
    pub damage_charge: String,
    pub additional_charge: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub additional_charge_note: String,
    pub discount_amount: String,
    pub tax_rate_percent: String,
    pub amount_paid: String,
    pub payment_status: String,
    pub payment_method: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub payment_reference: String,
    pub room_condition: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub completion_note: String,
}

pub async fn check_out(id: i64, payload: &CheckoutPayload) -> ApiResult<Reservation> {
    let body = serde_json::to_value(payload)
        .map_err(|e| ApiError::local(format!("could not encode checkout: {e}")))?;
    request(
        Method::Post,
        &format!("/reservations/{id}/checkout/"),
        Some(body),
        Auth::Required,
    )
    .await
}

pub async fn update_reservation_notes(
    id: i64,
    guest_note: Option<&str>,
    organization_note: Option<&str>,
) -> ApiResult<Reservation> {
    let mut body = serde_json::Map::new();
    if let Some(n) = guest_note {
        body.insert("guest_note".into(), serde_json::json!(n));
    }
    if let Some(n) = organization_note {
        body.insert("organization_note".into(), serde_json::json!(n));
    }
    request(
        Method::Patch,
        &format!("/reservations/{id}/notes/"),
        Some(serde_json::Value::Object(body)),
        Auth::Required,
    )
    .await
}

// ---------------------------------------------------------------------------
// Payments — derived from reservation settlements
// ---------------------------------------------------------------------------

/// One row of the payments ledger.
///
/// The API has no `/payments/` resource: money is recorded on the reservation's
/// `completions` entry, written at checkout. A transaction is therefore a
/// (reservation, completion) pair, and stays that have not checked out yet show
/// as outstanding rather than as transactions.
#[derive(Debug, Clone, PartialEq)]
pub struct Transaction {
    pub reservation_id: i64,
    pub reference: String,
    pub guest_name: String,
    pub room_number: String,
    pub room_type: String,
    pub nights: u32,
    pub settled_at: Option<String>,
    pub total: f64,
    pub paid: f64,
    pub outstanding: f64,
    pub status: String,
    pub method: String,
    pub payment_reference: String,
    pub extras: f64,
    pub tax: f64,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct PaymentsLedger {
    pub transactions: Vec<Transaction>,
    /// Confirmed and in-house stays with no settlement written yet.
    pub pending: Vec<Reservation>,
    pub total_collected: f64,
    pub total_outstanding: f64,
    pub expected_from_pending: f64,
}

/// Builds the ledger from every reservation of the active hotel.
pub async fn payments_ledger(search: &str) -> ApiResult<PaymentsLedger> {
    let page = list_reservations(&ReservationQuery {
        search: search.trim().to_string(),
        page_size: 100,
        ordering: "-created_at".into(),
        ..Default::default()
    })
    .await?;

    let mut ledger = PaymentsLedger::default();

    // The list endpoint omits `completions`, so each settled stay is re-read.
    // Only Completed reservations can carry one, which keeps this bounded.
    for row in page.items {
        if row.status_str() == "Completed" {
            let detail = get_reservation(row.id).await.unwrap_or(row.clone());
            match detail.completion() {
                Some(c) => {
                    ledger.total_collected += c.paid();
                    ledger.total_outstanding += c.outstanding();
                    ledger.transactions.push(Transaction {
                        reservation_id: detail.id,
                        reference: detail.reference(),
                        guest_name: detail.guest_name(),
                        room_number: detail.room_label(),
                        room_type: detail.room_type().to_string(),
                        nights: c.number_of_nights.max(detail.number_of_nights),
                        settled_at: c.actual_check_out_at.clone().or(c.created_at.clone()),
                        total: c.total(),
                        paid: c.paid(),
                        outstanding: c.outstanding(),
                        status: c.status_str().to_string(),
                        method: c.method_str().to_string(),
                        payment_reference: c.payment_reference.clone().unwrap_or_default(),
                        extras: num(c.additional_charge.as_deref())
                            + num(c.damage_charge.as_deref())
                            + num(c.extra_bed_charge.as_deref())
                            + num(c.pet_charge.as_deref()),
                        tax: num(c.tax_amount.as_deref()),
                    });
                }
                None => ledger.pending.push(detail),
            }
        } else if matches!(row.status_str(), "Confirmed" | "In-house") {
            ledger.expected_from_pending += row.amount();
            ledger.pending.push(row);
        }
    }
    Ok(ledger)
}

// ---------------------------------------------------------------------------
// Guests
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct GuestRow {
    pub id: i64,
    #[serde(default)]
    pub uuid: String,
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub display_email: String,
    #[serde(default)]
    pub display_phone: String,
    #[serde(default)]
    pub initials: String,
    #[serde(default)]
    pub nationality: Option<String>,
    #[serde(default)]
    pub id_or_passport_number: Option<String>,
    #[serde(default)]
    pub is_vip: bool,
    #[serde(default)]
    pub company: Option<String>,
    #[serde(default)]
    pub preferred_language: Option<String>,
    #[serde(default)]
    pub member_since: Option<String>,
    #[serde(default)]
    pub booking_ref: String,
    #[serde(default)]
    pub room_number: String,
    #[serde(default)]
    pub room_type: String,
    #[serde(default)]
    pub stay_dates: String,
    #[serde(default)]
    pub nights: u32,
    #[serde(default)]
    pub guests_count: u32,
    #[serde(default)]
    pub stay_status: String,
}

impl GuestRow {
    pub fn nationality_str(&self) -> &str {
        self.nationality
            .as_deref()
            .filter(|s| !s.is_empty() && *s != "Unspecified")
            .unwrap_or("—")
    }
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct GuestMetrics {
    #[serde(default)]
    pub total_guests: u32,
    #[serde(default)]
    pub currently_staying: u32,
    #[serde(default)]
    pub arriving_today: u32,
    #[serde(default)]
    pub checking_out_today: u32,
    #[serde(default)]
    pub vip_guests: u32,
}

/// The guest record itself, as returned nested under `data.guest`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct GuestProfile {
    #[serde(default, skip_serializing)]
    pub id: i64,
    #[serde(default)]
    pub full_name: String,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default, skip_serializing)]
    pub display_name: String,
    #[serde(default, skip_serializing)]
    pub display_email: String,
    #[serde(default, skip_serializing)]
    pub display_phone: String,
    #[serde(default, skip_serializing)]
    pub initials: String,
    #[serde(default)]
    pub id_or_passport_number: Option<String>,
    #[serde(default)]
    pub nationality: Option<String>,
    #[serde(default)]
    pub date_of_birth: Option<String>,
    #[serde(default)]
    pub preferred_language: Option<String>,
    #[serde(default)]
    pub address: Option<String>,
    #[serde(default)]
    pub is_vip: bool,
    #[serde(default, skip_serializing)]
    pub member_since: Option<String>,
    #[serde(default)]
    pub company: Option<String>,
    #[serde(default)]
    pub internal_notes: Option<String>,
    #[serde(default)]
    pub special_preferences: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct GuestStayMetrics {
    #[serde(default)]
    pub total_stays: u32,
    #[serde(default)]
    pub current_stay_length: Option<String>,
    #[serde(default)]
    pub current_stay_dates: Option<String>,
    #[serde(default)]
    pub total_nights: u32,
    #[serde(default)]
    pub guest_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct ActiveBooking {
    #[serde(default)]
    pub reservation_id: i64,
    #[serde(default)]
    pub booking_reference: String,
    #[serde(default)]
    pub room_number: Option<String>,
    #[serde(default)]
    pub room_type: Option<String>,
    #[serde(default)]
    pub check_in_date: Option<String>,
    #[serde(default)]
    pub check_out_date: Option<String>,
    #[serde(default)]
    pub stay_dates: Option<String>,
    #[serde(default)]
    pub nights: u32,
    #[serde(default)]
    pub number_of_guests: u32,
    #[serde(default)]
    pub payment_method: Option<String>,
    #[serde(default)]
    pub total_amount: Option<String>,
    #[serde(default)]
    pub stay_status: Option<String>,
    #[serde(default)]
    pub booking_status: Option<String>,
    #[serde(default)]
    pub special_notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct PastBooking {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub booking_reference: String,
    #[serde(default)]
    pub room_display: Option<String>,
    #[serde(default)]
    pub room_number: Option<String>,
    #[serde(default)]
    pub room_type: Option<String>,
    #[serde(default)]
    pub stay_dates: Option<String>,
    #[serde(default)]
    pub nights: u32,
    #[serde(default)]
    pub amount: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub payment_status: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct GuestDetail {
    #[serde(default)]
    pub guest: GuestProfile,
    #[serde(default)]
    pub metrics: GuestStayMetrics,
    #[serde(default)]
    pub active_booking: Option<ActiveBooking>,
    #[serde(default)]
    pub past_bookings: Vec<PastBooking>,
}

/// Maps a UI tab label to the API's `status_tab` value.
pub fn guest_tab_param(label: &str) -> String {
    match label {
        "All" | "" => String::new(),
        "In House" | "In-house" => "in_house".into(),
        "Arriving Today" => "arriving_today".into(),
        "Checking Out" | "Checking Out Today" => "checking_out_today".into(),
        "VIP" => "vip".into(),
        other => other.to_lowercase().replace(' ', "_"),
    }
}

pub async fn list_guests(status_tab: &str, search: &str) -> ApiResult<Page<GuestRow>> {
    let params: Params = vec![
        ("page_size", "100".into()),
        ("status_tab", guest_tab_param(status_tab)),
        ("search", search.trim().to_string()),
    ];
    request_page("/guests/", &params).await
}

pub async fn guest_metrics() -> ApiResult<GuestMetrics> {
    request(Method::Get, "/guests/metrics/", None, Auth::Required).await
}

pub async fn get_guest(id: i64) -> ApiResult<GuestDetail> {
    request(
        Method::Get,
        &format!("/guests/{id}/"),
        None,
        Auth::Required,
    )
    .await
}

/// Resolves the guest behind a booking reference.
///
/// The guest routes are keyed on the booking reference, and `/guests/` accepts
/// one as a search term, so a single lookup finds the record.
pub async fn get_guest_by_booking_ref(reference: &str) -> ApiResult<GuestDetail> {
    let page = list_guests("All", reference).await?;
    let hit = page
        .items
        .iter()
        .find(|g| g.booking_ref == reference)
        .or_else(|| page.items.first())
        .ok_or_else(|| ApiError {
            status: 404,
            code: "NOT_FOUND".into(),
            message: format!("No guest found for booking {reference}."),
            fields: Vec::new(),
        })?;
    get_guest(hit.id).await
}

pub async fn create_guest(guest: &GuestProfile) -> ApiResult<GuestProfile> {
    let body = serde_json::to_value(guest)
        .map_err(|e| ApiError::local(format!("could not encode guest: {e}")))?;
    request(Method::Post, "/guests/", Some(body), Auth::Required).await
}

pub async fn update_guest(id: i64, guest: &GuestProfile) -> ApiResult<GuestProfile> {
    let body = serde_json::to_value(guest)
        .map_err(|e| ApiError::local(format!("could not encode guest: {e}")))?;
    request(
        Method::Patch,
        &format!("/guests/{id}/"),
        Some(body),
        Auth::Required,
    )
    .await
}

pub async fn update_guest_notes(
    id: i64,
    internal_notes: &str,
    special_preferences: &str,
) -> ApiResult<()> {
    let body = serde_json::json!({
        "internal_notes": internal_notes,
        "special_preferences": special_preferences,
    });
    request_value(
        Method::Patch,
        &format!("/guests/{id}/notes/"),
        Some(body),
        Auth::Required,
    )
    .await?;
    Ok(())
}

pub async fn toggle_guest_vip(id: i64, is_vip: bool) -> ApiResult<()> {
    let body = serde_json::json!({ "is_vip": is_vip });
    request_value(
        Method::Patch,
        &format!("/guests/{id}/toggle-vip/"),
        Some(body),
        Auth::Required,
    )
    .await?;
    Ok(())
}

pub async fn export_guests() -> ApiResult<()> {
    download("/guests/export/", "guests.csv").await
}

// ---------------------------------------------------------------------------
// Reviews
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct ReviewBooking {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub booking_reference: String,
    #[serde(default)]
    pub check_in_date: Option<String>,
    #[serde(default)]
    pub check_out_date: Option<String>,
    #[serde(default)]
    pub room_number: Option<String>,
    #[serde(default)]
    pub room_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct ReviewUser {
    #[serde(default)]
    pub full_name: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub profile_picture: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct ReviewRow {
    pub id: i64,
    #[serde(default)]
    pub rate: i32,
    #[serde(default)]
    pub comment: String,
    #[serde(default)]
    pub reply: Option<String>,
    #[serde(default)]
    pub replied_at: Option<String>,
    #[serde(default)]
    pub replied_by_name: Option<String>,
    #[serde(default)]
    pub guest_name: String,
    #[serde(default)]
    pub user_info: Option<ReviewUser>,
    #[serde(default)]
    pub booking_info: Option<ReviewBooking>,
    #[serde(default)]
    pub created_at: Option<String>,
}

impl ReviewRow {
    pub fn has_reply(&self) -> bool {
        self.reply.as_deref().is_some_and(|r| !r.trim().is_empty())
    }
    pub fn initials(&self) -> String {
        self.guest_name
            .split_whitespace()
            .filter_map(|w| w.chars().next())
            .take(2)
            .collect::<String>()
            .to_uppercase()
    }
    pub fn room_label(&self) -> String {
        match &self.booking_info {
            Some(b) => match (b.room_number.as_deref(), b.room_type.as_deref()) {
                (Some(n), Some(t)) if !n.is_empty() => format!("{n} · {t}"),
                (Some(n), _) if !n.is_empty() => n.to_string(),
                _ => "—".into(),
            },
            None => "—".into(),
        }
    }
    pub fn reference(&self) -> String {
        self.booking_info
            .as_ref()
            .map(|b| b.booking_reference.clone())
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct RatingBucket {
    #[serde(default)]
    pub count: u32,
    #[serde(default)]
    pub percentage: f32,
}

/// Star histogram. The API keys these `5_star` … `1_star`.
#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct RatingBreakdown {
    #[serde(default, rename = "5_star")]
    pub five: RatingBucket,
    #[serde(default, rename = "4_star")]
    pub four: RatingBucket,
    #[serde(default, rename = "3_star")]
    pub three: RatingBucket,
    #[serde(default, rename = "2_star")]
    pub two: RatingBucket,
    #[serde(default, rename = "1_star")]
    pub one: RatingBucket,
}

impl RatingBreakdown {
    /// Five stars first, as the UI lists them.
    pub fn rows(&self) -> [(i32, &RatingBucket); 5] {
        [
            (5, &self.five),
            (4, &self.four),
            (3, &self.three),
            (2, &self.two),
            (1, &self.one),
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct ReviewMetrics {
    #[serde(default)]
    pub total_reviews: u32,
    #[serde(default)]
    pub average_rating: f32,
    #[serde(default)]
    pub rating_breakdown: RatingBreakdown,
    #[serde(default)]
    pub replied_count: u32,
    #[serde(default)]
    pub pending_reply_count: u32,
    #[serde(default)]
    pub reply_rate_percentage: f32,
}

/// `rate` filters to one star value; `has_reply` splits replied / awaiting.
pub async fn list_reviews(
    search: &str,
    rate: Option<i32>,
    has_reply: Option<bool>,
) -> ApiResult<Page<ReviewRow>> {
    let mut params: Params = vec![
        ("page_size", "100".into()),
        ("search", search.trim().to_string()),
        ("ordering", "-created_at".into()),
    ];
    if let Some(r) = rate {
        params.push(("rate", r.to_string()));
    }
    if let Some(h) = has_reply {
        params.push(("has_reply", h.to_string()));
    }
    request_page("/reviews/", &params).await
}

pub async fn review_metrics() -> ApiResult<ReviewMetrics> {
    request(Method::Get, "/reviews/metrics/", None, Auth::Required).await
}

/// Posts (or updates) the hotel's reply to a guest review.
pub async fn reply_review(id: i64, text: &str) -> ApiResult<()> {
    let body = serde_json::json!({ "reply": text });
    request_value(
        Method::Post,
        &format!("/reviews/{id}/reply/"),
        Some(body),
        Auth::Required,
    )
    .await?;
    Ok(())
}

pub async fn delete_review(id: i64) -> ApiResult<()> {
    request_value(
        Method::Delete,
        &format!("/reviews/{id}/"),
        None,
        Auth::Required,
    )
    .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Staff & roles
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct StaffMember {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub uuid: String,
    #[serde(default)]
    pub first_name: Option<String>,
    #[serde(default)]
    pub last_name: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub role_id: Option<i64>,
    #[serde(default)]
    pub role_name: Option<String>,
    #[serde(default)]
    pub is_system_role: bool,
    #[serde(default)]
    pub created_at: Option<String>,
}

impl StaffMember {
    pub fn name(&self) -> String {
        let full = [self.first_name.as_deref(), self.last_name.as_deref()]
            .into_iter()
            .flatten()
            .filter(|s| !s.trim().is_empty())
            .collect::<Vec<_>>()
            .join(" ");
        if full.is_empty() {
            self.email.clone().unwrap_or_else(|| "Staff".into())
        } else {
            full
        }
    }
    pub fn initials(&self) -> String {
        self.name()
            .split_whitespace()
            .filter_map(|w| w.chars().next())
            .take(2)
            .collect::<String>()
            .to_uppercase()
    }
    pub fn role_str(&self) -> &str {
        self.role_name.as_deref().unwrap_or("—")
    }
    pub fn status_str(&self) -> &str {
        self.status.as_deref().unwrap_or("Active")
    }
    pub fn is_active(&self) -> bool {
        self.status_str() == "Active"
    }
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct Role {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub uuid: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub is_system_role: bool,
    #[serde(default)]
    pub permissions: Vec<String>,
}

pub async fn list_staff() -> ApiResult<Vec<StaffMember>> {
    let org = require_org()?;
    request_list(&format!("/organizations/{org}/staff/"), &Vec::new()).await
}

pub async fn list_roles() -> ApiResult<Vec<Role>> {
    request_list("/roles-permissions/roles/", &Vec::new()).await
}

/// The permission codes the platform defines, for the role editor.
pub async fn list_permissions() -> ApiResult<Vec<String>> {
    let data = request_value(
        Method::Get,
        "/roles-permissions/permissions/",
        None,
        Auth::Required,
    )
    .await?;
    // The endpoint returns either bare codes or objects carrying a `code`.
    let items = data.as_array().cloned().unwrap_or_default();
    Ok(items
        .into_iter()
        .filter_map(|v| match v {
            serde_json::Value::String(s) => Some(s),
            other => other
                .get("code")
                .or_else(|| other.get("codename"))
                .or_else(|| other.get("name"))
                .and_then(|c| c.as_str())
                .map(str::to_string),
        })
        .collect())
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct NewStaff {
    pub first_name: String,
    pub last_name: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub email: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub phone: String,
    pub password: String,
    pub role_id: i64,
}

pub async fn add_staff(staff: &NewStaff) -> ApiResult<StaffMember> {
    let org = require_org()?;
    let body = serde_json::to_value(staff)
        .map_err(|e| ApiError::local(format!("could not encode staff: {e}")))?;
    request(
        Method::Post,
        &format!("/organizations/{org}/staff/"),
        Some(body),
        Auth::Required,
    )
    .await
}

pub async fn update_staff_role(user_id: i64, role_id: i64) -> ApiResult<StaffMember> {
    let org = require_org()?;
    let body = serde_json::json!({ "role_id": role_id });
    request(
        Method::Patch,
        &format!("/organizations/{org}/staff/{user_id}/"),
        Some(body),
        Auth::Required,
    )
    .await
}

pub async fn remove_staff(user_id: i64) -> ApiResult<()> {
    let org = require_org()?;
    request_value(
        Method::Delete,
        &format!("/organizations/{org}/staff/{user_id}/"),
        None,
        Auth::Required,
    )
    .await?;
    Ok(())
}

pub async fn reset_staff_password(user_id: i64, new_password: &str) -> ApiResult<()> {
    let org = require_org()?;
    let body = serde_json::json!({ "new_password": new_password });
    request_value(
        Method::Post,
        &format!("/organizations/{org}/staff/{user_id}/reset-password/"),
        Some(body),
        Auth::Required,
    )
    .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Notifications
// ---------------------------------------------------------------------------

pub const NOTIFICATION_TYPES: &[&str] = &["Booking", "Payment", "System", "Alert"];
pub const TARGET_ROLES: &[&str] = &["ALL", "STAFF", "ADMIN", "GUEST"];

#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct Sender {
    #[serde(default)]
    pub full_name: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub profile_picture: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct Notification {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub sender_info: Option<Sender>,
    #[serde(default)]
    pub notification_type: Option<String>,
    #[serde(default)]
    pub target_role: Option<String>,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub entity_type: Option<String>,
    #[serde(default)]
    pub entity_id: Option<String>,
    #[serde(default)]
    pub action_url: Option<String>,
    #[serde(default)]
    pub is_read: bool,
    #[serde(default)]
    pub created_at: Option<String>,
}

impl Notification {
    pub fn kind(&self) -> &str {
        self.notification_type.as_deref().unwrap_or("System")
    }
    pub fn sender_name(&self) -> String {
        self.sender_info
            .as_ref()
            .and_then(|s| s.full_name.clone())
            .filter(|n| !n.trim().is_empty())
            .unwrap_or_else(|| "System".into())
    }
    pub fn initials(&self) -> String {
        self.sender_name()
            .split_whitespace()
            .filter_map(|w| w.chars().next())
            .take(2)
            .collect::<String>()
            .to_uppercase()
    }
    /// Where the CRM should navigate when the row is opened.
    pub fn route(&self) -> Option<String> {
        match (self.entity_type.as_deref(), self.entity_id.as_deref()) {
            (Some("RESERVATION"), Some(_)) => Some("/reservations".into()),
            (Some("REVIEW"), _) => Some("/reviews".into()),
            (Some("ROOM"), _) => Some("/rooms".into()),
            (Some("GUEST"), _) => Some("/guests".into()),
            (Some("PAYMENT"), _) => Some("/payments".into()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct UnreadCounts {
    #[serde(default)]
    pub unread_count: u32,
    #[serde(default)]
    pub booking_unread: u32,
    #[serde(default)]
    pub payment_unread: u32,
    #[serde(default)]
    pub system_unread: u32,
    #[serde(default)]
    pub alert_unread: u32,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct NotificationPrefs {
    #[serde(default)]
    pub sound_enabled: bool,
    #[serde(default)]
    pub email_notifications: bool,
    #[serde(default)]
    pub booking_alerts: bool,
    #[serde(default)]
    pub payment_alerts: bool,
    #[serde(default)]
    pub system_alerts: bool,
    #[serde(default)]
    pub sound_choice: Option<String>,
}

pub async fn list_notifications(
    kind: Option<&str>,
    is_read: Option<bool>,
    search: &str,
) -> ApiResult<Page<Notification>> {
    let mut params: Params = vec![
        ("page_size", "100".into()),
        ("search", search.trim().to_string()),
        ("ordering", "-created_at".into()),
    ];
    if let Some(k) = kind.filter(|k| *k != "All") {
        params.push(("notification_type", k.to_string()));
    }
    if let Some(r) = is_read {
        params.push(("is_read", r.to_string()));
    }
    request_page("/notifications/", &params).await
}

pub async fn unread_counts() -> ApiResult<UnreadCounts> {
    request(
        Method::Get,
        "/notifications/unread-count/",
        None,
        Auth::Required,
    )
    .await
}

pub async fn mark_notification_read(id: i64) -> ApiResult<()> {
    request_value(
        Method::Post,
        &format!("/notifications/{id}/read/"),
        None,
        Auth::Required,
    )
    .await?;
    Ok(())
}

pub async fn mark_all_notifications_read() -> ApiResult<()> {
    request_value(
        Method::Post,
        "/notifications/mark-all-read/",
        None,
        Auth::Required,
    )
    .await?;
    Ok(())
}

pub async fn delete_notification(id: i64) -> ApiResult<()> {
    request_value(
        Method::Delete,
        &format!("/notifications/{id}/"),
        None,
        Auth::Required,
    )
    .await?;
    Ok(())
}

pub async fn get_notification_prefs() -> ApiResult<NotificationPrefs> {
    request(
        Method::Get,
        "/notifications/preferences/",
        None,
        Auth::Required,
    )
    .await
}

pub async fn update_notification_prefs(prefs: &NotificationPrefs) -> ApiResult<NotificationPrefs> {
    let body = serde_json::to_value(prefs)
        .map_err(|e| ApiError::local(format!("could not encode preferences: {e}")))?;
    request(
        Method::Patch,
        "/notifications/preferences/",
        Some(body),
        Auth::Required,
    )
    .await
}

/// Sends a notification to a role inside the hotel.
///
/// This is the closest the API has to messaging — there is no guest-thread
/// resource — so the CRM's Messages screen is built on broadcast plus the
/// notification feed.
pub async fn broadcast_notification(
    target_role: &str,
    title: &str,
    message: &str,
    kind: &str,
) -> ApiResult<()> {
    let mut body = serde_json::json!({
        "target_role": target_role,
        "title": title.trim(),
        "message": message.trim(),
        "notification_type": kind,
        "entity_type": "SYSTEM",
    });
    if let (Some(map), Some(org)) = (body.as_object_mut(), session::active_org()) {
        map.insert("organization_id".into(), serde_json::json!(org));
    }
    request_value(
        Method::Post,
        "/notifications/broadcast/",
        Some(body),
        Auth::Required,
    )
    .await?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Audit log
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct AuditActor {
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub first_name: Option<String>,
    #[serde(default)]
    pub last_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct AuditDetails {
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
pub struct AuditLog {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub uuid: String,
    #[serde(default)]
    pub action: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub ip_address: Option<String>,
    #[serde(default)]
    pub details: Option<AuditDetails>,
    #[serde(default)]
    pub user: Option<AuditActor>,
    #[serde(default)]
    pub created_at: Option<String>,
}

impl AuditLog {
    /// `STAFF_MEMBER_ADDED` -> `Staff member added`.
    pub fn action_label(&self) -> String {
        let lower = self.action.replace('_', " ").to_lowercase();
        let mut c = lower.chars();
        match c.next() {
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            None => lower,
        }
    }
    pub fn description(&self) -> String {
        self.details
            .as_ref()
            .and_then(|d| d.description.clone())
            .unwrap_or_else(|| self.action_label())
    }
    pub fn actor(&self) -> String {
        let Some(u) = &self.user else {
            return "System".into();
        };
        let full = [u.first_name.as_deref(), u.last_name.as_deref()]
            .into_iter()
            .flatten()
            .filter(|s| !s.trim().is_empty())
            .collect::<Vec<_>>()
            .join(" ");
        if full.is_empty() {
            u.email.clone().unwrap_or_else(|| "System".into())
        } else {
            full
        }
    }
    pub fn succeeded(&self) -> bool {
        self.status.eq_ignore_ascii_case("success")
    }
}

pub async fn list_audit_logs(search: &str, action: &str) -> ApiResult<Page<AuditLog>> {
    let params: Params = vec![
        ("page_size", "50".into()),
        ("search", search.trim().to_string()),
        ("action", action.trim().to_string()),
    ];
    request_page("/accounts/audit-logs/", &params).await
}
