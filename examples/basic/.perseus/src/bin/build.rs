use app::{
    get_immutable_store, get_locales, get_mutable_store, get_plugins, get_templates_vec,
    get_translations_manager,
};
use futures::executor::block_on;
use perseus::{build_app, plugins::PluginAction, SsrNode};
use std::rc::Rc;

fn main() {
    let exit_code = real_main();
    std::process::exit(exit_code)
}

fn real_main() -> i32 {
    let plugins = Rc::new(get_plugins());

    plugins
        .functional_actions
        .build_actions
        .before_build
        .run((), plugins.get_plugin_data());

    let immutable_store = plugins
        .control_actions
        .build_actions
        .get_immutable_store
        .run((), plugins.get_plugin_data())
        .unwrap_or_else(get_immutable_store);
    let mutable_store = get_mutable_store();
    let translations_manager = block_on(get_translations_manager());
    let locales = get_locales();

    // Build the site for all the common locales (done in parallel)
    // All these parameters can be modified by `define_app!` and plugins, so there's no point in having a plugin opportunity here
    let fut = build_app(
        get_templates_vec::<SsrNode>(),
        &locales,
        (&immutable_store, &mutable_store),
        &translations_manager,
        // We use another binary to handle exporting
        false,
    );
    let res = block_on(fut);
    if let Err(err) = res {
        let err_msg = format!("Static generation failed: '{}'.", &err);
        plugins
            .functional_actions
            .build_actions
            .after_failed_build
            .run(err, plugins.get_plugin_data());
        eprintln!("{}", err_msg);
        1
    } else {
        plugins
            .functional_actions
            .build_actions
            .after_successful_build
            .run((), plugins.get_plugin_data());
        println!("Static generation successfully completed!");
        0
    }
}
