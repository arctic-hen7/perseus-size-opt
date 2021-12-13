use perseus::{define_app, ErrorPages, Plugins, Template};
use perseus_size_opt::{perseus_size_opt, SizeOpts};
use sycamore::prelude::view;

define_app! {
    templates: [
        Template::<G>::new("index").template(|_| {
            view! {
                p { "Hello World!" }
            }
        })
    ],
    error_pages: ErrorPages::new(|url, status, err, _| {
        view! {
            p { (format!("An error with HTTP code {} occurred at '{}': '{}'.", status, url, err)) }
        }
    }),
    plugins: Plugins::new().plugin(perseus_size_opt, SizeOpts::default())
}
