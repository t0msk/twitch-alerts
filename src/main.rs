use anyhow::{anyhow, Result};
use gtk::gdk;
use gtk::prelude::*;
use serde::Deserialize;
use std::fs;
use webkit2gtk::{WebContext, WebView, WebViewExt};

#[derive(Debug, Deserialize)]
struct Config {
    urls: Vec<UrlEntry>,
}

#[derive(Debug, Deserialize)]
struct UrlEntry {
    name: String,
    url: String,
}

fn main() -> Result<()> {
    // Initialize GTK with explicit backend
    std::env::set_var("GDK_BACKEND", "x11");
    // Set the environment variable before initializing WebKitGTK, fixes blank content of pages
    std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");

    // Initialize GTK and CSS
    gtk::init()?;
    load_css()?;

    let app = gtk::Application::new(Some("eu.tomsk.twitch-alerts-rs"), Default::default());

    app.connect_activate(move |app| {
        let window = gtk::ApplicationWindow::new(app);
        window.realize();

        // Set the application window as a strong reference
        app.add_window(&window);

        window.set_title("Twitch Alerts");
        window.set_default_size(1024, 768);

        // Setup main UI
        let main_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        window.add(&main_box);

        if let Err(e) = setup_interface(&window, &main_box) {
            eprintln!("Fatal error: {}", e);
            app.quit();
        }

        window.show_all();
    });

    // Run the application
    let exit_status = app.run();

    // Ensure the main function waits for the application to finish
    std::process::exit(exit_status);
}

fn setup_interface(_window: &gtk::ApplicationWindow, main_box: &gtk::Box) -> Result<()> {
    let config_data = fs::read_to_string("urls.json")?;
    let config: Config = serde_json::from_str(&config_data)?;

    // Must keep web_context reference alive
    let _web_context =
        WebContext::default().ok_or_else(|| anyhow!("WebKit initialization failed"))?;

    let notebook = gtk::Notebook::new();

    // Ensure notebook expands properly
    main_box.pack_start(&notebook, true, true, 0);

    let notification = gtk::Revealer::new();
    notification.set_reveal_child(false);
    notification.set_transition_type(gtk::RevealerTransitionType::SlideUp);

    // Create a container for proper notification styling
    let notification_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    let notification_label = gtk::Label::new(None);

    // Add styling class instead of inline CSS
    notification_label.style_context().add_class("notification");
    notification_box.add(&notification_label);
    notification.add(&notification_box);

    main_box.pack_end(&notification, false, false, 0);

    // Show all main box components
    main_box.show_all();

    let web_context =
        WebContext::default().ok_or_else(|| anyhow::anyhow!("Failed to create WebContext"))?;

    for url_entry in config.urls {
        let scrolled_window = gtk::ScrolledWindow::new(
            None::<&gtk::Adjustment>, // Explicit type annotation for horizontal adjustment
            None::<&gtk::Adjustment>, // Explicit type annotation for vertical adjustment
        );

        // Ensure scrolled window expands
        scrolled_window.set_hexpand(true);
        scrolled_window.set_vexpand(true);

        let web_view = WebView::with_context(&web_context);

        web_view.load_uri(&url_entry.url);
        scrolled_window.add(&web_view);

        // Create tab label container
        let tab_label = gtk::Box::new(gtk::Orientation::Horizontal, 5);
        let label = gtk::Label::new(Some(&url_entry.name));

        let refresh_button =
            gtk::Button::from_icon_name(Some("view-refresh-symbolic"), gtk::IconSize::SmallToolbar);

        // Connect the refresh button to reload the page.
        let web_view_clone = web_view.clone();
        refresh_button.connect_clicked(move |_| {
            if let Some(uri) = web_view_clone.uri() {
                println!("Refresh clicked! Reloading URI: {}", uri);
                // Option 1: Call reload() directly if available.
                web_view_clone.reload();
                // Option 2: Or force a reload by stopping and reloading.
                // web_view_clone.stop_loading();
                // web_view_clone.load_uri(&uri);
            } else {
                println!("Refresh clicked, but no URI found!");
            }
        });

        tab_label.pack_start(&label, true, true, 0);
        tab_label.pack_start(&refresh_button, false, false, 0);

        // Add page to notebook
        notebook.append_page(&scrolled_window, Some(&tab_label));

        // Show all components
        scrolled_window.show_all();
        tab_label.show_all();
    }

    // Show the notebook after adding all pages
    notebook.show_all();
    Ok(())
}

fn load_css() -> Result<()> {
    let provider = gtk::CssProvider::new();
    provider.load_from_data(include_bytes!("style.css"))?;

    gtk::StyleContext::add_provider_for_screen(
        &gdk::Screen::default().ok_or_else(|| anyhow::anyhow!("No GDK screen available"))?,
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
    Ok(())
}
