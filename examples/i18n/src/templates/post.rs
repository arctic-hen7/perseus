use perseus::{StringResultWithCause, Template};
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use sycamore::prelude::{component, template, GenericNode, Template as SycamoreTemplate};

#[derive(Serialize, Deserialize)]
pub struct PostPageProps {
    title: String,
    content: String,
}

#[component(PostPage<G>)]
pub fn post_page(props: PostPageProps) -> SycamoreTemplate<G> {
    let title = props.title;
    let content = props.content;
    template! {
        h1 {
            (title)
        }
        p {
            (content)
        }
    }
}

pub fn get_template<G: GenericNode>() -> Template<G> {
    Template::new("post")
        .build_paths_fn(Rc::new(get_static_paths))
        .build_state_fn(Rc::new(get_static_props))
        .template(template_fn())
}

pub async fn get_static_props(path: String) -> StringResultWithCause<String> {
    // This is just an example
    let title = urlencoding::decode(&path).unwrap();
    let content = format!(
        "This is a post entitled '{}'. Its original slug was '{}'.",
        title, path
    );

    Ok(serde_json::to_string(&PostPageProps {
        title: title.to_string(),
        content,
    })
    .unwrap())
}

pub async fn get_static_paths() -> Result<Vec<String>, String> {
    Ok(vec!["test".to_string(), "blah/test/blah".to_string()])
}

pub fn template_fn<G: GenericNode>() -> perseus::template::TemplateFn<G> {
    Rc::new(|props| {
        template! {
            PostPage(
                serde_json::from_str::<PostPageProps>(&props.unwrap()).unwrap()
            )
        }
    })
}
