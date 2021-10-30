use perseus::{define_app, Plugins, ErrorPages, Template};
use perseus_size_opt::{perseus_size_opt, SizeOpts};
use sycamore::template;

define_app! {
    templates: [
        Template::<G>::new("index").template(|_| {
            template! {
                p { "Hello World!" }
            }
        })
    ],
    error_pages: ErrorPages::new(|url, status, err, _| {
        template! {
            p { (format!("An error with HTTP code {} occurred at '{}': '{}'.", status, url, err)) }
        }
    }),
    plugins: Plugins::new().plugin(perseus_size_opt, SizeOpts::default())
}
