use anyhow::{anyhow, Result};
use gtk::prelude::*;
use gtk::{
    Application, ApplicationWindow, Box, Button, Label, Notebook, Orientation, ScrolledWindow,
};
use serde::Deserialize;
use std::fs;

// Hypothetical CEF bindings; you’ll need to choose one (e.g. cef-sys, cef-rs) and adjust the API accordingly.
mod cef {
    // These types and functions are illustrative.
    #[derive(Debug)]
    pub struct CefSettings {
        // Add fields as needed.
    }

    impl CefSettings {
        pub fn default() -> Self {
            CefSettings {}
        }
    }

    pub struct CefApp;

    impl CefApp {
        pub fn initialize(settings: CefSettings) -> Result<(), &'static str> {
            // Initialize CEF; actual implementation depends on the binding.
            println!("CEF initialized with settings: {:?}", settings);
            Ok(())
        }

        pub fn shutdown() {
            // Clean up CEF resources.
            println!("CEF shutdown.");
        }
    }

    // A browser view that can be embedded in GTK.
    #[derive(Clone)]
    pub struct BrowserView {
        url: String,
        // Underlying widget handle, etc.
    }

    impl BrowserView {
        pub fn new(initial_url: &str) -> Self {
            // Create a new browser instance loading the initial URL.
            println!("Creating CEF browser for URL: {}", initial_url);
            BrowserView {
                url: initial_url.to_string(),
            }
        }

        pub fn reload(&self) {
            // Reload the current page.
            println!("Reloading URL: {}", self.url);
            // Actual reload logic goes here.
        }
    }

    // To integrate with GTK, assume BrowserView can be converted into a GTK widget.
    impl AsRef<gtk::Widget> for BrowserView {
        fn as_ref(&self) -> &gtk::Widget {
            // In a real binding, this would return the GTK widget representing the browser.
            // Here we use a placeholder dummy widget.
            static DUMMY: once_cell::sync::Lazy<gtk::Button> = once_cell::sync::Lazy::new(|| {
                let btn = gtk::Button::with_label("CEF Browser Placeholder");
                btn.set_sensitive(false);
                btn
            });
            DUMMY.upcast_ref()
        }
    }
}

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
    // Initialize GTK
    gtk::init()?;

    // Initialize CEF with default settings
    let cef_settings = cef::CefSettings::default();
    cef::CefApp::initialize(cef_settings)
        .map_err(|e| anyhow!("CEF initialization failed: {:?}", e))?;

    let app = Application::new(Some("eu.tomsk.twitch-alerts-rs"), Default::default())
        .expect("Failed to create GTK application");

    app.connect_activate(|app| {
        let window = ApplicationWindow::new(app);
        window.set_title("Twitch Alerts (CEF)");
        window.set_default_size(1024, 768);

        let main_box = Box::new(Orientation::Vertical, 0);
        window.add(&main_box);

        // Read configuration file
        let config_data = fs::read_to_string("urls.json").expect("Failed to read urls.json");
        let config: Config = serde_json::from_str(&config_data).expect("Failed to parse urls.json");

        let notebook = Notebook::new();
        main_box.pack_start(&notebook, true, true, 0);

        // For each URL entry, create a tab with a CEF browser view
        for url_entry in config.urls {
            let scrolled_window =
                ScrolledWindow::new(None::<&gtk::Adjustment>, None::<&gtk::Adjustment>);
            scrolled_window.set_hexpand(true);
            scrolled_window.set_vexpand(true);

            // Create a CEF browser view that loads the given URL.
            let browser_view = cef::BrowserView::new(&url_entry.url);

            // Add the browser view (which implements AsRef<gtk::Widget>) to the scrolled window.
            scrolled_window.add(browser_view.as_ref());

            // Create tab label with a refresh button.
            let tab_label = Box::new(Orientation::Horizontal, 5);
            let label = Label::new(Some(&url_entry.name));
            let refresh_button =
                Button::from_icon_name(Some("view-refresh-symbolic"), gtk::IconSize::SmallToolbar);

            {
                // Clone browser_view for the closure.
                let browser_clone = browser_view.clone();
                refresh_button.connect_clicked(move |_| {
                    browser_clone.reload();
                });
            }

            tab_label.pack_start(&label, true, true, 0);
            tab_label.pack_start(&refresh_button, false, false, 0);

            notebook.append_page(&scrolled_window, Some(&tab_label));
            scrolled_window.show_all();
            tab_label.show_all();
        }

        window.show_all();
    });

    let ret = app.run();

    // Shutdown CEF after GTK application exits
    cef::CefApp::shutdown();

    Ok(())
}
