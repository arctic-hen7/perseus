/// An internal macro used for defining a function to get the user's preferred immutable store (which requires multiple branches).
#[doc(hidden)]
#[macro_export]
macro_rules! define_get_immutable_store {
    () => {
        pub fn get_immutable_store() -> $crate::stores::ImmutableStore {
            // This will be executed in the context of the user's directory, but moved into `.perseus`
            // If we're in prod mode on the server though, this is fine too
            $crate::stores::ImmutableStore::new("./dist".to_string())
        }
    };
    ($dist_path:literal) => {
        pub fn get_immutable_store() -> $crate::stores::ImmutableStore {
            $crate::stores::ImmutableStore::new($dist_path.to_string())
        }
    };
}
/// An internal macro used for defining a function to get the user's preferred mutable store (which requires multiple branches).
#[doc(hidden)]
#[macro_export]
macro_rules! define_get_mutable_store {
    () => {
        pub fn get_mutable_store() -> impl $crate::stores::MutableStore {
            // This will be executed in the context of the user's directory, but moved into `.perseus`
            // If we're in prod mode on the server though, this is fine too
            // Note that this is separated out from the immutable store deliberately
            $crate::stores::FsMutableStore::new("./dist/mutable".to_string())
        }
    };
    ($mutable_store:expr) => {
        pub fn get_mutable_store() -> impl $crate::stores::MutableStore {
            $mutable_store
        }
    };
}
/// An internal macro used for defining the HTML `id` at which to render the Perseus app (which requires multiple branches). The default
/// is `root`.
#[doc(hidden)]
#[macro_export]
macro_rules! define_app_root {
    () => {
        pub static APP_ROOT: &str = "root";
    };
    ($root_id:literal) => {
        pub static APP_ROOT: &str = $root_id;
    };
}
/// An internal macro used for defining a function to get the user's preferred translations manager (which requires multiple branches).
#[doc(hidden)]
#[macro_export]
macro_rules! define_get_translations_manager {
    ($locales:expr) => {
        pub async fn get_translations_manager() -> impl $crate::TranslationsManager {
            // This will be executed in the context of the user's directory, but moved into `.perseus`
            // Note that `translations/` must be next to `src/`, not within it
            // By default, all translations are cached
            let all_locales: Vec<String> = $locales
                .get_all()
                .iter()
                // We have a `&&String` at this point, hence the double clone
                .cloned()
                .cloned()
                .collect();
            // If we're running on a server, we should be using a flattened directory structure
            let translations_dir = if ::std::env::var("PERSEUS_STANDALONE").is_ok() {
                "./translations"
            } else {
                "../translations"
            };
            $crate::FsTranslationsManager::new(
                translations_dir.to_string(),
                all_locales,
                $crate::TRANSLATOR_FILE_EXT.to_string(),
                $crate::path_prefix::get_path_prefix_server(),
            )
            .await
        }
    };
    ($locales:expr, $no_i18n:literal) => {
        pub async fn get_translations_manager() -> impl $crate::TranslationsManager {
            $crate::translations_manager::DummyTranslationsManager::new()
        }
    };
    ($locales:expr, $translations_manager:expr) => {
        pub async fn get_translations_manager() -> impl $crate::TranslationsManager {
            $translations_manager
        }
    };
    // If the user doesn't want i18n but also sets their own transations manager, the latter takes priority
    ($locales:expr, $no_i18n:literal, $translations_manager:expr) => {
        pub async fn get_translations_manager() -> impl $crate::TranslationsManager {
            $translations_manager
        }
    };
}
/// An internal macro used for defining locales data. This is abstracted because it needs multiple branches.
#[doc(hidden)]
#[macro_export]
macro_rules! define_get_locales {
    {
        default: $default_locale:literal,
        other: [$($other_locale:literal),*]
    } => {
        pub fn get_locales() -> $crate::Locales {
            $crate::Locales {
                default: $default_locale.to_string(),
                other: vec![
                    $($other_locale.to_string()),*
                ],
                using_i18n: true
            }
        }
    };
    // With i18n disabled, the default locale will be `xx-XX`
    {
        default: $default_locale:literal,
        other: [$($other_locale:literal),*],
        no_i18n: $no_i18n:literal
    } => {
        pub fn get_locales() -> $crate::Locales {
            $crate::Locales {
                default: "xx-XX".to_string(),
                other: Vec::new(),
                using_i18n: false
            }
        }
    };
}
/// An internal macro for defining a function that gets the user's static content aliases (abstracted because it needs multiple
/// branches).
#[doc(hidden)]
#[macro_export]
macro_rules! define_get_static_aliases {
    (
        static_aliases: {
            $($url:literal => $resource:literal)*
        }
    ) => {
        pub fn get_static_aliases() -> ::std::collections::HashMap<String, String> {
            let mut static_aliases = ::std::collections::HashMap::new();
            $(
                let resource = $resource.to_string();
                // We need to move this from being scoped to the app to being scoped for `.perseus/`
                // TODO make sure this works properly on Windows
                let resource = if resource.starts_with("/") {
                    // Absolute paths are a security risk and are disallowed
                    panic!("it's a security risk to include absolute paths in `static_aliases`");
                }  else if resource.starts_with("../") {
                    // Anything outside this directory is a security risk as well
                    panic!("it's a security risk to include paths outside the current directory in `static_aliases`");
                } else if resource.starts_with("./") {
                    // `./` -> `../` (moving to execution from `.perseus/`)
                    // But if we're operating standalone, it stays the same
                    if ::std::env::var("PERSEUS_STANDALONE").is_ok() {
                        resource
                    } else {
                        format!(".{}", resource)
                    }
                } else {
                    // Anything else gets a `../` prepended
                    // But if we're operating standalone, it stays the same
                    if ::std::env::var("PERSEUS_STANDALONE").is_ok() {
                        resource
                    } else {
                        format!("../{}", resource)
                    }
                };
                static_aliases.insert($url.to_string(), resource);
            )*
            static_aliases
        }
    };
    () => {
        pub fn get_static_aliases() -> ::std::collections::HashMap<String, String> {
            ::std::collections::HashMap::new()
        }
    };
}
/// An internal macro used for defining the plugins for an app.
#[doc(hidden)]
#[macro_export]
macro_rules! define_plugins {
    () => {
        pub fn get_plugins() -> $crate::plugins::Plugins {
            $crate::plugins::Plugins::new()
        }
    };
    ($plugins:expr) => {
        pub fn get_plugins() -> $crate::plugins::Plugins {
            $plugins
        }
    };
}

