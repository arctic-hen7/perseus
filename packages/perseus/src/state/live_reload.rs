use sycamore::prelude::{
    create_rc_signal, provide_context, try_use_context, use_context, RcSignal, Scope,
};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::{ErrorEvent, MessageEvent, WebSocket};

#[derive(Debug)]
struct LiveReloadIndicatorInner(bool);

/// A context store for the live reload indicator, a flip-flop `RcSignal` used for emitting a live reload event.
///
/// This has to be an `RcSignal` to avoid the issues of closure wrapping.
#[derive(Debug, Clone)]
pub struct LiveReloadIndicator(RcSignal<LiveReloadIndicatorInner>);
impl<'a> LiveReloadIndicator {
    /// Creates a new instance of the live reload indicator, registering with the context of the given reactive scope and mirroring it.
    pub fn new(cx: Scope<'a>) -> Self {
        let inner = {
            if let Some(ctx) = try_use_context::<RcSignal<LiveReloadIndicatorInner>>(cx) {
                ctx.set(LiveReloadIndicatorInner(true));
            } else {
                provide_context(cx, create_rc_signal(LiveReloadIndicatorInner(true)));
            }
            use_context::<RcSignal<LiveReloadIndicatorInner>>(cx)
        }
        .clone();

        Self(inner)
    }
    /// Creates a new instance of the live reload indicator from the context of the given reactive scope. If the required types do not exist in the given scope, this will panic.
    pub fn from_ctx(cx: Scope<'a>) -> Self {
        Self(use_context::<RcSignal<LiveReloadIndicatorInner>>(cx).clone())
    }
    /// Gets the inner value.
    pub fn get(&self) -> bool {
        self.0.get().0
    }
    /// Gets the inner value without adding it to reactive dependencies.
    pub fn get_untracked(&self) -> bool {
        self.0.get_untracked().0
    }
    /// Sets the inner value.
    pub fn set(&self, val: bool) {
        self.0.set(LiveReloadIndicatorInner(val))
    }
}

/// Connects to the reload server if it's online. This takes a flip-flop `RcSignal` that it can use to signal other parts of the code to perform actual reloading (we can't do that here because
/// we don't have access to the render context for freezing and thawing).
///
/// We need to use an `RcSignal` here due to the requirements of closure wrapping (lifetime scopes do not work there).
pub(crate) fn connect_to_reload_server(live_reload_indicator: LiveReloadIndicator) {
    // Get the host and port
    let host = get_window_var("__PERSEUS_RELOAD_SERVER_HOST");
    let port = get_window_var("__PERSEUS_RELOAD_SERVER_PORT");
    let (host, port) = match (host, port) {
        (Some(host), Some(port)) => (host, port),
        // If either the host or port weren't set, the server almost certainly isn't online, so we won't connect
        _ => return,
    };

    // Connect to the server (it's expected that the host does not include a protocol)
    let ws = match WebSocket::new(&format!("ws://{}:{}/receive", host, port)) {
        Ok(ws) => ws,
        Err(err) => return log(&format!("Connection failed: {:?}.", err)),
    };
    // This is apparently more efficient for small bianry messages
    ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

    // Set up a message handler
    let onmessage_callback = Closure::wrap(Box::new(move |_| {
        // With this server, if we receive any message it will be telling us to reload, so we'll do so
        log("Reloading...");
        // Signal the rest of the code that we need to reload (and potentially freeze state if HSR is enabled)
        // Amazingly, the reactive scope isn't interrupted and this actually works!
        live_reload_indicator.set(!live_reload_indicator.get_untracked());
    }) as Box<dyn FnMut(MessageEvent)>);
    ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
    // To keep the closure alive, we need to forget about it
    onmessage_callback.forget();

    // We should log to the console about errors from the server
    let onerror_callback = Closure::wrap(Box::new(move |err: ErrorEvent| {
        log(&format!("Error: {:?}.", err));
    }) as Box<dyn FnMut(ErrorEvent)>);
    ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
    onerror_callback.forget();

    // We'll just log when the connection is successfully established for informational purposes
    let onopen_callback = Closure::wrap(Box::new(move |_| {
        log("Connected.");
    }) as Box<dyn FnMut(JsValue)>);
    ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
    onopen_callback.forget();
}

fn get_window_var(name: &str) -> Option<String> {
    let val_opt = web_sys::window().unwrap().get(name);
    let js_obj = match val_opt {
        Some(js_obj) => js_obj,
        None => return None,
    };
    // The object should only actually contain the string value that was injected
    let state_str = match js_obj.as_string() {
        Some(state_str) => state_str,
        None => return None,
    };
    // On the server-side, we encode a `None` value directly (otherwise it will be some convoluted stringified JSON)
    Some(state_str)
}

/// An internal function for logging data for development reloading.
fn log(msg: &str) {
    web_sys::console::log_1(&JsValue::from("[Live Reload Server]: ".to_string() + msg));
}

/// Force-reloads the page. Any code after this will NOT be called, as the browser will completely reload the page, dumping your code and restarting from the beginning. This will result in
/// a total loss of all state unless it's frozen in some way.
///
/// Note that the parameter that forces the browser to bypass its cache is non-standard, and only impacts Firefox. On all other browsers, this has no effect.
///
/// # Panics
/// This will panic if it was impossible to reload (which would be caused by a *very* old browser).
pub fn force_reload() {
    web_sys::window()
        .unwrap()
        .location()
        .reload_with_forceget(true)
        .unwrap();
}
