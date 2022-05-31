#![allow(missing_docs)] // Prevents double-documenting some things

use crate::PERSEUS_VERSION;
use clap::Parser;

// The documentation for the `Opts` struct will appear in the help page, hence the lack of puncutation and the lowercasing in places

/// The command-line interface for Perseus, a super-fast WebAssembly frontend development framework!
#[derive(Parser)]
#[clap(version = PERSEUS_VERSION)]
// #[clap(setting = AppSettings::ColoredHelp)]
pub struct Opts {
    /// The URL of a Git repository to clone to provide a custom engine. If this is set to `default`, the normal Perseus engine (packaged with the CLI) will be used. A branch name can be added at
    /// the end of this after `@` to specify a custom branch/version
    #[clap(short, long, default_value = "default")]
    pub engine: String,
    #[clap(subcommand)]
    pub subcmd: Subcommand,
}

#[derive(Parser, PartialEq, Eq, Clone)]
pub enum Integration {
    ActixWeb,
    Warp,
    Axum,
}
// We use an `enum` for this so we don't get errors from Cargo about non-existent feature flags, overly verbose but fails quickly
impl std::str::FromStr for Integration {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "actix-web" => Ok(Self::ActixWeb),
            "warp" => Ok(Self::Warp),
            "axum" => Ok(Self::Axum),
            _ => Err("invalid integration name".into()),
        }
    }
}
impl ToString for Integration {
    fn to_string(&self) -> String {
        match self {
            Self::ActixWeb => "actix-web".to_string(),
            Self::Warp => "warp".to_string(),
            Self::Axum => "axum".to_string(),
        }
    }
}

#[derive(Parser)]
pub enum Subcommand {
    Build(BuildOpts),
    ExportErrorPage(ExportErrorPageOpts),
    Export(ExportOpts),
    Serve(ServeOpts),
    /// Serves your app as `perseus serve` does, but puts it in testing mode
    Test(ServeOpts),
    Clean(CleanOpts),
    /// Ejects you from the CLI harness, enabling you to work with the internals of Perseus
    Eject,
    Deploy(DeployOpts),
    /// Prepares the `.perseus/` directory (done automatically by `build` and `serve`)
    Prep,
    Tinker(TinkerOpts),
    /// Runs one of the underlying commands that builds your app, allowing you to see more detailed logs
    #[clap(subcommand)]
    Snoop(SnoopSubcommand),
}
/// Builds your app
#[derive(Parser)]
pub struct BuildOpts {
    /// Build for production
    #[clap(long)]
    pub release: bool,
}
/// Exports your app to purely static files
#[derive(Parser, Clone)]
pub struct ExportOpts {
    /// Export for production
    #[clap(long)]
    pub release: bool,
    /// Serve the generated static files locally
    #[clap(short, long)]
    pub serve: bool,
    /// Where to host your exported app
    #[clap(long, default_value = "127.0.0.1")]
    pub host: String,
    /// The port to host your exported app on
    #[clap(long, default_value = "8080")]
    pub port: u16,
    /// Watch the files in your working directory for changes (exluding `target/` and `.perseus/`)
    #[clap(short, long)]
    pub watch: bool,
}
/// Exports an error page for the given HTTP status code
#[derive(Parser, Clone)]
pub struct ExportErrorPageOpts {
    #[clap(short, long)]
    pub code: String,
    #[clap(short, long)]
    pub output: String,
}
/// Serves your app (set the `$HOST` and `$PORT` environment variables to change the location it's served at)
#[derive(Parser, Clone)]
pub struct ServeOpts {
    /// Don't run the final binary, but print its location instead as the last line of output
    #[clap(long)]
    pub no_run: bool,
    /// Only build the server, and use the results of a previous `perseus build`
    #[clap(long)]
    pub no_build: bool,
    /// Build and serve for production
    #[clap(long)]
    pub release: bool,
    /// Make the final binary standalone (this is used in `perseus deploy` only, don't manually invoke it unless you have a good reason!)
    #[clap(long)]
    pub standalone: bool,
    /// The server integration to use
    #[clap(short, long, default_value = "warp")]
    pub integration: Integration,
    /// Watch the files in your working directory for changes (exluding `target/` and `.perseus/`)
    #[clap(short, long)]
    pub watch: bool,
    /// Where to host your exported app
    #[clap(long, default_value = "127.0.0.1")]
    pub host: String,
    /// The port to host your exported app on
    #[clap(long, default_value = "8080")]
    pub port: u16,
}
/// Removes `.perseus/` entirely for updates or to fix corruptions
#[derive(Parser)]
pub struct CleanOpts {
    /// Only remove the `.perseus/dist/` folder (use if you've ejected)
    #[clap(short, long)]
    pub dist: bool,
    /// Remove the directory, even if you've ejected (this will permanently destroy any changes you've made to `.perseus/`!)
    #[clap(short, long)]
    pub force: bool,
}
/// Packages your app for deployment
#[derive(Parser)]
pub struct DeployOpts {
    /// Change the output from `pkg/` to somewhere else
    #[clap(short, long, default_value = "pkg")]
    pub output: String,
    /// Export you app to purely static files (see `export`)
    #[clap(short, long)]
    pub export_static: bool,
    /// The server integration to use (only affects non-exported deployments)
    #[clap(short, long, default_value = "warp")]
    pub integration: Integration,
}
/// Runs the `tinker` action of plugins, which lets them modify the Perseus engine
#[derive(Parser)]
pub struct TinkerOpts {
    /// Don't remove and recreate the `.perseus/` directory
    #[clap(long)]
    pub no_clean: bool,
    /// Force this command to run, even if you've ejected (this may result in some or all of your changes being removed, it depends on the plugins you're using)
    #[clap(long)]
    pub force: bool,
}

#[derive(Parser)]
pub enum SnoopSubcommand {
    /// Snoops on the static generation process (this will let you see `dbg!` calls and the like)
    Build,
    /// Snoops on the Wasm building process (mostly for debugging errors)
    WasmBuild(SnoopWasmOpts),
    /// Snoops on the server process (run `perseus build` before this)
    Serve(SnoopServeOpts),
}

#[derive(Parser)]
pub struct SnoopWasmOpts {
    /// Produce a profiling build (for use with `twiggy` and the like)
    #[clap(short, long)]
    pub profiling: bool,
}

#[derive(Parser)]
pub struct SnoopServeOpts {
    /// The server integration to use
    #[clap(short, long, default_value = "warp")]
    pub integration: Integration,
    /// Where to host your exported app
    #[clap(long, default_value = "127.0.0.1")]
    pub host: String,
    /// The port to host your exported app on
    #[clap(long, default_value = "8080")]
    pub port: u16,
}
