use perseus::plugins::*;
use perseus::Template;

#[derive(Debug)]
pub struct TestPluginData {
    pub about_page_greeting: String,
}

pub fn get_test_plugin<G: perseus::GenericNode>() -> Plugin<G, TestPluginData> {
    Plugin::new(
        "test-plugin",
        |mut actions| {
            actions
                .settings_actions
                .add_static_aliases
                .register_plugin("test-plugin", |_, _| {
                    let mut map = std::collections::HashMap::new();
                    map.insert("/Cargo.toml".to_string(), "Cargo.toml".to_string());
                    map
                });
            actions.settings_actions.add_templates.register_plugin(
                "test-plugin",
                |_, plugin_data| {
                    if let Some(plugin_data) = plugin_data.downcast_ref::<TestPluginData>() {
                        let about_page_greeting = plugin_data.about_page_greeting.to_string();
                        vec![Template::new("about")
                        .template(move |_| sycamore::template! { p { (about_page_greeting) } })
                        .head(|_| {
                            sycamore::template! {
                                title { "About Page (Plugin Modified) | Perseus Example – Plugins" }
                            }
                        })]
                    } else {
                        unreachable!()
                    }
                },
            );
            actions.tinker.register_plugin("test-plugin", |_, _| {
                println!("{:?}", std::env::current_dir().unwrap())
            });
            actions
        },
        empty_control_actions_registrar,
    )
}
