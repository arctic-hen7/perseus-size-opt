/*!
This is a very simple plugin for [Perseus](https://arctic-hen7.github.io/perseus) that applies size optimizations automatically, which
decrease the size of your final Wasm bundle significantly, meaning faster loads for users because a smaller amount of data needs to be
transferred to their browsers. Because Perseus renders a page almost immediately through static generation, and then the Wasm bundle is
needed to make that page interactive, applying this plugin will decrease your app's time-to-interactive and its total blocking time
(a Lighthouse metric that normal Perseus apps don't do so well in on mobile).

If you're new to Perseus, check it out [on it's website](https://arctic-hen7.github.io/perseus) and [on GitHub](https://github.com/arctic-hen7/perseus)! Basically though, it's a really fast and fully-featured web framework for Rust!

## Usage

In your `src/lib.rs`, add the following to the bottom of the `define_app!` macro:

```rust
plugins: Plugins::new().plugin(perseus_size_opt(), SizeOpts::default())
```

If you have any other plugins defined, add the `.plugin()` call where appropriate. You'll also need to add the following imports:

```rust
use perseus_size_opt::{perseus_size_opt, SizeOpts};
```

Once that's done, run `perseus tinker` to apply the optimizations (this needs a separate command because they involve modifying the `.perseus/` directory), and then you can work with your app as normal!

If you ever want to uninstall the plugin, just remove the relevant `.plugin()` call and re-run `perseus tinker`, and it'll be completely removed.

## Optimizations

This plugin currently performs the following optimizations:

- `wee_alloc` -- an alternative allocator designed for Wasm that reduces binary size at the expense of slightly slower allocations
- `lto` -- reduces binary size when set to `true` by enabling link-time optimizations
- `opt-level` -- optimizes aggressively for binary size when set to `z`
- `codegen-units` -- makes faster and smaller code when set to lower values (but makes compile times slower in release mode)

Note that all optimizations will only apply to release builds. except for the use of `wee_alloc`, which will also affect development builds.

## Options

There are a few defaults available for setting size optimization levels, or you can build your own custom settings easily by constructing
`SizeOpts` manually.

- `::default()` -- enables all optimizations
- `::default_no_lto()` -- enables all optimizations except `lto = true`, because that can break compilation of execution on some hosting providers, like Netlify.
- `::only_wee_alloc()` -- only uses `wee_alloc`, applying no other optimizations
- `::no_wee_alloc()` -- applies all optimizations other than `wee_alloc`

## Stability

This plugin is considered quite stable due to how basic its optimizations are (the whole thing is one file), and so its stability is mostly dependent on that of Perseus. If you're happy to use Perseus, you shouldn't need to worry about using this plugin as well (in fact, it's recommended that all Perseus apps use this plugin).
*/

use cargo_toml::{Dependency, Manifest, Profile, Value};
use perseus::plugins::{empty_control_actions_registrar, Plugin, PluginAction, PluginEnv};
use perseus::Html;
use std::collections::BTreeMap;
use std::fs;
use thiserror::Error;
use toml::value::Map;

const PLUGIN_NAME: &str = "perseus-size-opt";
const WEE_ALLOC_DEF: &str = "#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;";
// This will need updating over time
const WEE_ALLOC_VERSION: &str = "0.4";

/// Options for size optimizations. Note that these settings will only affect release builds (e.g. when you run `perseus deploy`), not
/// development builds (except using `wee_alloc`, whcih will affect everything).
pub struct SizeOpts {
    /// Whether or not to use `wee_alloc`, which reduces binary size significantly. This defaults to `true`. Note that this also makes
    /// allocations slightly slower, so you'll need to decide what tradeoff between size and speed you want your app to have.
    pub wee_alloc: bool,
    /// Whether or not to use link-time optimizations, which defaults to `true`. If your app isn't recognized by providers like Netlify,
    /// disable this.
    pub lto: bool,
    /// The optimization level to use, which defaults to `z` (aggressively optimize for speed). Cargo typically sets this to `3` for
    /// release builds.
    pub opt_level: String,
    /// The value for the `codegen-units` property, which is set by default to 1 by this plugin. Higher values here will mean faster
    /// compile times but slower code. The Rust default is 16 (256 with incremental builds).
    pub codegen_units: u16,
    /// Whether or not to enable the patch for `fluent-bundle` that fixes Perseus #83 (compiling taking forever with size optimizations). If you're running
    /// Rust 2021, you should enable this until [this upstream issue](https://github.com/rust-lang/rust/issues/91011) is fixed.
    pub enable_fluent_bundle_patch: bool,
}
impl Default for SizeOpts {
    fn default() -> Self {
        Self {
            wee_alloc: true,
            lto: true,
            opt_level: "z".to_string(),
            codegen_units: 1,
            enable_fluent_bundle_patch: true,
        }
    }
}
// We add a few more sensible named defaults
impl SizeOpts {
    /// The usual default, but without the `fluent-bundle` patch. Use this for greater size reductions if you're not using Rust 2021.
    pub fn default_2018() -> Self {
        Self {
            wee_alloc: true,
            lto: true,
            opt_level: "z".to_string(),
            codegen_units: 1,
            enable_fluent_bundle_patch: false,
        }
    }
    /// The usual default, but without `lto` enabled, which is known to cause problems on some hosting services like Netlify. If your
    /// app runs out of memory during compilation, or won't be served properly, try this.
    pub fn default_no_lto() -> Self {
        Self {
            wee_alloc: true,
            lto: false,
            opt_level: "z".to_string(),
            codegen_units: 1,
            enable_fluent_bundle_patch: true,
        }
    }
    /// Only enables the alternative allocator `wee_alloc`, with no further additional optimizations made.
    pub fn only_wee_alloc() -> Self {
        Self {
            wee_alloc: true,
            lto: false,
            opt_level: "3".to_string(),
            codegen_units: 16,
            enable_fluent_bundle_patch: true,
        }
    }
    /// Enables all optimizations other than changing the default allocator.
    pub fn no_wee_alloc() -> Self {
        Self {
            wee_alloc: false,
            lto: true,
            opt_level: "z".to_string(),
            codegen_units: 1,
            enable_fluent_bundle_patch: true,
        }
    }
}

