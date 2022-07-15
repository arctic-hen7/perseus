use crate::cmd::{cfg_spinner, run_stage};
use crate::parse::BuildOpts;
use crate::thread::{spawn_thread, ThreadHandle};
use crate::{errors::*, get_user_crate_name};
use console::{style, Emoji};
use indicatif::{MultiProgress, ProgressBar};
use std::env;
use std::path::PathBuf;

// Emojis for stages
static GENERATING: Emoji<'_, '_> = Emoji("🔨", "");
static BUILDING: Emoji<'_, '_> = Emoji("🏗️ ", ""); // Yes, there's a space here, for some reason it's needed...

/// Returns the exit code if it's non-zero.
macro_rules! handle_exit_code {
    ($code:expr) => {
        let (_, _, code) = $code;
        if code != 0 {
            return ::std::result::Result::Ok(code);
        }
    };
}

// /// Finalizes the build by renaming some directories.
// pub fn finalize(target: &Path) -> Result<(), ExecutionError> {
//     // Move the `pkg/` directory into `dist/pkg/`
//     let pkg_dir = target.join("dist/pkg");
//     if pkg_dir.exists() {
//         if let Err(err) = fs::remove_dir_all(&pkg_dir) {
//             return Err(ExecutionError::MovePkgDirFailed { source: err });
//         }
//     }
//     // The `fs::rename()` function will fail on Windows if the destination already exists, so this should work (we've just deleted it as per https://github.com/rust-lang/rust/issues/31301#issuecomment-177117325)
//     if let Err(err) = fs::rename(target.join("pkg"), target.join("dist/pkg"))
// {         return Err(ExecutionError::MovePkgDirFailed { source: err });
//     }

//     Ok(())
// }

/// Actually builds the user's code, program arguments having been interpreted.
/// This needs to know how many steps there are in total because the serving
/// logic also uses it. This also takes a `MultiProgress` to interact with so it
/// can be used truly atomically. This returns handles for waiting on the
/// component threads so we can use it composably.
#[allow(clippy::type_complexity)]
pub fn build_internal(
    dir: PathBuf,
    spinners: &MultiProgress,
    num_steps: u8,
    is_release: bool,
) -> Result<
    (
        ThreadHandle<impl FnOnce() -> Result<i32, ExecutionError>, Result<i32, ExecutionError>>,
        ThreadHandle<impl FnOnce() -> Result<i32, ExecutionError>, Result<i32, ExecutionError>>,
    ),
    ExecutionError,
> {
    let crate_name = get_user_crate_name(&dir)?;
    // Static generation message
    let sg_msg = format!(
        "{} {} Generating your app",
        style(format!("[1/{}]", num_steps)).bold().dim(),
        GENERATING
    );
    // Wasm building message
    let wb_msg = format!(
        "{} {} Building your app to Wasm",
        style(format!("[2/{}]", num_steps)).bold().dim(),
        BUILDING
    );

    // Prepare the optimization flags for the Wasm build (only used in release mode)
    let wasm_opt_flags = if is_release {
        env::var("PERSEUS_WASM_RELEASE_RUSTFLAGS")
            .unwrap_or_else(|_| "-C opt-level=z -C codegen-units=1".to_string())
    } else {
        String::new()
    };

    // We parallelize the first two spinners (static generation and Wasm building)
    // We make sure to add them at the top (the server spinner may have already been
    // instantiated)
    let sg_spinner = spinners.insert(0, ProgressBar::new_spinner());
    let sg_spinner = cfg_spinner(sg_spinner, &sg_msg);
    let sg_dir = dir.clone();
    let wb_spinner = spinners.insert(1, ProgressBar::new_spinner());
    let wb_spinner = cfg_spinner(wb_spinner, &wb_msg);
    let wb_dir = dir;
    let sg_thread = spawn_thread(move || {
        handle_exit_code!(run_stage(
            vec![&format!(
                "{} run {} {}",
                env::var("PERSEUS_CARGO_PATH").unwrap_or_else(|_| "cargo".to_string()),
                if is_release { "--release" } else { "" },
                env::var("PERSEUS_CARGO_ENGINE_ARGS").unwrap_or_else(|_| String::new())
            )],
            &sg_dir,
            &sg_spinner,
            &sg_msg,
            vec![
                ("PERSEUS_ENGINE_OPERATION", "build"),
                ("CARGO_TARGET_DIR", "dist/target_engine")
            ]
        )?);

        Ok(0)
    });
    let wb_thread = spawn_thread(move || {
        handle_exit_code!(run_stage(
            vec![
                // Build the Wasm artifact first (and we know where it will end up, since we're setting the target directory)
                &format!(
                    "{} build --target wasm32-unknown-unknown {} {}",
                    env::var("PERSEUS_CARGO_PATH").unwrap_or_else(|_| "cargo".to_string()),
                    if is_release { "--release" } else { "" },
                    env::var("PERSEUS_CARGO_BROWSER_ARGS").unwrap_or_else(|_| String::new())
                ),
                // NOTE The `wasm-bindgen` version has to be *identical* to the dependency version
                &format!(
                    // TODO Somehow pin output artifact name...
                    "{cmd} ./dist/target_wasm/wasm32-unknown-unknown/{profile}/{crate_name}.wasm --out-dir dist/pkg --out-name perseus_engine --target web {args}",
                    cmd=env::var("PERSEUS_WASM_BINDGEN_PATH").unwrap_or_else(|_| "wasm-bindgen".to_string()),
                    profile={ if is_release { "release" } else { "debug" } },
                    args=env::var("PERSEUS_WASM_BINDGEN_ARGS").unwrap_or_else(|_| String::new()),
                    crate_name=crate_name
                )
            ],
            &wb_dir,
            &wb_spinner,
            &wb_msg,
            if is_release {
                vec![
                    ("CARGO_TARGET_DIR", "dist/target_wasm"),
                    ("RUSTFLAGS", &wasm_opt_flags),
                ]
            } else {
                vec![("CARGO_TARGET_DIR", "dist/target_wasm")]
            }
        )?);

        Ok(0)
    });

    Ok((sg_thread, wb_thread))
}

/// Builds the subcrates to get a directory that we can serve. Returns an exit
/// code.
pub fn build(dir: PathBuf, opts: BuildOpts) -> Result<i32, ExecutionError> {
    let spinners = MultiProgress::new();

    let (sg_thread, wb_thread) = build_internal(dir, &spinners, 2, opts.release)?;
    let sg_res = sg_thread
        .join()
        .map_err(|_| ExecutionError::ThreadWaitFailed)??;
    if sg_res != 0 {
        return Ok(sg_res);
    }
    let wb_res = wb_thread
        .join()
        .map_err(|_| ExecutionError::ThreadWaitFailed)??;
    if wb_res != 0 {
        return Ok(wb_res);
    }

    // This waits for all the threads and lets the spinners draw to the terminal
    // spinners.join().map_err(|_| ErrorKind::ThreadWaitFailed)?;
    // And now we can run the finalization stage
    // finalize(&dir)?;

    // We've handled errors in the component threads, so the exit code is now zero
    Ok(0)
}
