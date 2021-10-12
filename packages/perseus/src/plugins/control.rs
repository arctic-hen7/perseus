use crate::plugins::{PluginAction, PluginData, Runner};
use std::collections::HashMap;

/// An action for a control plugin, which can only be taken by one plugin. When run, control actions will return an `Option<R>` on what
/// their runners return, which will be `None` if no runner is set.
pub struct ControlPluginAction<A, R> {
    /// The name of the plugin that controls this action. As this is a control plugin action, only one plugin can manage a single
    /// action.
    controller_name: String,
    /// The single runner function for this action. This may not be defined if no plugin takes this action.
    runner: Option<Runner<A, R>>,
}
impl<A, R> PluginAction<A, R, Option<R>> for ControlPluginAction<A, R> {
    /// Runs the single registered runner for the action.
    fn run(&self, action_data: A, plugin_data: &HashMap<String, Box<dyn PluginData>>) -> Option<R> {
        // If no runner is defined, this won't have any effect (same as functional actions with no registered runners)
        self.runner.as_ref().map(|runner| {
            runner(
                &action_data,
                // We must have data registered for every active plugin (even if it's empty)
                plugin_data.get(&self.controller_name).unwrap_or_else(|| {
                    panic!(
                        "no plugin data for registered plugin {}",
                        &self.controller_name
                    )
                }),
            )
        })
    }
    fn register_plugin(&mut self, name: String, runner: Runner<A, R>) {
        // Check if the action has already been taken by another plugin
        if self.runner.is_some() {
            // We panic here because an explicitly requested plugin couldn't be loaded, so we have to assume that any further behavior in the engine is unwanted
            // Therefore, a graceful error would be inappropriate, this is critical in every sense
            panic!("attempted to register runner from plugin '{}' for control action that already had a registered runner from plugin '{}' (these plugins conflict, see the book for further details)", name, self.controller_name);
        }

        self.controller_name = name;
        self.runner = Some(runner);
    }
}
// Using a default implementation allows us to avoid the action data having to implement `Default` as well, which is frequently infeasible
impl<A, R> Default for ControlPluginAction<A, R> {
    fn default() -> Self {
        Self {
            controller_name: String::default(),
            runner: None,
        }
    }
}

/// All the actions that a control plugin can perform.
#[derive(Default)]
pub struct ControlPluginActions {
    /// Actions pertaining to the build process.
    pub build_actions: ControlPluginBuildActions,
    /// Actions pertaining to the export process.
    pub export_actions: ControlPluginExportActions,
    /// Actions pertaining to the server.
    pub server_actions: ControlPluginServerActions,
    /// Actions pertaining to the client-side code.
    pub client_actions: ControlPluginClientActions,
}

// TODO add actions

/// The actions a control plugin can take that pertain to the build process.
#[derive(Default)]
pub struct ControlPluginBuildActions {
    /// Gets an immutable store to be used by the build process.
    pub get_immutable_store: ControlPluginAction<(), crate::stores::ImmutableStore>,
}
/// The actions a control plugin can take that pertain to the export process.
#[derive(Default)]
pub struct ControlPluginExportActions {
    /// Gets an immutable store to be used by the export process.
    pub get_immutable_store: ControlPluginAction<(), crate::stores::ImmutableStore>,
    /// Gets the path to the `index.html` file, relative to `.perseus/`. This is the only way to use an alternative path for that.
    pub get_html_shell_path: ControlPluginAction<(), String>,
    /// Gets the path to the directory that stores static files, relative to `.perseus/`. This is the only way to use an alternative
    /// path for that.
    pub get_static_dir_path: ControlPluginAction<(), String>,
}
/// The actions a control plugin can take that pertain to the server.
#[derive(Default)]
pub struct ControlPluginServerActions {
    /// Gets the path to the `index.html` file on the server. Because this may run as a standalone binary, this will be passed a variable
    /// `is_standalone`. If that's `true`, the server binary in not inside `.perseus/`, and you should consider it basically relative
    /// to the project root (but any files outside Perseus' normal purview must be copied manually in deployment).
    pub get_html_shell_path: ControlPluginAction<bool, String>,
    /// Gets the path to the static directory on the server. Because this may run as a standalone binary, this will be passed a variable
    /// `is_standalone`. If that's `true`, the server binary in not inside `.perseus/`, and you should consider it basically relative
    /// to the project root (but any files outside Perseus' normal purview must be copied manually in deployment).
    pub get_static_dir_path: ControlPluginAction<bool, String>,
    /// Gets an immutable store to be used by the server.
    pub get_immutable_store: ControlPluginAction<(), crate::stores::ImmutableStore>,
}
/// The actions a control plugin can take that pertain to the client-side code. As yet, there are none of these.
#[derive(Default)]
pub struct ControlPluginClientActions {}
