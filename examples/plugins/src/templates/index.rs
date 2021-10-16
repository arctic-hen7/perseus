use perseus::{GenericNode, Template};
use sycamore::prelude::{component, template, Template as SycamoreTemplate};

#[component(IndexPage<G>)]
pub fn index_page() -> SycamoreTemplate<G> {
    template! {
        p { "Hello World!" }
        a(href = "about", id = "about-link") { "About!" }
    }
}

pub fn get_template<G: GenericNode>() -> Template<G> {
    Template::new("index")
        .template(|_| {
            template! {
                IndexPage()
            }
        })
        .head(|_| {
            template! {
                title { "Index Page | Perseus Example – Plugins" }
            }
        })
}
