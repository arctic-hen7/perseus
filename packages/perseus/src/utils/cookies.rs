/// This Modules add support for cookies in the Perseus Request Type
use crate::Request;

/// Cookies Trait impl for Request type
pub trait Cookies {
    /// Get Cookie from cookie header
    fn get_cookie(&self, name: &str) -> Option<String>;
    /// Set Cookie over Set-Cookie header
    fn set_cookie(&mut self, name: &str, value: &str);
}

impl Cookies for Request {
    /// Reads the cookie from the Cookies header
    fn get_cookie(&self, name: &str) -> Option<String> {
        self.headers().get("Cookie").and_then(|cookie_header| {
            let cookie_header = cookie_header.to_str().ok()?;
            let mut cookies = cookie_header.split(';');
            for cookie in cookies.by_ref() {
                let cookie = cookie.trim();
                if cookie.starts_with(name) {
                    let mut parts = cookie.split('=');
                    let _ = parts.next();
                    return Some(parts.next()?.to_string());
                }
            }
            None
        })
    }

    /// Sets a cookie with the Set-Cookie header
    fn set_cookie(&mut self, name: &str, value: &str) {
        self.headers_mut().insert(
            "Set-Cookie",
            format!("{}={}; Path=/", name, value).parse().unwrap(),
        );
    }
}
