use app::{get_config_manager, get_locales, get_templates_vec, get_translations_manager};
use futures::executor::block_on;
use perseus::{build_app, SsrNode};

fn main() {
    let exit_code = real_main();
    std::process::exit(exit_code)
}

fn real_main() -> i32 {
    let config_manager = get_config_manager();
    let translations_manager = block_on(get_translations_manager());
    let locales = get_locales();

    // Build the site for all the common locales (done in parallel)
    let fut = build_app(
        get_templates_vec::<SsrNode>(),
        &locales,
        &config_manager,
        &translations_manager,
        // We use another binary to handle exporting
        false,
    );
    let res = block_on(fut);
    if let Err(err) = res {
        eprintln!("Static generation failed: '{}'.", err);
        1
    } else {
        println!("Static generation successfully completed!");
        0
    }
}
