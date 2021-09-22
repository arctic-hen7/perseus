use crate::errors::*;
use crate::extraction::extract_dir;
#[allow(unused_imports)]
use crate::PERSEUS_VERSION;
use cargo_toml::Manifest;
use include_dir::{include_dir, Dir};
use std::env;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

// This literally includes the entire subcrate in the program, allowing more efficient development.
// This MUST be copied in from `../../examples/cli/.perseus/` every time the CLI is tested (use the Bonnie script).
const SUBCRATES: Dir = include_dir!("./.perseus");

/// Prepares the user's project by copying in the `.perseus/` subcrates. We use these subcrates to do all the building/serving, we just
/// have to execute the right commands in the CLI. We can essentially treat the subcrates themselves as a blackbox of just a folder.
pub fn prepare(dir: PathBuf) -> Result<()> {
    // The location in the target directory at which we'll put the subcrates
    let mut target = dir;
    target.extend([".perseus"]);

    if target.exists() {
        // We don't care if it's corrupted etc., it just has to exist
        // If the user wants to clean it, they can do that
        // Besides, we want them to be able to customize stuff
        Ok(())
    } else {
        // Write the stored directory to that location, creating the directory first
        if let Err(err) = fs::create_dir(&target) {
            bail!(ErrorKind::ExtractionFailed(
                target.to_str().map(|s| s.to_string()),
                err.to_string()
            ))
        }
        // Notably, this function will not do anything or tell us if the directory already exists...
        if let Err(err) = extract_dir(SUBCRATES, &target) {
            bail!(ErrorKind::ExtractionFailed(
                target.to_str().map(|s| s.to_string()),
                err.to_string()
            ))
        }
        // Use the current version of this crate (and thus all Perseus crates) to replace the relative imports
        // That way everything works in dev and in prod on another system!
        // We have to store `Cargo.toml` as `Cargo.toml.old` for packaging
        let mut root_manifest_pkg = target.clone();
        root_manifest_pkg.extend(["Cargo.toml.old"]);
        let mut root_manifest = target.clone();
        root_manifest.extend(["Cargo.toml"]);
        let mut server_manifest_pkg = target.clone();
        server_manifest_pkg.extend(["server", "Cargo.toml.old"]);
        let mut server_manifest = target.clone();
        server_manifest.extend(["server", "Cargo.toml"]);
        let root_manifest_contents = fs::read_to_string(&root_manifest_pkg).map_err(|err| {
            ErrorKind::ManifestUpdateFailed(
                root_manifest_pkg.to_str().map(|s| s.to_string()),
                err.to_string(),
            )
        })?;
        let server_manifest_contents = fs::read_to_string(&server_manifest_pkg).map_err(|err| {
            ErrorKind::ManifestUpdateFailed(
                server_manifest_pkg.to_str().map(|s| s.to_string()),
                err.to_string(),
            )
        })?;
        // Get the name of the user's crate (which the subcrates depend on)
        // We assume they're running this in a folder with a Cargo.toml...
        let user_manifest = Manifest::from_path("./Cargo.toml")
            .map_err(|err| ErrorKind::GetUserManifestFailed(err.to_string()))?;
        let user_crate_name = user_manifest.package;
        let user_crate_name = match user_crate_name {
            Some(package) => package.name,
            None => bail!(ErrorKind::GetUserManifestFailed(
                "no '[package]' section in manifest".to_string()
            )),
        };
        // Update the name of the user's crate (Cargo needs more than just a path and an alias)
        // Also create a workspace so the subcrates share a `target/` directory (speeds up builds)
        let updated_root_manifest = root_manifest_contents
            .replace("perseus-example-basic", &user_crate_name)
            + "\n[workspace]\nmembers = [ \"server\" ]";
        let updated_server_manifest =
            server_manifest_contents.replace("perseus-example-basic", &user_crate_name);

        // If we're not in development, also update relative path references
        #[cfg(not(debug_assertions))]
        let updated_root_manifest = updated_root_manifest.replace(
            "{ path = \"../../../packages/perseus\" }",
            &format!("\"{}\"", PERSEUS_VERSION),
        );
        #[cfg(not(debug_assertions))]
        let updated_server_manifest = updated_server_manifest.replace(
            "{ path = \"../../../../packages/perseus-actix-web\" }",
            &format!("\"{}\"", PERSEUS_VERSION),
        );

        // Write the updated manifests back
        if let Err(err) = fs::write(&root_manifest, updated_root_manifest) {
            bail!(ErrorKind::ManifestUpdateFailed(
                root_manifest.to_str().map(|s| s.to_string()),
                err.to_string()
            ))
        }
        if let Err(err) = fs::write(&server_manifest, updated_server_manifest) {
            bail!(ErrorKind::ManifestUpdateFailed(
                server_manifest.to_str().map(|s| s.to_string()),
                err.to_string()
            ))
        }

        // If we aren't already gitignoring the subcrates, update .gitignore to do so
        if let Ok(contents) = fs::read_to_string(".gitignore") {
            if contents.contains(".perseus/") {
                return Ok(());
            }
        }
        let file = OpenOptions::new()
            .append(true)
            .create(true) // If it doesn't exist, create it
            .open(".gitignore");
        let mut file = match file {
            Ok(file) => file,
            Err(err) => bail!(ErrorKind::GitignoreUpdateFailed(err.to_string())),
        };
        // Check for errors with appending to the file
        if let Err(err) = file.write_all(b"\n.perseus/") {
            bail!(ErrorKind::GitignoreUpdateFailed(err.to_string()))
        }
        Ok(())
    }
}

/// Checks if the user has the necessary prerequisites on their system (i.e. `cargo` and `wasm-pack`). These can all be checked
/// by just trying to run their binaries and looking for errors. If the user has other paths for these, they can define them under the
/// environment variables `PERSEUS_CARGO_PATH` and `PERSEUS_WASM_PACK_PATH`.
pub fn check_env() -> Result<()> {
    // We'll loop through each prerequisite executable to check their existence
    // If the spawn returns an error, it's considered not present, success means presence
    let prereq_execs = vec![
        (
            env::var("PERSEUS_CARGO_PATH").unwrap_or_else(|_| "cargo".to_string()),
            "cargo",
            "PERSEUS_CARGO_PATH",
        ),
        (
            env::var("PERSEUS_WASM_PACK_PATH").unwrap_or_else(|_| "wasm-pack".to_string()),
            "wasm-pack",
            "PERSEUS_WASM_PACK_PATH",
        ),
    ];

    for exec in prereq_execs {
        let res = Command::new(&exec.0).output();
        // Any errors are interpreted as meaning that the user doesn't have the prerequisite installed properly.
        if let Err(err) = res {
            bail!(ErrorKind::PrereqFailed(
                exec.1.to_string(),
                exec.2.to_string(),
                err.to_string()
            ))
        }
    }

    Ok(())
}
