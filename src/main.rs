mod app;

use app::*;
use leptos::prelude::*;

fn main() {
    console_error_panic_hook::set_once();
    if let Some(document) = web_sys::window().and_then(|window| window.document()) {
        if let Some(boot_screen) = document.get_element_by_id("boot-screen") {
            boot_screen.remove();
        }
    }
    mount_to_body(|| {
        view! {
            <App/>
        }
    })
}
