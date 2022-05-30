# Architecture

Perseus has several main components:

-   `perseus` -- the core module that defines everything necessary to build a Perseus app if you try hard enough
-   `perseus-actix-web` -- an integration that makes it easy to run Perseus on the [Actix Web](https://actix.rs) framework
-   `perseus-warp` -- an integration that makes it easy to run Perseus on the [Warp](https://github.com/seanmonstar/warp) framework
-   `perseus-cli` -- the command-line interface used to run Perseus apps conveniently
-   `perseus-engine` -- an internal crate created by the CLI responsible for building an app
-   `perseus-engine-server` -- an internal crate created by the CLI responsible for serving an app and performing runtime logic

## Core

At the core of Perseus is the [`perseus`](https://docs.rs/perseus) module, which is used for nearly everything in Perseus. In theory, you could build a fully-functional app based on this crate alone, but you'd be reinventing the wheel at least three times. This crate exposes types for the i18n systems, configuration management, routing, and asset fetching, most of which aren't intended to be used directly by the user.

What is intended to be used directly is the `Template<G>` `struct`, which is integral to Perseus. This stores closures for every rendering strategy, which are executed as provided and necessary at build and runtime. Note that these are all stored in `Rc`s, and `Template<G>`s are cloned.

The other commonly used system from this crate is the `Translator` system, explained in detail in [the i18n section](:i18n/intro). `Translator`s are passed around in `Rc`s, and `TranslationsManager` on the server caches all translations by default in memory on the server.

## Server Integrations

The core of Perseus provides very few systems to set up a functional Perseus server though, which requires a significant amount of additional work. To this end, server integration crates are used to make this process easy. If you've ejected, you'll be working with these directly, which should be relatively simple, as they just accept configuration options and then should simply work.

## CLI

As documented in [this section](:cli), the CLI simply runs commands to execute the last two components of the Perseus system, acting as a convenience. It also contains these two components inside its binary (using [`include_dir!`](https://github.com/Michael-F-Bryan/include_dir)).

### CLI Builder

This system can be further broken down into two parts.

#### Static Generator

This is a single binary that just imports the user's templates and some other information (like locales) and then calls `build_app`. This will result in generating a number of files to `.perseus/dist/`, which will be served by the server to any clients, which will then hydrate those static pages into fully-fledged Sycamore templates.

#### App Shell

This is encapsulated in `.perseus/src/lib.rs`, and it performs a number of integral functions:

-   Ensures that any `panic!`s or the like ar printed properly in the browser console
-   Creates and manages the internal router
-   Renders your actual app
-   Handles locale detection
-   Invokes the core app shell to manage initial/subsequent loads and translations
-   Handles error page displaying

### CLI Server

This is just an invocation of the the appropriate server integration's systems with the data provided by the user through `PerseusApp`.
