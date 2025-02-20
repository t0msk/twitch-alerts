use anyhow::{anyhow, Result};
use glib::clone;
use gtk::gdk;
use gtk::prelude::*;
use serde::Deserialize;
use std::fs;
use std::time::Duration;
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
    // Set the environment variable before initializing WebKitGTK
    std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");

    // Add these for better Wayland/X11 integration
    std::env::set_var("GDK_BACKEND", "x11,wayland");
    std::env::set_var("GTK_CSD", "0");

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

    // Run the application and properly hold the main thread
    app.run_with_args::<&str>(&[]);

    // Remove the manual process::exit call
    Ok(())
}

fn setup_interface(_window: &gtk::ApplicationWindow, main_box: &gtk::Box) -> Result<()> {
    let config_data = fs::read_to_string("urls.json")?;
    let config: Config = serde_json::from_str(&config_data)?;

    // Must keep web_context reference alive
    let web_context =
        WebContext::default().ok_or_else(|| anyhow!("WebKit initialization failed"))?;

    let notebook = gtk::Notebook::new();

    // Ensure notebook expands properly
    main_box.pack_start(&notebook, true, true, 0);

    // Create a vertical box for notification positioning
    let notification_container = gtk::Box::new(gtk::Orientation::Vertical, 0);
    main_box.pack_end(&notification_container, false, false, 0);

    // Create the revealer with proper alignment
    let notification = gtk::Revealer::new();
    notification.set_valign(gtk::Align::End);
    notification.set_halign(gtk::Align::Center);
    notification.set_margin_bottom(20);
    notification.set_transition_type(gtk::RevealerTransitionType::SlideUp);
    notification.set_transition_duration(500);

    // Create a proper notification box with styling
    let notification_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
    notification_box
        .style_context()
        .add_class("notification-box");

    let notification_label = gtk::Label::new(None);
    notification_label.set_line_wrap(true);
    notification_box.add(&notification_label);

    // Add close button
    let close_button = gtk::Button::from_icon_name(
        Some("window-close-symbolic"), // Wrap in Some()
        gtk::IconSize::SmallToolbar,
    );
    close_button.connect_clicked(clone!(@weak notification => move |_| {
        notification.set_reveal_child(false);
    }));

    notification_box.add(&close_button);

    notification.add(&notification_box);
    notification_container.add(&notification);

    // Show all main box components
    main_box.show_all();

    for url_entry in config.urls {
        let scrolled_window = gtk::ScrolledWindow::new(
            None::<&gtk::Adjustment>, // Explicit type annotation for horizontal adjustment
            None::<&gtk::Adjustment>, // Explicit type annotation for vertical adjustment
        );

        // Ensure scrolled window expands
        scrolled_window.set_hexpand(true);
        scrolled_window.set_vexpand(true);

        let web_view = WebView::with_context(&web_context);

        // Connect to load-changed signal
        {
            let notification = notification.clone();
            let notification_label = notification_label.clone();

            web_view.connect_load_changed(move |_, load_event| {
                if let webkit2gtk::LoadEvent::Finished = load_event {
                    notification_label.set_text("Page loaded successfully");
                    notification_label.style_context().remove_class("error");
                    notification_label.style_context().add_class("success");
                    notification.set_reveal_child(true);

                    // Auto-hide after 5 seconds using std::time::Duration
                    let notification = notification.clone();
                    glib::timeout_add_local(Duration::from_millis(5000), move || {
                        notification.set_reveal_child(false);
                        glib::Continue(false)
                    });
                }
            });
        }

        // Connect to load-failed signal
        {
            let notification = notification.clone();
            let notification_label = notification_label.clone();
            web_view.connect_load_failed(move |_, _, _, error| {
                // Page failed to load
                notification_label.set_text(&format!("Failed to load page: {}", error));
                notification_label.style_context().remove_class("success");
                notification_label.style_context().add_class("error");
                notification.set_reveal_child(true);

                // Auto-hide after 5 seconds using std::time::Duration
                let notification = notification.clone();
                glib::timeout_add_local(Duration::from_millis(5000), move || {
                    notification.set_reveal_child(false);
                    glib::Continue(false)
                });
                true
            });
        }

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
                web_view_clone.reload();
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
