use crate::core::*;
use crate::log;
use crate::settings::GameSettings;
use crate::settings::Setting::*;
use crate::utils::logger::Severity;
use crate::utils::types::Vector2i;
use crate::utils::Logger;
use once_cell::sync::Lazy;
use std::io::{stdout, Write};
use std::sync::Arc;
use std::thread;

static DEFAULT_SETTINGS: Lazy<GameSettings> = Lazy::new(|| {
    let obj = GameSettings::new();
    obj.set(
        "window.fullscreenResolution",
        IVector2(Vector2i::new(1920, 1080)),
    );
    obj.set("window.fullscreenMonitor", SignedInt(0));
    obj.set("window.size", IVector2(Vector2i::new(256, 256)));
    obj.set("window.minimumSize", IVector2(Vector2i::new(256, 256)));
    obj.set("window.maximumSize", IVector2(Vector2i::new(-1, -1))); // No max size
    obj.set("window.fullscreen", Boolean(false));
    obj.set("window.resizable", Boolean(true));
    obj.set("window.maximized", Boolean(false));
    obj.set("window.vsync", Vsync(WindowVsyncMode::Enabled));
    obj.set("window.transparent", Boolean(false));

    obj.set("window.debugContext", Boolean(true));

    obj.set("window.title", Str(String::from("Spaghetti game")));
    obj.set("window.icon16", Empty);
    obj.set("window.icon32", Empty);

    obj
});

// Has to be done in a single thread because of glfw limitations
#[test]
fn window() {
    // Default settings
    {
        let settings = settings_clone();
        init_window(&settings);
    }

    // Fullscreen
    {
        let settings = settings_clone();
        settings.set("window.fullscreen", Boolean(true));
        init_window(&settings);
    }

    // Maximized
    {
        let settings = settings_clone();
        settings.set("window.maximized", Boolean(true));
        let window = init_window(&settings);
        assert!(window.is_maximized());
    }

    // Resizable
    {
        let settings = settings_clone();
        settings.set("window.resizable", Boolean(false));
        let window = init_window(&settings);
        assert!(!window.is_resizable());
    }

    // Debug context
    {
        let settings = settings_clone();
        settings.set("window.debugContext", Boolean(true));
        let window = init_window(&settings);
        assert!(window.is_debug_context());
    }

    // Transparency
    {
        let settings = settings_clone();
        settings.set("window.transparent", Boolean(true));
        let window = init_window(&settings);
        assert!(window.is_transparent());
    }

    // Window size limits (positive)
    {
        let settings = settings_clone();
        settings.set("window.minimumSize", IVector2(Vector2i::new(100, 100)));
        settings.set("window.maximumSize", IVector2(Vector2i::new(500, 500)));
        let window = init_window(&settings);
        assert_eq!(window.get_size_limits(), (100, 100, 500, 500));
    }

    // Window size limits (negative)
    {
        let settings = settings_clone();
        settings.set("window.minimumSize", IVector2(Vector2i::new(-4, 100)));
        settings.set("window.maximumSize", IVector2(Vector2i::new(500, 100)));
        let window = init_window(&settings);
        assert_eq!(window.get_size_limits(), (-1, -1, 500, 100));
    }

    // Window size
    {
        let settings = settings_clone();
        settings.set("window.minimumSize", IVector2(Vector2i::new(100, 100)));
        settings.set("window.maximumSize", IVector2(Vector2i::new(-1, -1)));
        settings.set("window.size", IVector2(Vector2i::new(120, 300)));
        let window = init_window(&settings);
        assert_eq!(window.get_size(), (120, 300));
    }

    // Window settings
    let settings = settings_clone();
    settings.set("window.transparent", Boolean(true));
    let mut window = init_window(&settings);

    window.set_fullscreen_primary((1920, 1080));
    assert!(window.is_fullscreen());

    window.set_windowed();
    assert!(!window.is_fullscreen());

    window.set_size((534, 272));
    assert_eq!(window.get_size(), (534, 272));

    window.set_size_limits((12, 40, 900, 800));
    assert_eq!(window.get_size_limits(), (12, 40, 900, 800));

    window.set_decorated(false);
    assert!(!window.is_decorated());

    window.set_should_close(true);
    assert!(window.should_close());

    window.set_visible(false);
    assert!(!window.is_visible());

    window.set_opacity(0.3);
    assert!((window.get_opacity() - 0.3).abs() < 0.1);

    // This should create an error
    thread::spawn(|| {
        let settings = settings_clone();
        match GameWindow::new(&settings) {
            Ok(window) => panic!("Should've crashed"),
            Err(_) => {}
        };
    })
    .join()
    .unwrap();
}

fn settings_clone() -> Arc<GameSettings> {
    Arc::new(DEFAULT_SETTINGS.clone())
}

fn init_window(settings: &Arc<GameSettings>) -> GameWindow {
    match GameWindow::new(settings) {
        Ok(window) => window,
        Err(error) => {
            panic!("{}", error);
        }
    }
}
