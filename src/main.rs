use dioxus::prelude::*;

fn main() {
    let cfg = dioxus::desktop::Config::new()
        .with_window(
            dioxus::desktop::WindowBuilder::new()
                .with_title("IP Finder")
        );
    dioxus::LaunchBuilder::desktop().with_cfg(cfg).launch(App);
}

fn extract_hostname(url: &str) -> String {
    let stripped = if url.contains("://") {
        url.split("://").nth(1).unwrap_or(url)
    } else {
        url
    };
    stripped
        .split('/')
        .next()
        .unwrap_or(stripped)
        .split(':')
        .next()
        .unwrap_or(stripped)
        .trim()
        .to_string()
}

#[component]
fn App() -> Element {
    let mut url_input = use_signal(|| String::new());
    let mut site_ipv6 = use_signal(|| Option::<String>::None);
    let mut site_ipv4 = use_signal(|| Option::<String>::None);
    let mut device_ip = use_signal(|| String::from("Fetching…"));
    let mut loading = use_signal(|| false);
    let mut error_msg = use_signal(|| String::new());

    // Fetch device public IP on mount
    use_effect(move || {
        spawn(async move {
            match reqwest::get("https://ifconfig.me/ip").await {
                Ok(resp) => match resp.text().await {
                    Ok(ip) => device_ip.set(ip.trim().to_string()),
                    Err(_) => device_ip.set("Could not read response".to_string()),
                },
                Err(_) => device_ip.set("Could not reach ifconfig.me".to_string()),
            }
        });
    });

    // Shared lookup logic
    let mut do_lookup = move || {
        let url = url_input.read().clone();
        if url.trim().is_empty() {
            error_msg.set("Please enter a URL or hostname.".to_string());
            return;
        }
        spawn(async move {
            loading.set(true);
            error_msg.set(String::new());
            site_ipv6.set(None);

            let hostname = extract_hostname(&url);
            let addr = format!("{}:80", hostname);

            site_ipv4.set(None);
            site_ipv6.set(None);

            let hostname = extract_hostname(&url);
            let addr = format!("{}:80", hostname);

            let result = tokio::task::spawn_blocking(move || {
                use std::net::ToSocketAddrs;
                addr.to_socket_addrs().map(|iter| {
                    let mut v4 = None;
                    let mut v6 = None;
                    for a in iter {
                        if a.is_ipv4() && v4.is_none() { v4 = Some(a.ip().to_string()); }
                        if a.is_ipv6() && v6.is_none() { v6 = Some(a.ip().to_string()); }
                    }
                    (v4, v6)
                })
            })
            .await;

            match result {
                Ok(Ok((v4, v6))) => {
                    if v4.is_none() && v6.is_none() {
                        error_msg.set(format!("No IP found for \"{}\"", hostname));
                    }
                    site_ipv4.set(v4);
                    site_ipv6.set(v6);
                }
                Ok(Err(e)) => error_msg.set(format!("DNS lookup failed: {}", e)),
                Err(e) => error_msg.set(format!("Task error: {}", e)),
            }

            loading.set(false);
        });
    };

    let on_click = move |_: MouseEvent| {
        do_lookup();
    };

    let on_key = move |evt: KeyboardEvent| {
        if evt.key() == Key::Enter {
            do_lookup();
        }
    };

    rsx! {
        style { {STYLES} }

        div { class: "container",

            h1 { class: "title", "IP Finder" }

            // Device IP card
            div { class: "card device-card",
                div { class: "card-label", "Your Device's Public IP" }
                div { class: "card-value", "{device_ip}" }
            }

            // Site lookup section
            div { class: "lookup-section",
                div { class: "card-label", "Look Up a Site's IP" }
                div { class: "input-row",
                    input {
                        class: "url-input",
                        r#type: "text",
                        placeholder: "e.g. wpmudev.com or https://wpmudev.com",
                        value: "{url_input}",
                        oninput: move |e| url_input.set(e.value()),
                        onkeydown: on_key,
                    }
                    button {
                        class: "lookup-btn",
                        disabled: *loading.read(),
                        onclick: on_click,
                        if *loading.read() {
                            "Looking up…"
                        } else {
                            "Look Up"
                        }
                    }
                }
            }

            // Error
            if !error_msg.read().is_empty() {
                div { class: "card error-card",
                    div { class: "card-label", "Error" }
                    div { class: "card-value error-value", "{error_msg}" }
                }
            }

            // Result
            if site_ipv4.read().is_some() || site_ipv6.read().is_some() {
                div { class: "card result-card",
                    if let Some(ip) = site_ipv4.read().clone() {
                        div { class: "card-label", "IPv4" }
                        div { class: "card-value result-value", "{ip}" }
                    }
                    if let Some(ip) = site_ipv6.read().clone() {
                        div { class: "card-label", style: "margin-top: 12px", "IPv6" }
                        div { class: "card-value result-value", "{ip}" }
                    }
                    div { class: "card-hint", "Resolved from: {extract_hostname(&url_input.read())}" }
                }
            }
        }
    }
}

const STYLES: &str = r#"
* { box-sizing: border-box; margin: 0; padding: 0; }

body {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
    background: #0f1117;
    color: #e2e8f0;
    min-height: 100vh;
}

.container {
    max-width: 540px;
    margin: 0 auto;
    padding: 48px 24px;
    display: flex;
    flex-direction: column;
    gap: 20px;
}

.title {
    font-size: 28px;
    font-weight: 700;
    color: #f8fafc;
    margin-bottom: 8px;
}

.card {
    background: #1e2330;
    border: 1px solid #2d3448;
    border-radius: 12px;
    padding: 20px 24px;
}

.device-card {
    border-color: #3b4fd8;
    background: #1a1f3a;
}

.card-label {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: #64748b;
    margin-bottom: 8px;
}

.card-value {
    font-size: 22px;
    font-weight: 600;
    font-family: "SF Mono", "Fira Code", monospace;
    color: #7c9dff;
}

.card-hint {
    font-size: 12px;
    color: #475569;
    margin-top: 8px;
    font-family: "SF Mono", "Fira Code", monospace;
}

.lookup-section {
    display: flex;
    flex-direction: column;
    gap: 10px;
}

.input-row {
    display: flex;
    gap: 10px;
}

.url-input {
    flex: 1;
    background: #1e2330;
    border: 1px solid #2d3448;
    border-radius: 8px;
    padding: 12px 16px;
    font-size: 14px;
    color: #e2e8f0;
    outline: none;
    transition: border-color 0.15s;
}

.url-input:focus {
    border-color: #3b4fd8;
}

.url-input::placeholder {
    color: #475569;
}

.lookup-btn {
    background: #3b4fd8;
    color: #fff;
    border: none;
    border-radius: 8px;
    padding: 12px 20px;
    font-size: 14px;
    font-weight: 600;
    cursor: pointer;
    white-space: nowrap;
    transition: background 0.15s;
}

.lookup-btn:hover:not(:disabled) {
    background: #4f63e8;
}

.lookup-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
}

.result-card {
    border-color: #1a4a2e;
    background: #121f18;
}

.result-value {
    color: #4ade80;
}

.error-card {
    border-color: #4a1a1a;
    background: #1f1212;
}

.error-value {
    font-size: 14px;
    color: #f87171;
    font-family: inherit;
    font-weight: 400;
}
"#;
