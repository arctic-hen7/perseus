# Basic Example

This is a basic example of Perseus that shows the fundamentals of a slightly more advanced Perseus app. This is considered a core example because it not only contains end-to-end tests for Perseus itself, it's also the site of the development of the default Perseus engine. For that reason, the `.perseus/` directory is checked into Git here, and this is used as the single source of truth for the default engine. The reason for developing it here is to provide the context of an actual usage of Perseus, which makes a number of things easier. Then, some scripts bridge the gap to make the engine integrate into the CLI (you shouldn't have to worry about this when working in the Perseus repo as long as you're using the `bonnie dev example ...` script).

Note that this example used to be more complex, illustrating features such as setting custom headers and hosting static content, though demonstrations of these have since been moved to independent examples.
