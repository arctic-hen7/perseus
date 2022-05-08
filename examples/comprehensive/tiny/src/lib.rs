use perseus::{Html, PerseusApp, Template};
use sycamore::view;

#[perseus::main]
pub fn main<G: Html>() -> PerseusApp<G> {
    PerseusApp::new().template(|| {
        Template::new("index").template(|cx, _| {
            view! { cx,
                p { "Hello World!" }
            }
        })
    })
}
