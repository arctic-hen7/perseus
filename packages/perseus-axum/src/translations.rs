use axum::{extract::Path, http::StatusCode, Extension};
use fmterr::fmt_err;
use perseus::internal::{i18n::TranslationsManager, serve::ServerOptions};
use std::sync::Arc;

pub async fn translations_handler<T: TranslationsManager>(
    Path(locale): Path<String>,
    Extension(opts): Extension<Arc<ServerOptions>>,
    Extension(translations_manager): Extension<Arc<T>>,
) -> (StatusCode, String) {
    // Check if the locale is supported
    if opts.locales.is_supported(&locale) {
        // We know that the locale is supported, so any failure to get translations is a 500
        let translations = translations_manager
            .get_translations_str_for_locale(locale.to_string())
            .await;
        let translations = match translations {
            Ok(translations) => translations,
            Err(err) => return (StatusCode::INTERNAL_SERVER_ERROR, fmt_err(&err)),
        };

        (StatusCode::OK, translations)
    } else {
        (StatusCode::NOT_FOUND, "locale not supported".to_string())
    }
}
