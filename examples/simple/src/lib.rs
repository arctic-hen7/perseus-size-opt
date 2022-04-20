use perseus::{ErrorPages, Html, PerseusApp, Plugins, Template};
use sycamore::view;

use perseus_size_opt::{perseus_size_opt, SizeOpts};

#[perseus::main]
pub fn main<G: Html>() -> PerseusApp<G> {
    PerseusApp::new()
        .template(|| {
            Template::new("index").template(|_| {
                view! {
                    p { "Hello World!" }
                }
            })
        })
        .error_pages(|| ErrorPages::new(|url, status, err, _| {
            view! {
                p { (format!("An error with HTTP code {} occurred at '{}': '{}'.", status, url, err)) }
            }
        }))
        .plugins(
            Plugins::new()
                // If you're on Rust 2018, `SizeOpts.default_2018()` is more efficient!
                .plugin(perseus_size_opt, SizeOpts::default())
        )
}
