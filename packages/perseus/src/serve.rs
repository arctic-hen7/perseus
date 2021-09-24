// This file contains the universal logic for a serving process, regardless of framework

use crate::config_manager::ConfigManager;
use crate::decode_time_str::decode_time_str;
use crate::errors::*;
use crate::template::{States, Template, TemplateMap};
use crate::Request;
use crate::TranslationsManager;
use crate::Translator;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::rc::Rc;
use sycamore::prelude::SsrNode;

/// Represents the data necessary to render a page, including document metadata.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PageData {
    /// Prerendered HTML content.
    pub content: String,
    /// The state for hydration. This is kept as a string for ease of typing. Some pages may not need state or generate it in another way,
    /// so this might be `None`.
    pub state: Option<String>,
    /// The string to interpolate into the document's `<head>`.
    pub head: String,
}

/// Gets the configuration of how to render each page.
pub async fn get_render_cfg(
    config_manager: &impl ConfigManager,
) -> Result<HashMap<String, String>> {
    let content = config_manager.read("render_conf.json").await?;
    let cfg = serde_json::from_str::<HashMap<String, String>>(&content)?;

    Ok(cfg)
}

/// Renders a template that uses state generated at build-time.
async fn render_build_state(
    path_encoded: &str,
    config_manager: &impl ConfigManager,
) -> Result<(String, String, Option<String>)> {
    // Get the static HTML
    let html = config_manager
        .read(&format!("static/{}.html", path_encoded))
        .await?;
    let head = config_manager
        .read(&format!("static/{}.head.html", path_encoded))
        .await?;
    // Get the static JSON
    let state = match config_manager
        .read(&format!("static/{}.json", path_encoded))
        .await
    {
        Ok(state) => Some(state),
        Err(_) => None,
    };

    Ok((html, head, state))
}
/// Renders a template that generated its state at request-time. Note that revalidation and ISR have no impact on SSR-rendered pages.
async fn render_request_state(
    template: &Template<SsrNode>,
    translator: Rc<Translator>,
    path: &str,
    req: Request,
) -> Result<(String, String, Option<String>)> {
    // Generate the initial state (this may generate an error, but there's no file that can't exist)
    let state = Some(template.get_request_state(path.to_string(), req).await?);
    // Use that to render the static HTML
    let html =
        sycamore::render_to_string(|| template.render_for_template(state.clone(), Rc::clone(&translator)));
    let head = template.render_head_str(state.clone(), Rc::clone(&translator));

    Ok((html, head, state))
}
/// Checks if a template that uses ISR has already been cached.
async fn get_incremental_cached(
    path_encoded: &str,
    config_manager: &impl ConfigManager,
) -> Option<(String, String)> {
    let html_res = config_manager
        .read(&format!("static/{}.html", path_encoded))
        .await;

    // We should only treat it as cached if it can be accessed and if we aren't in development (when everything should constantly reload)
    match html_res {
        Ok(html) if !cfg!(debug_assertions) => {
            // If the HTML exists, the head must as well
            let head = config_manager
                .read(&format!("static/{}.html", path_encoded))
                .await
                .unwrap();
            Some((html, head))
        },
        Ok(_) | Err(_) => None,
    }
}
/// Checks if a template should revalidate by time.
async fn should_revalidate(
    template: &Template<SsrNode>,
    path_encoded: &str,
    config_manager: &impl ConfigManager,
) -> Result<bool> {
    let mut should_revalidate = false;
    // If it revalidates after a certain period of time, we needd to check that BEFORE the custom logic
    if template.revalidates_with_time() {
        // Get the time when it should revalidate (RFC 3339)
        let datetime_to_revalidate_str = config_manager
            .read(&format!("static/{}.revld.txt", path_encoded))
            .await?;
        let datetime_to_revalidate = DateTime::parse_from_rfc3339(&datetime_to_revalidate_str)?;
        // Get the current time (UTC)
        let now = Utc::now();

        // If the datetime to revalidate is still in the future, end with `false`
        if datetime_to_revalidate > now {
            return Ok(false);
        }
        should_revalidate = true;
    }

    // Now run the user's custom revalidation logic
    if template.revalidates_with_logic() {
        should_revalidate = template.should_revalidate().await?;
    }
    Ok(should_revalidate)
}
/// Revalidates a template
async fn revalidate(
    template: &Template<SsrNode>,
    translator: Rc<Translator>,
    path: &str,
    path_encoded: &str,
    config_manager: &impl ConfigManager,
) -> Result<(String, String, Option<String>)> {
    // We need to regenerate and cache this page for future usage (until the next revalidation)
    let state = Some(
        template
            .get_build_state(format!("{}/{}", template.get_path(), path))
            .await?,
    );
    let html =
        sycamore::render_to_string(|| template.render_for_template(state.clone(), Rc::clone(&translator)));
    let head = template.render_head_str(state.clone(), Rc::clone(&translator));
    // Handle revalidation, we need to parse any given time strings into datetimes
    // We don't need to worry about revalidation that operates by logic, that's request-time only
    if template.revalidates_with_time() {
        // IMPORTANT: we set the new revalidation datetime to the interval from NOW, not from the previous one
        // So if you're revalidating many pages weekly, they will NOT revalidate simultaneously, even if they're all queried thus
        let datetime_to_revalidate = decode_time_str(&template.get_revalidate_interval().unwrap())?;
        config_manager
            .write(
                &format!("static/{}.revld.txt", path_encoded),
                &datetime_to_revalidate,
            )
            .await?;
    }
    config_manager
        .write(
            &format!("static/{}.json", path_encoded),
            &state.clone().unwrap(),
        )
        .await?;
    config_manager
        .write(&format!("static/{}.html", path_encoded), &html)
        .await?;
    config_manager
        .write(&format!("static/{}.head.html", path_encoded), &head)
        .await?;

    Ok((html, head, state))
}

