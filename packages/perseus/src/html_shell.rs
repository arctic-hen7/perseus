use crate::page_data::PageData;
use std::collections::HashMap;
use std::env;

const INTERPOLATED_HEAD_MARKER: &str = "<!-- PERSEUS_INTERPOLATED_HEAD_BEGINS -->";

struct HtmlShell {
    inner: String,
}

impl HtmlShell {
    /// Initializes the HTML shell by interpolating necessary scripts into it, as well as by adding the render configuration.
    fn new(shell: &str, render_cfg: &HashMap<String, String>, path_prefix: &str) -> Self {
        let set_testing_flag = if env::var("PERSEUS_TESTING").is_ok() {
            "window.__PERSEUS_TESTING = true;"
        } else {
            ""
        };

        // Here is the list of things that we are adding *right after* the beginning of the <head> tag:
        //
        // - Render config as a global variable.
        //
        // - Global variable to identify whether we are testing (picked up by app shell to trigger helper DOM events).
        //
        // - A `<base>` tag with the base path of the app, which allows us to serve at relative paths just from an environment variable.
        //
        //      This one has to be at the very top of the <head> element so that it applies to everything in the HTML shell.
        //      Otherwise any stylesheets loaded before it won't work properly.
        //
        //      Also add a trailing `/` to the path_prefix (https://stackoverflow.com/a/26043021);
        //      note that it's already had any pre-existing ones stripped away.
        //
        let script = format!(
            r#"
        <base href="{path_prefix}/" />
        <script type="module">
            window.__PERSEUS_RENDER_CFG = '{render_config}';
            {set_testing_flag}
    
            import init, {{ run }} from "{path_prefix}/.perseus/bundle.js";
            async function main() {{
                await init("{path_prefix}/.perseus/bundle.wasm");
                run();
            }}
            main();
        </script>
        "#,
            path_prefix = path_prefix,
            render_config = serde_json::to_string(render_cfg).unwrap(),
            set_testing_flag = set_testing_flag,
        );

        let shell = shell.replace("<head>", &format!("<head>{}", script));

        // Append a delimiter comment that will be used to wall off the constant document head from the interpolated document head.
        let shell = shell.replace("</head>", &format!("{}</head>", INTERPOLATED_HEAD_MARKER));
        Self { inner: shell }
    }

