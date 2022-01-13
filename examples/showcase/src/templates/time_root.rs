use std::time::Duration;

use perseus::{RenderFnResultWithCause, Template};
use serde::{Deserialize, Serialize};
use sycamore::prelude::{component, view, Html, View};

#[derive(Serialize, Deserialize, Debug)]
pub struct TimePageProps {
    pub time: String,
}

#[perseus::template(TimePage)]
#[component(TimePage<G>)]
pub fn time_page(props: TimePageProps) -> View<G> {
    view! {
        p { (format!("The time when this page was last rendered was '{}'.", props.time)) }
    }
}

pub fn get_template<G: Html>() -> Template<G> {
    Template::new("time")
        .template(time_page)
        // This page will revalidate every five seconds (to illustrate revalidation)
        // Try changing this to a week, even though the below custom logic says to always revalidate, we'll only do it weekly
        .revalidate_after(Duration::new(5, 0))
        .should_revalidate_fn(|| async { Ok(true) })
        .build_state_fn(get_build_state)
}

#[perseus::autoserde(build_state)]
pub async fn get_build_state(
    _path: String,
    _locale: String,
) -> RenderFnResultWithCause<TimePageProps> {
    Ok(TimePageProps {
        time: format!("{:?}", std::time::SystemTime::now()),
    })
}
