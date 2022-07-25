use perseus::utils::cookies::Cookies;
use perseus::{RenderFnResultWithCause, Request, Template};
use sycamore::prelude::{view, Html, Scope, SsrNode, View};

#[perseus::make_rx(IndexPropsRx)]
pub struct IndexProps {
    cookie_value: String,
}

#[perseus::template_rx]
pub fn index_page<'a, G: Html>(cx: Scope<'a>, props: IndexPropsRx<'a>) -> View<G> {
    let cookie_value = props.cookie_value.clone();
    view! { cx,
        p { (cookie_value) }
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
        .request_state_fn(get_request_state)
        .template(index_page)
        .head(head)
}

#[perseus::request_state]
pub async fn get_request_state(
    _path: String,
    _locale: String,
    mut req: Request,
) -> RenderFnResultWithCause<IndexProps> {
    let cookie_value = req.get_cookie("test").unwrap();
    req.set_cookie("foo", "bar");
    Ok(IndexProps { cookie_value })
}
