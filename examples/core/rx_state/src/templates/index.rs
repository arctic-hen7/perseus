use perseus::{Html, RenderFnResultWithCause, Template};
use sycamore::prelude::{view, Scope, SsrNode, View};

#[perseus::make_rx(IndexPageStateRx)]
pub struct IndexPageState {
    pub username: String,
}

// This macro will make our state reactive *and* store it in the page state store, which means it'll be the same even if we go to the about page and come back (as long as we're in the same session)
#[perseus::template_rx]
pub fn index_page<G: Html>(cx: Scope, state: IndexPageStateRx) -> View<G> {
    let username = state.username;
    let username_2 = username.clone();

    view! { cx,
        p { (format!("Greetings, {}!", username.get())) }
        input(bind:value = username_2, placeholder = "Username")

        a(href = "about", id = "about-link") { "About" }
    }
}

#[perseus::head]
pub fn head(cx: Scope) -> View<SsrNode> {
    view! { cx,
        title { "Index Page" }
    }
}

pub fn get_template<G: Html>() -> Template<G> {
    Template::new("index")
        .template(index_page)
        .head(head)
        .build_state_fn(get_build_state)
}

#[perseus::autoserde(build_state)]
pub async fn get_build_state(
    _path: String,
    _locale: String,
) -> RenderFnResultWithCause<IndexPageState> {
    Ok(IndexPageState {
        username: "".to_string(),
    })
}