    fn interpolate_page_data(self, page_data: &PageData, root_id: &str) {
        let html_with_head = self.inner.replace(
            INTERPOLATED_HEAD_MARKER,
            &format!("{0}{1}", INTERPOLATED_HEAD_MARKER, &page_data.head),
        );

        // Interpolate a global variable of the state so the app shell doesn't have to make any more trips
        // The app shell will unset this after usage so it doesn't contaminate later non-initial loads
        // Error pages (above) will set this to `error`
        let state_var = format!("<script>window.__PERSEUS_INITIAL_STATE = `{}`;</script>", {
            if let Some(state) = &page_data.state {
                state
                    // We escape any backslashes to prevent their interfering with JSON delimiters
                    .replace(r#"\"#, r#"\\"#)
                    // We escape any backticks, which would interfere with JS's raw strings system
                    .replace(r#"`"#, r#"\`"#)
                    // We escape any interpolations into JS's raw string system
                    .replace(r#"${"#, r#"\${"#)
            } else {
                "None".to_string()
            }
        });

        let html_with_state = html_with_head.replace("</head>", &format!("{}\n</head>", state_var));
    }
}

/// Interpolates content, metadata, and state into the HTML shell, ready to be sent to the user for initial loads. This should be passed
/// an HTMl shell prepared with `prep_html_shell`. This also takes the HTML `id` of the element in the shell to interpolate content
/// into.
pub fn interpolate_page_data(html_shell: &str, page_data: &PageData, root_id: &str) -> String {
    // Interpolate the document `<head>`
    let html_with_head = html_shell.replace(
        "<!--PERSEUS_INTERPOLATED_HEAD_BEGINS-->",
        &format!("<!--PERSEUS_INTERPOLATED_HEAD_BEGINS-->{}", &page_data.head),
    );

    // Interpolate a global variable of the state so the app shell doesn't have to make any more trips
    // The app shell will unset this after usage so it doesn't contaminate later non-initial loads
    // Error pages (above) will set this to `error`
    let state_var = format!("<script>window.__PERSEUS_INITIAL_STATE = `{}`;</script>", {
        if let Some(state) = &page_data.state {
            state
                // We escape any backslashes to prevent their interfering with JSON delimiters
                .replace(r#"\"#, r#"\\"#)
                // We escape any backticks, which would interfere with JS's raw strings system
                .replace(r#"`"#, r#"\`"#)
                // We escape any interpolations into JS's raw string system
                .replace(r#"${"#, r#"\${"#)
        } else {
            "None".to_string()
        }
    });
    // We put this at the very end of the head (after the delimiter comment) because it doesn't matter if it's expunged on subsequent loads
    let html_with_state = html_with_head.replace("</head>", &format!("{}\n</head>", state_var));

    // Figure out exactly what we're interpolating in terms of content
    // The user MUST place have a `<div>` of this exact form (documented explicitly)
    // We permit either double or single quotes
    let html_to_replace_double = format!("<div id=\"{}\">", root_id);
    let html_to_replace_single = format!("<div id='{}'>", root_id);
    let html_replacement = format!(
        // We give the content a specific ID so that it can be deleted if an error page needs to be rendered on the client-side
        "{}<div id=\"__perseus_content_initial\" class=\"__perseus_content\">{}</div>",
        &html_to_replace_double,
        &page_data.content
    );
    // Now interpolate that HTML into the HTML shell
    html_with_state
        .replace(&html_to_replace_double, &html_replacement)
        .replace(&html_to_replace_single, &html_replacement)
}

/// Intepolates a fallback for locale redirection pages such that, even if JavaScript is disabled, the user will still be redirected to the default locale.
/// From there, Perseus' inbuilt progressive enhancement can occur, but without this a user directed to an unlocalized page with JS disabled would see a
/// blank screen, which is terrible UX. Note that this also includes a fallback for if JS is enabled but Wasm is disabled. Note that the redirect URL
/// is expected to be generated with a path prefix inbuilt.
///
/// This also adds a `__perseus_initial_state` `<div>` in case it's needed (for Wasm redirections).
pub fn interpolate_locale_redirection_fallback(
    html_shell: &str,
    redirect_url: &str,
    root_id: &str,
) -> String {
    // This will be used if JavaScript is completely disabled (it's then the site's responsibility to show a further message)
    let dumb_redirector = format!(
        r#"<noscript>
    <meta http-equiv="refresh" content="0; url={}" />
</noscript>"#,
        redirect_url
    );
    // This will be used if JS is enabled, but Wasm is disabled or not supported (it's then the site's responsibility to show a further message)
    // Wasm support detector courtesy https://stackoverflow.com/a/47880734
    let js_redirector = format!(
        r#"<script>
    function wasmSupported() {{
        try {{
            if (typeof WebAssembly === "object"
                && typeof WebAssembly.instantiate === "function") {{
                const module = new WebAssembly.Module(Uint8Array.of(0x0, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00));
                if (module instanceof WebAssembly.Module) {{
                    return new WebAssembly.Instance(module) instanceof WebAssembly.Instance;
                }}
            }}
        }} catch (e) {{}}
        return false;
    }}

    if (!wasmSupported()) {{
        window.location.replace("{}");
    }}
</script>"#,
        redirect_url
    );

    let html = html_shell.replace(
        "</head>",
        &format!("{}\n{}\n</head>", js_redirector, dumb_redirector),
    );

    // The user MUST place have a `<div>` of this exact form (documented explicitly)
    // We permit either double or single quotes
    let html_to_replace_double = format!("<div id=\"{}\">", root_id);
    let html_to_replace_single = format!("<div id='{}'>", root_id);
    let html_replacement = format!(
        // We give the content a specific ID so that it can be deleted if an error page needs to be rendered on the client-side
        "{}<div id=\"__perseus_content_initial\" class=\"__perseus_content\"></div>",
        &html_to_replace_double,
    );
    // Now interpolate that HTML into the HTML shell
    html.replace(&html_to_replace_double, &html_replacement)
        .replace(&html_to_replace_single, &html_replacement)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::prep_html_shell;

    const SHELL: &str = r#"
<html>
    <head>
        <title>Shell</title>
    </head>
    <body>
        <p>Content</p>
    </body>
</html>
"#;

    #[test]
    fn some_example() {
        let expected = "\n<html>\n    <head>\n<base href=\"prefix/\" />\n        <title>Shell</title>\n    <script>window.__PERSEUS_RENDER_CFG = '{}';</script>\n<script type=\"module\">\n    import init, { run } from \"prefix/.perseus/bundle.js\";\n    async function main() {\n        await init(\"prefix/.perseus/bundle.wasm\");\n        run();\n    }\n    main();\n</script>\n<!--PERSEUS_INTERPOLATED_HEAD_BEGINS-->\n</head>\n    <body>\n        <p>Content</p>\n    </body>\n</html>\n";
        let result = prep_html_shell(String::from(SHELL), &HashMap::new(), "prefix");

        assert_eq!(result, expected);
    }
}
