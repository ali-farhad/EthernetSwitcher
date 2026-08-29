use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(catch, js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct EthernetAdapter {
    guid: String,
    name: String,
    description: String,
    status: String,
    link_speed: String,
    mac_address: String,
    ipv4_address: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SwitchArgs<'a> {
    adapter_guid: &'a str,
}

fn error_text(value: JsValue) -> String {
    value
        .as_string()
        .or_else(|| {
            js_sys::Reflect::get(&value, &JsValue::from_str("message"))
                .ok()
                .and_then(|message| message.as_string())
        })
        .unwrap_or_else(|| "Windows returned an unexpected error.".to_owned())
}

async fn fetch_adapters() -> Result<Vec<EthernetAdapter>, String> {
    let value = invoke("list_ethernet_adapters", js_sys::Object::new().into())
        .await
        .map_err(error_text)?;
    serde_wasm_bindgen::from_value(value)
        .map_err(|error| format!("Could not understand the adapter list: {error}"))
}

async fn reload(
    set_adapters: WriteSignal<Vec<EthernetAdapter>>,
    set_loading: WriteSignal<bool>,
    set_error: WriteSignal<Option<String>>,
) {
    set_loading.set(true);
    set_error.set(None);
    match fetch_adapters().await {
        Ok(value) => set_adapters.set(value),
        Err(error) => set_error.set(Some(error)),
    }
    set_loading.set(false);
}

#[component]
pub fn App() -> impl IntoView {
    let (adapters, set_adapters) = signal(Vec::<EthernetAdapter>::new());
    let (loading, set_loading) = signal(true);
    let (switching, set_switching) = signal(false);
    let (error, set_error) = signal(None::<String>);
    let (notice, set_notice) = signal(None::<String>);
    let (search, set_search) = signal(String::new());
    let (pending, set_pending) = signal(None::<EthernetAdapter>);

    Effect::new(move |_| {
        spawn_local(reload(set_adapters, set_loading, set_error));
    });

    let filtered = Memo::new(move |_| {
        let query = search.get().to_lowercase();
        adapters
            .get()
            .into_iter()
            .filter(|adapter| {
                query.is_empty()
                    || adapter.name.to_lowercase().contains(&query)
                    || adapter.description.to_lowercase().contains(&query)
                    || adapter
                        .ipv4_address
                        .as_deref()
                        .unwrap_or_default()
                        .contains(&query)
            })
            .collect::<Vec<_>>()
    });

    let active_count = move || {
        adapters
            .get()
            .iter()
            .filter(|adapter| adapter.status.eq_ignore_ascii_case("up"))
            .count()
    };

    view! {
        <div class="app-shell">
            <div class="ambient ambient-one"></div>
            <div class="ambient ambient-two"></div>

            <header class="topbar">
                <a class="brand" href="#" aria-label="Ethernet Switcher home">
                    <span class="brand-mark">
                        <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M7 8V5h10v3M5 11v6h4v-6H5Zm10 0v6h4v-6h-4ZM10 11v6h4v-6h-4ZM12 8v3M7 8h10"/></svg>
                    </span>
                    <span><strong>"Ethernet"</strong><small>"SWITCHER"</small></span>
                </a>
                <div class="system-state">
                    <span class="pulse"></span>
                    <span>{move || if active_count() > 0 { "Network online" } else { "No active link" }}</span>
                </div>
            </header>

            <main>
                <section class="hero">
                    <div>
                        <p class="eyebrow">"WINDOWS NETWORK CONTROL"</p>
                        <h1>"Your wired networks,"<br/><em>"one click away."</em></h1>
                        <p class="intro">"Choose a physical Ethernet adapter. Windows will enable it and safely disable the others."</p>
                    </div>
                    <div class="hero-stat">
                        <span>{move || adapters.get().len()}</span>
                        <p>"ADAPTERS"<br/>"FOUND"</p>
                    </div>
                </section>

                <section class="control-panel">
                    <div class="panel-heading">
                        <div>
                            <h2>"Available connections"</h2>
                            <p>"Physical Ethernet adapters detected by Windows"</p>
                        </div>
                        <div class="tools">
                            <label class="search">
                                <svg viewBox="0 0 24 24" aria-hidden="true"><circle cx="11" cy="11" r="7"/><path d="m20 20-4-4"/></svg>
                                <input
                                    type="search"
                                    placeholder="Search adapters"
                                    aria-label="Search adapters"
                                    on:input=move |event| set_search.set(event_target_value(&event))
                                />
                            </label>
                            <button
                                class="icon-button"
                                title="Refresh adapters"
                                aria-label="Refresh adapters"
                                disabled=move || loading.get()
                                on:click=move |_| spawn_local(reload(set_adapters, set_loading, set_error))
                            >
                                <svg class:spinning=move || loading.get() viewBox="0 0 24 24" aria-hidden="true"><path d="M20 12a8 8 0 1 1-2.34-5.66L20 8"/><path d="M20 3v5h-5"/></svg>
                            </button>
                        </div>
                    </div>

                    {move || error.get().map(|message| view! {
                        <div class="alert error-alert" role="alert">
                            <span>"!"</span><div><strong>"Couldn’t load adapters"</strong><p>{message}</p></div>
                        </div>
                    })}

                    {move || notice.get().map(|message| view! {
                        <div class="alert success-alert" role="status">
                            <span>"✓"</span><div><strong>"Connection switched"</strong><p>{message}</p></div>
                            <button aria-label="Dismiss" on:click=move |_| set_notice.set(None)>"×"</button>
                        </div>
                    })}

                    <div class="adapter-list" aria-live="polite">
                        {move || if loading.get() && adapters.get().is_empty() {
                            view! { <div class="empty-state"><span class="loader"></span><h3>"Finding Ethernet adapters…"</h3><p>"Asking Windows for physical network interfaces"</p></div> }.into_any()
                        } else if filtered.get().is_empty() {
                            view! {
                                <div class="empty-state">
                                    <div class="empty-icon">"⌁"</div>
                                    <h3>{move || if search.get().is_empty() { "No Ethernet adapters found" } else { "No matching adapters" }}</h3>
                                    <p>{move || if search.get().is_empty() { "Connect a wired network adapter, then refresh." } else { "Try a different name or IP address." }}</p>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <For
                                    each=move || filtered.get()
                                    key=|adapter| adapter.guid.clone()
                                    children=move |adapter| {
                                        let is_active = adapter.status.eq_ignore_ascii_case("up");
                                        let is_exclusive = is_active
                                            && adapters
                                                .get_untracked()
                                                .iter()
                                                .filter(|item| item.status.eq_ignore_ascii_case("up"))
                                                .count()
                                                == 1;
                                        let adapter_for_click = adapter.clone();
                                        view! {
                                            <article class="adapter-card" class:active=is_active>
                                                <div class="adapter-icon">
                                                    <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M6 4h12v7l-3 3H9l-3-3V4Z"/><path d="M9 4v4m3-4v4m3-4v4M8 17h8m-4-3v3"/></svg>
                                                </div>
                                                <div class="adapter-main">
                                                    <div class="adapter-title">
                                                        <h3>{adapter.name.clone()}</h3>
                                                        <span class="status" class:online=is_active><i></i>{if is_active { "Connected" } else if adapter.status == "Disabled" { "Disabled" } else { "Disconnected" }}</span>
                                                    </div>
                                                    <p class="description">{adapter.description.clone()}</p>
                                                    <div class="metadata">
                                                        <span><small>"IP ADDRESS"</small>{adapter.ipv4_address.clone().unwrap_or_else(|| "Not assigned".to_owned())}</span>
                                                        <span><small>"LINK SPEED"</small>{adapter.link_speed.clone()}</span>
                                                        <span><small>"MAC"</small>{adapter.mac_address.clone()}</span>
                                                    </div>
                                                </div>
                                                <button
                                                    class="switch-button"
                                                    class:current=is_exclusive
                                                    disabled=move || is_exclusive || switching.get()
                                                    on:click=move |_| set_pending.set(Some(adapter_for_click.clone()))
                                                >
                                                    {if is_exclusive { "Active" } else if is_active { "Use only" } else { "Switch" }}
                                                    {(!is_exclusive).then(|| view! { <span>"→"</span> })}
                                                </button>
                                            </article>
                                        }
                                    }
                                />
                            }.into_any()
                        }}
                    </div>
                </section>

                <footer><span>"A local Windows utility"</span><span>"No network data leaves this device"</span></footer>
            </main>

            {move || pending.get().map(|adapter| {
                let adapter_name = adapter.name.clone();
                let guid = adapter.guid.clone();
                view! {
                    <div class="modal-backdrop" role="presentation">
                        <div class="modal" role="dialog" aria-modal="true" aria-labelledby="confirm-title">
                            <div class="modal-icon">"↗"</div>
                            <p class="eyebrow">"CONFIRM SWITCH"</p>
                            <h2 id="confirm-title">"Use "{adapter.name.clone()}"?"</h2>
                            <p>"This enables the selected adapter and disables all other physical Ethernet adapters. Your connection may pause briefly."</p>
                            <div class="modal-actions">
                                <button class="secondary" disabled=move || switching.get() on:click=move |_| set_pending.set(None)>"Cancel"</button>
                                <button class="primary" disabled=move || switching.get() on:click=move |_| {
                                    let guid = guid.clone();
                                    let adapter_name = adapter_name.clone();
                                    spawn_local(async move {
                                        set_switching.set(true);
                                        set_error.set(None);
                                        let args = serde_wasm_bindgen::to_value(&SwitchArgs { adapter_guid: &guid }).unwrap();
                                        match invoke("switch_adapter", args).await {
                                            Ok(_) => {
                                                set_pending.set(None);
                                                set_notice.set(Some(format!("{} is now the preferred wired connection.", adapter_name)));
                                                reload(set_adapters, set_loading, set_error).await;
                                            }
                                            Err(value) => {
                                                set_pending.set(None);
                                                set_error.set(Some(error_text(value)));
                                            }
                                        }
                                        set_switching.set(false);
                                    });
                                }>
                                    {move || if switching.get() { "Switching…" } else { "Switch connection" }}
                                </button>
                            </div>
                        </div>
                    </div>
                }
            })}
        </div>
    }
}
