use perseus::{RenderFnResultWithCause, Template};
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use sycamore::prelude::{component, template, GenericNode, Template as SycamoreTemplate};

#[derive(Serialize, Deserialize, Debug)]
pub struct IndexPageProps {
    pub greeting: String,
}

#[component(IndexPage<G>)]
pub fn index_page(props: IndexPageProps) -> SycamoreTemplate<G> {
    template! {
        p {(props.greeting)}
    }
}

pub fn get_template<G: GenericNode>() -> Template<G> {
    Template::new("index")
        .build_state_fn(Rc::new(get_static_props))
        .template(Rc::new(|props| {
            template! {
                IndexPage(
                    serde_json::from_str::<IndexPageProps>(&props.unwrap()).unwrap()
                )
            }
        }))
}

pub async fn get_static_props(_path: String, _locale: String) -> RenderFnResultWithCause<String> {
    Ok(serde_json::to_string(&IndexPageProps {
        greeting: "Hello World!".to_string(),
    })?)
}
