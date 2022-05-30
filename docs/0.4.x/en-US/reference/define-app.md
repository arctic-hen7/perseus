# `define_app!`

Perseus used to be configured through a macro rather than through `PerseusApp`: `define_app!`. For now, this is still supported, but it will be removed in the next major release. If you're still using `define_app!`, you should switch to `PerseusApp` when possible. Note also that `define_app!` is now simply a wrapper for `PerseusApp`.

The smallest this can reasonably get is a fully self-contained app (taken from [here](https://github.com/arctic-hen7/perseus/tree/main/examples/comprehensive/tiny/src/lib.rs)):

```rust
{{#include ../../../examples/comprehensive/tiny/src/lib.rs}}
```

In a more complex app though, this macro still remains very manageable (taken from [here](https://github.com/arctic-hen7/perseus/tree/main/examples/core/state_generation/src/lib.rs)):

```rust
{{#include ../../../examples/core/state_generation/src/lib.rs}}
```

## Parameters

Here's a list of everything you can provide to the macro and what each one does (note that the order of these matters):

-   `root` (optional) -- the HTML `id` to which your app will be rendered, the default is `root`; this MUST be reflected in your `index.html` file as an exact replication (spacing and all) of `<div id="root-id-here"></div>` (replacing `root-id-here` with the value of this property)
-   `templates` -- defines a list of your templates in which order is irrelevant
-   `error_pages` -- defines an instance of `ErrorPages`, which tells Perseus what to do on an error (like a _404 Not Found_)
-   `locales` (optional) -- defines options for i18n (internationalization), this shouldn't be specified for apps not using i18n
    -   `default` -- the default locale of your app (e.g. `en-US`)
    -   `other` -- a list of the other locales your app supports
-   `static_aliases` (optional) -- a list of aliases to static files in your project (e.g. for a favicon)
-   `plugins` (optional) -- a list of plugins to add to extend Perseus (see [here](:reference/plugins/intro))
-   `dist_path` (optional) -- a custom path to distribution artifacts (this is relative to `.perseus/`!)
-   `mutable_store` (optional) -- a custom mutable store
-   `translations_manager` (optional) -- a custom translations manager

**WARNING:** if you try to include something from outside the current directory in `static_aliases`, **no part of your app will load**! If you could include such content, you might end up serving `/etc/passwd`, which would be a major security risk.

## Other Files

There's only one other file that the `define_app!` macro expects to exist: `index.html`. Note that any content in the `<head>` of this will be on every page, above anything inserted by the template.

Here's an example of this file (taken from [here](https://github.com/arctic-hen7/perseus/blob/main/examples/core/basic/index.html)):

```html
{{#include ../../../examples/core/basic/index.html}}
```
