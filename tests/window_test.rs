use once_cell::sync::Lazy;
use spaghetti_engine::settings::GameSettings;
use spaghetti_engine::settings::Setting::*;
use spaghetti_engine::utils::types::Vector2i;
use spaghetti_engine::window::{GameWindow, VsyncMode, WindowMonitor};
use spaghetti_engine::{log, spaghetti_debug_entry_point};
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
    obj.set("window.vsync", Vsync(VsyncMode::Enabled));
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
    spaghetti_debug_entry_point!(|| {
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
    });

    // Window settings
    spaghetti_debug_entry_point!(|| {
        let settings = settings_clone();
        settings.set("window.transparent", Boolean(true));
        let mut window = init_window(&settings);

        let _ = window.set_fullscreen_primary((1920, 1080));
        assert!(window.is_fullscreen());

        window.set_windowed();
        assert!(!window.is_fullscreen());

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

        // This should not create an error anymore!
        thread::spawn(|| {
            let settings = settings_clone();
            match GameWindow::new(&settings) {
                Ok(_) => {}
                Err(_) => panic!("Should've succeeded"),
            };
        })
        .join()
        .unwrap();
    });
}

fn window_integration() {
    spaghetti_debug_entry_point!(|| {
        let settings = settings_clone();
        let mut window = init_window(&settings);
        window.make_context_current();
        window.set_visible(true);

        let _ = WindowMonitor::with_monitors(|monitors| {
            for monitor in monitors.iter() {
                if let Some(name) = monitor.get_name() {
                    log!(Info, "Monitor name: {}", name);
                }
            }
        });

        while !window.should_close() {
            window.swap();
        }
    });
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