/// Internal logic behind `get_page`. The only differences are that this takes a full template rather than just a template name, which
/// can avoid an unnecessary lookup if you already know the template in full (e.g. initial load server-side routing).
// TODO possible further optimizations on this for futures?
pub async fn get_page_for_template(
    // This must not contain the locale
    raw_path: &str,
    locale: &str,
    template: &Template<SsrNode>,
    req: Request,
    config_manager: &impl ConfigManager,
    translations_manager: &impl TranslationsManager
) -> Result<PageData> {
    // Get a translator for this locale (for sanity we hope the manager is caching)
    let translator = Rc::new(
        translations_manager
            .get_translator_for_locale(locale.to_string())
            .await?,
    );

    let mut path = raw_path;
    // If the path is empty, we're looking for the special `index` page
    if path.is_empty() {
        path = "index";
    }
    // Remove `/` from the path by encoding it as a URL (that's what we store) and add the locale
    let path_encoded = format!("{}-{}", locale, urlencoding::encode(path).to_string());

    // Only a single string of HTML is needed, and it will be overridden if necessary (priorities system)
    let mut html = String::new();
    // The same applies for the document metadata
    let mut head = String::new();
    // Multiple rendering strategies may need to amalgamate different states
    let mut states: States = States::new();

    // Handle build state (which might use revalidation or incremental)
    if template.uses_build_state() || template.is_basic() {
        // If the template uses incremental generation, that is its own contained process
        if template.uses_incremental() {
            // Get the cached content if it exists (otherwise `None`)
            let html_and_head_opt = get_incremental_cached(&path_encoded, config_manager).await;
            match html_and_head_opt {
                // It's cached
                Some((html_val, head_val)) => {
                    // Check if we need to revalidate
                    if should_revalidate(template, &path_encoded, config_manager).await? {
                        let (html_val, head_val, state) = revalidate(
                            template,
                            Rc::clone(&translator),
                            path,
                            &path_encoded,
                            config_manager,
                        )
                        .await?;
                        // Build-time generated HTML is the lowest priority, so we'll only set it if nothing else already has
                        if html.is_empty() {
                            html = html_val;
                            head = head_val;
                        }
                        states.build_state = state;
                    } else {
                        // Build-time generated HTML is the lowest priority, so we'll only set it if nothing else already has
                        if html.is_empty() {
                            html = html_val;
                            head = head_val;
                        }
                        // Get the static JSON (if it exists, but it should)
                        states.build_state = match config_manager
                            .read(&format!("static/{}.json", path_encoded))
                            .await
                        {
                            Ok(state) => Some(state),
                            Err(_) => None,
                        };
                    }
                }
                // It's not cached
                None => {
                    // We need to generate and cache this page for future usage
                    let state = Some(template.get_build_state(path.to_string()).await?);
                    let html_val = sycamore::render_to_string(|| {
                        template.render_for_template(state.clone(), Rc::clone(&translator))
                    });
                    let head_val = template.render_head_str(state.clone(), Rc::clone(&translator));
                    // Handle revalidation, we need to parse any given time strings into datetimes
                    // We don't need to worry about revalidation that operates by logic, that's request-time only
                    // Obviously we don't need to revalidate now, we just created it
                    if template.revalidates_with_time() {
                        let datetime_to_revalidate =
                            decode_time_str(&template.get_revalidate_interval().unwrap())?;
                        // Write that to a static file, we'll update it every time we revalidate
                        // Note that this runs for every path generated, so it's fully usable with ISR
                        config_manager
                            .write(
                                &format!("static/{}.revld.txt", path_encoded),
                                &datetime_to_revalidate,
                            )
                            .await?;
                    }
                    // Cache all that
                    config_manager
                        .write(
                            &format!("static/{}.json", path_encoded),
                            &state.clone().unwrap(),
                        )
                        .await?;
                    // Write that prerendered HTML to a static file
                    config_manager
                        .write(&format!("static/{}.html", path_encoded), &html_val)
                        .await?;
                    config_manager
                        .write(&format!("static/{}.head.html", path_encoded), &head_val)
                        .await?;

                    states.build_state = state;
                    // Build-time generated HTML is the lowest priority, so we'll only set it if nothing else already has
                    if html.is_empty() {
                        html = html_val;
                        head = head_val;
                    }
                }
            }
        } else {
            // Handle if we need to revalidate
            if should_revalidate(template, &path_encoded, config_manager).await? {
                let (html_val, head_val, state) = revalidate(
                    template,
                    Rc::clone(&translator),
                    path,
                    &path_encoded,
                    config_manager,
                )
                .await?;
                // Build-time generated HTML is the lowest priority, so we'll only set it if nothing else already has
                if html.is_empty() {
                    html = html_val;
                    head = head_val;
                }
                states.build_state = state;
            } else {
                let (html_val, head_val, state) = render_build_state(&path_encoded, config_manager).await?;
                // Build-time generated HTML is the lowest priority, so we'll only set it if nothing else already has
                if html.is_empty() {
                    html = html_val;
                    head = head_val;
                }
                states.build_state = state;
            }
        }
    }
    // Handle request state
    if template.uses_request_state() {
        let (html_val, head_val, state) =
            render_request_state(template, Rc::clone(&translator), path, req).await?;
        // Request-time HTML always overrides anything generated at build-time or incrementally (this has more information)
        html = html_val;
        head = head_val;
        states.request_state = state;
    }

    // Amalgamate the states
    // If the user has defined custom logic for this, we'll defer to that
    // Otherwise we go as with HTML, request trumps build
    // Of course, if only one state was defined, we'll just use that regardless (so `None` prioritization is impossible)
    // If this is the case, the build content will still be served, and then it's up to the client to hydrate it with the new amalgamated state
    let state: Option<String>;
    if !states.both_defined() {
        state = states.get_defined()?;
    } else if template.can_amalgamate_states() {
        state = template.amalgamate_states(states)?;
    } else {
        state = states.request_state;
    }

    // Combine everything into one JSON object
    let res = PageData {
        content: html,
        state,
        head
    };

    Ok(res)
}

/// Gets the HTML/JSON data for the given page path. This will call SSG/SSR/etc., whatever is needed for that page. Note that HTML generated
/// at request-time will **always** replace anything generated at build-time, incrementally, revalidated, etc.
pub async fn get_page(
    // This must not contain the locale
    raw_path: &str,
    locale: &str,
    template_name: &str,
    req: Request,
    templates: &TemplateMap<SsrNode>,
    config_manager: &impl ConfigManager,
    translations_manager: &impl TranslationsManager,
) -> Result<PageData> {
    let mut path = raw_path;
    // If the path is empty, we're looking for the special `index` page
    if path.is_empty() {
        path = "index";
    }
    // Get the template to use
    let template = templates.get(template_name);
    let template = match template {
        Some(template) => template,
        // This shouldn't happen because the client should already have performed checks against the render config, but it's handled anyway
        None => bail!(ErrorKind::PageNotFound(path.to_string())),
    };

    let res = get_page_for_template(
        raw_path,
        locale,
        template,
        req,
        config_manager,
        translations_manager
    )
    .await?;
    Ok(res)
}
