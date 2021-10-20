# Perseus Size Optimization Plugin

> WARNING: Until [Perseus #66](https://github.com/arctic-hen7/perseus/issues/66) is fixed, this plugin can actually _increase_ overall binary size! Once that issue is fixed though, it should have the desired effect.

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

## License

See [`LICENSE`](./LICENSE).
