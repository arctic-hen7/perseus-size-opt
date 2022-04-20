# Perseus Size Optimization Plugin

> Note: users of v0.1.0-v0.1.3 should upgrade to v0.1.4 if they're using Rust 2021, as an upstream bug occurs if you attempt to compile your app in release mode. See [here](#compiling-in-release-mode-never-finishes) for further details.

This is a very simple plugin for [Perseus](https://arctic-hen7.github.io/perseus) that applies size optimizations automatically, which
decrease the size of your final Wasm bundle significantly, meaning faster loads for users because a smaller amount of data needs to be
transferred to their browsers. Because Perseus renders a page almost immediately through static generation, and then the Wasm bundle is
needed to make that page interactive, applying this plugin will decrease your app's time-to-interactive and its total blocking time
(a Lighthouse metric that normal Perseus apps don't do so well in on mobile).

If you're new to Perseus, check it out [on it's website](https://arctic-hen7.github.io/perseus) and [on GitHub](https://github.com/arctic-hen7/perseus)! Basically though, it's a really fast and fully-featured web framework for Rust!

## Usage

In your `src/lib.rs`, call the following function on `PerseusApp`:

``` rust
PerseusApp::new()
    [...]
    .plugins(Plugins::new()
        .plugin(
            perseus_size_opt,
            SizeOpts::default()
        ))
```

<details>
<summary>I'm using `define_app!`</summary>

If you're still using `define_app!` from v0.3.3, you should upgrade to using `PerseusApp` soon, but you can still use this plugin by adding the following to the bottom of the `define_app!` call:

```rust
plugins: Plugins::new().plugin(perseus_size_opt, SizeOpts::default())
```

</details>

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

## Known Bugs

### Compiling in release mode never finishes

[This](https://github.com/arctic-hen7/perseus/issues/83) is due to an upstream issue in Rust 2021 that leads to size optimizations of the `fluent-bundle` package causing an overload of LLVM (see [this issue](https://github.com/rust-lang/rust/issues/91011)). As of v0.1.4, this plugin accounts for this and does not attempt to optimize `fluent-bundle` with the `default()` options. However, this will increase bundle size, so it's recommended that, until this upstream issue is fixed, users of this plugin remain on Rust 2018 for now and use the `default_2018()` function instead. This will optimize `fluent-bundle` appropriately while ensuring that this bug does not occur. If moving to Rust 2018 is infeasible, you'll have to put up with slightly larger bundles for now until the upstream issue is fixed (this seems to depend on the LLVM team now).

## Stability

This plugin is considered quite stable due to how basic its optimizations are (the whole thing is one file), and so its stability is mostly dependent on that of Perseus. If you're happy to use Perseus, you shouldn't need to worry about using this plugin as well (in fact, it's recommended that all Perseus apps use this plugin).

## License

See [`LICENSE`](./LICENSE).
