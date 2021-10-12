mod error_pages;
mod templates;

use perseus::define_app;
use perseus::plugins::*;

fn get_test_plugin() -> Plugin {
    Plugin {
        name: "test-plugin".to_string(),
        plugin_type: PluginType::Functional,
        functional_actions_registrar: Box::new(|mut actions| {
            actions
                .build_actions
                .after_successful_build
                .register_plugin(
                    "test-plugin".to_string(),
                    Box::new(|_, _| {
                        dbg!("Hello from a plugin!");
                    }),
                );
            actions
        }),
    }
}

define_app! {
    templates: [
        crate::templates::index::get_template::<G>(),
        crate::templates::about::get_template::<G>()
    ],
    error_pages: crate::error_pages::get_error_pages(),
    static_aliases: {
        "/test.txt" => "static/test.txt"
    },
    plugins: Plugins::new()
        .plugin(get_test_plugin(), ())
}