/// Defines the components to create an entrypoint for the app. The actual entrypoint is created in the `.perseus/` crate (where we can
/// get all the dependencies without driving the user's `Cargo.toml` nuts). This also defines the template map. This is intended to make
/// compatibility with the Perseus CLI significantly easier.
///
/// Warning: all properties must currently be in the correct order (`root`, `templates`, `error_pages`, `locales`, `static_aliases`,
/// `dist_path`, `mutable_store`, `translations_manager`).
#[macro_export]
macro_rules! define_app {
    // With locales
    {
        $(root: $root_selector:literal,)?
        templates: [
            $($template:expr),+
        ],
        error_pages: $error_pages:expr,
        // This deliberately enforces verbose i18n definition, and forces developers to consider i18n as integral
        locales: {
            default: $default_locale:literal,
            // The user doesn't have to define any other locales
            other: [$($other_locale:literal),*]
        }
        $(,static_aliases: {
            $($url:literal => $resource:literal)*
        })?
        $(,plugins: $plugins:expr)?
        $(,dist_path: $dist_path:literal)?
        $(,mutable_store: $mutable_store:expr)?
        $(,translations_manager: $translations_manager:expr)?
    } => {
        $crate::define_app!(
            @define_app,
            {
                $(root: $root_selector,)?
                templates: [
                    $($template),+
                ],
                error_pages: $error_pages,
                locales: {
                    default: $default_locale,
                    // The user doesn't have to define any other locales (but they'll still get locale detection and the like)
                    other: [$($other_locale),*]
                }
                $(,static_aliases: {
                    $($url => $resource)*
                })?
                $(,plugins: $plugins)?
                $(,dist_path: $dist_path)?
                $(,mutable_store: $mutable_store)?
                $(,translations_manager: $translations_manager)?
            }
        );
    };
    // Without locales (default locale is set to xx-XX)
    {
        $(root: $root_selector:literal,)?
        templates: [
            $($template:expr),+
        ],
        error_pages: $error_pages:expr
        $(,static_aliases: {
            $($url:literal => $resource:literal)*
        })?
        $(,plugins: $plugins:expr)?
        $(,dist_path: $dist_path:literal)?
        $(,mutable_store: $mutable_store:expr)?
    } => {
        $crate::define_app!(
            @define_app,
            {
                $(root: $root_selector,)?
                templates: [
                    $($template),+
                ],
                error_pages: $error_pages,
                // This deliberately enforces verbose i18n definition, and forces developers to consider i18n as integral
                locales: {
                    default: "xx-XX",
                    other: [],
                    no_i18n: true
                }
                $(,static_aliases: {
                    $($url => $resource)*
                })?
                $(,plugins: $plugins)?
                $(,dist_path: $dist_path)?
                $(,mutable_store: $mutable_store)?
            }
        );
    };
    // This is internal, and allows syntax abstractions and defaults
    (
        @define_app,
        {
            $(root: $root_selector:literal,)?
            templates: [
                $($template:expr),+
            ],
            error_pages: $error_pages:expr,
            // This deliberately enforces verbose i18n definition, and forces developers to consider i18n as integral
            locales: {
                default: $default_locale:literal,
                // The user doesn't have to define any other locales
                other: [$($other_locale:literal),*]
                // If this is defined at all, i18n will be disabled and the default locale will be set to `xx-XX`
                $(,no_i18n: $no_i18n:literal)?
            }
            $(,static_aliases: {
                $($url:literal => $resource:literal)*
            })?
            $(,plugins: $plugins:expr)?
            $(,dist_path: $dist_path:literal)?
            $(,mutable_store: $mutable_store:expr)?
            $(,translations_manager: $translations_manager:expr)?
        }
    ) => {
        /// The html `id` that will find the app root to render Perseus in. For server-side interpolation, this MUST be an element of
        /// the form <div id="root_id">` in your markup (double or single quotes, `root_id` replaced by what this property is set to).
        $crate::define_app_root!($($root_selector)?);

        /// Gets the immutable store to use. This allows the user to conveniently change the path of distribution artifacts.
        $crate::define_get_immutable_store!($($dist_path)?);
        /// Gets the mutable store to use. This allows the user to conveniently substitute the default filesystem store for another
        /// one in development and production.
        $crate::define_get_mutable_store!($($mutable_store)?);

        /// Gets the translations manager to use. This allows the user to conveniently test production managers in development. If
        /// nothing is given, the filesystem will be used.
        $crate::define_get_translations_manager!(get_locales() $(, $no_i18n)? $(, $translations_manager)?);

        /// Defines the locales the app should build for, specifying defaults and common locales (which will be built at build-time
        /// rather than on-demand).
        $crate::define_get_locales! {
            default: $default_locale,
            other: [
                $($other_locale),*
            ]
            $(, no_i18n: $no_i18n)?
        }

        /// Gets the plugins for the app.
       $crate::define_plugins!($($plugins)?);

        /// Gets a map of all the templates in the app by their root paths.
        pub fn get_templates_map<G: $crate::GenericNode>() -> $crate::TemplateMap<G> {
            $crate::get_templates_map![
                $($template),+
            ]
        }

        /// Gets a list of all the templates in the app in the order the user provided them.
        pub fn get_templates_vec<G: $crate::GenericNode>() -> Vec<$crate::Template<G>> {
            vec![
                $($template),+
            ]
        }

        /// Gets the error pages (done here so the user doesn't have to worry about naming).
        pub fn get_error_pages<G: $crate::GenericNode>() -> $crate::ErrorPages<G> {
            $error_pages
        }

        /// Gets any static content aliases provided by the user.
        $crate::define_get_static_aliases!(
            $(static_aliases: {
                $($url => $resource)*
            })?
        );
    };
}