/// The errors that this plugin can return.
#[derive(Error, Debug)]
pub enum Error {
    #[error("couldn't get and parse `.perseus/Cargo.toml`, try running `perseus tinker` again (without the `--no-clean` option)")]
    GetManifestFailed {
        #[source]
        source: cargo_toml::Error,
    },
    #[error("couldn't update `.perseus/Cargo.toml`, try running `perseus tinker` again (without the `--no-clean` option)")]
    WriteManifestFailed {
        #[source]
        source: std::io::Error,
    },
    #[error("couldn't read `.perseus/src/lib.rs`, try running `perseus tinker` again (without the `--no-clean` option)")]
    ReadLibFailed {
        #[source]
        source: std::io::Error,
    },
    #[error("couldn't update `.perseus/src/lib.rs`, try running `perseus tinker` again (without the `--no-clean` option)")]
    WriteLibFailed {
        #[source]
        source: std::io::Error,
    },
}

/// The actual mechanics of this plugin, which apply size optimizations to `.perseus/Cargo.toml` and `.perseus/src/lib.rs`.
fn apply_size_opts(opts: &SizeOpts) -> Result<(), Error> {
    // Get the internal `Cargo.toml` file in `.perseus/` (the current directory)
    let mut manifest = Manifest::from_path("Cargo.toml")
        .map_err(|err| Error::GetManifestFailed { source: err })?;
    // Apply size optimizations to the release profile
    let mut release_profile = manifest.profile.release.unwrap_or(
        // Because `Default` is not implemented for this `struct`...
        Profile {
            opt_level: None,
            lto: None,
            debug: None,
            rpath: None,
            debug_assertions: None,
            codegen_units: None,
            panic: None,
            incremental: None,
            overflow_checks: None,
            package: std::collections::BTreeMap::default(),
            build_override: None,
        },
    );
    release_profile.opt_level = Some(opts.opt_level.clone().into());
    release_profile.lto = Some(opts.lto.into());
    release_profile.codegen_units = Some(opts.codegen_units); // If the `fluent-bundle` patch is enabled, apply it
                                                              // TODO Remove this patch entirely once the upstream issue in LLVM is fixed and the error no longer occurs
    if opts.enable_fluent_bundle_patch {
        let mut fluent_bundle_conf = Map::new();
        fluent_bundle_conf.insert("opt-level".to_string(), Value::Integer(2));
        let mut patch = BTreeMap::new();
        patch.insert(
            "fluent-bundle".to_string(),
            Value::Table(fluent_bundle_conf),
        );
        release_profile.package = patch;
    }
    manifest.profile.release = Some(release_profile);
    // Add `wee_alloc` as a dependency if we're using that optimization
    if opts.wee_alloc {
        // We override any existing versions of `wee_alloc`, the user can disable that optimization if they're doing more advanced stuff
        manifest.dependencies.insert(
            "wee_alloc".to_string(),
            Dependency::Simple(WEE_ALLOC_VERSION.to_string()),
        );
    }
    // For some reason, our modifications to the manifest wipe out the `cdylib` definition (TODO investigate this further), so we need to add it back
    let mut manifest_lib = manifest.lib.unwrap_or_default();
    manifest_lib.crate_type = Some(vec!["cdylib".to_string(), "rlib".to_string()]);
    manifest.lib = Some(manifest_lib);

    // Write the new manifest
    // This will result in a ton of previously implied stuff being written, so the manifest will get a lot longer (fine because it's internal)
    let manifest_str = toml::to_string(&manifest).unwrap();
    fs::write("Cargo.toml", manifest_str)
        .map_err(|err| Error::WriteManifestFailed { source: err })?;

    if opts.wee_alloc {
        // Again, this is inside `.perseus/`, we're modifying the engine, not the user's code
        let lib_contents =
            fs::read_to_string("src/lib.rs").map_err(|err| Error::ReadLibFailed { source: err })?;
        // Prepend the new allocator definition to the file
        let lib_contents_with_wee_alloc = format!("{}\n{}", WEE_ALLOC_DEF, lib_contents);

        fs::write("src/lib.rs", lib_contents_with_wee_alloc)
            .map_err(|err| Error::WriteLibFailed { source: err })?;
    }

    Ok(())
}

/// Gets the plugin itself to be handed to Perseus' `define_app!` macro. Note that this plugin's optimizations will only take effect
/// in release mode (e.g. when you run `perseus deploy`).
pub fn perseus_size_opt<G: Html>() -> Plugin<G, SizeOpts> {
    Plugin::new(
        PLUGIN_NAME,
        |mut actions| {
            actions
                .tinker
                .register_plugin(PLUGIN_NAME, |_, plugin_data| {
                    if let Some(plugin_data) = plugin_data.downcast_ref::<SizeOpts>() {
                        let res = apply_size_opts(plugin_data);
                        if let Err(err) = res {
                            panic!("error in `perseus-size-opt`: {}", err);
                        }
                    } else {
                        unreachable!();
                    }
                });
            actions
        },
        empty_control_actions_registrar,
        // This plugin only needs to run at tinker-time, otherwise it increases binary sizes
        PluginEnv::Server,
    )
}
