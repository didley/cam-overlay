use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk4::{gdk, gio, glib};
use gstreamer::prelude::*;
use std::cell::{Cell, RefCell};

const SETTINGS_SCHEMA: &str = "io.github.didley.CamOverlay";

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct CamOverlayWindow {
        pub pipeline: RefCell<Option<gstreamer::Element>>,
        pub settings: RefCell<Option<gio::Settings>>,
        pub overlay_container: RefCell<Option<gtk4::Overlay>>,
        pub video_picture: RefCell<Option<gtk4::Picture>>,
        pub is_expanded: Cell<bool>,
        pub compact_width: Cell<i32>,
        pub compact_height: Cell<i32>,
        pub compact_x: Cell<i32>,
        pub compact_y: Cell<i32>,
        pub video_width: Cell<i32>,
        pub video_height: Cell<i32>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for CamOverlayWindow {
        const NAME: &'static str = "CamOverlayWindow";
        type Type = super::CamOverlayWindow;
        type ParentType = adw::ApplicationWindow;
    }

    impl ObjectImpl for CamOverlayWindow {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.init_window();
        }
    }

    impl WidgetImpl for CamOverlayWindow {}
    impl WindowImpl for CamOverlayWindow {}
    impl ApplicationWindowImpl for CamOverlayWindow {}
    impl AdwApplicationWindowImpl for CamOverlayWindow {}
}

glib::wrapper! {
    pub struct CamOverlayWindow(ObjectSubclass<imp::CamOverlayWindow>)
        @extends adw::ApplicationWindow, gtk4::ApplicationWindow, gtk4::Window, gtk4::Widget,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl CamOverlayWindow {
    pub fn new(app: &adw::Application) -> Self {
        glib::Object::builder()
            .property("application", app)
            .build()
    }

    fn init_window(&self) {
        let imp = self.imp();

        // Load settings
        let settings = gio::Settings::new(SETTINGS_SCHEMA);
        *imp.settings.borrow_mut() = Some(settings.clone());

        // Window setup
        self.set_decorated(false);
        self.set_resizable(true);

        let saved_width = settings.int("window-width");
        let saved_height = settings.int("window-height");
        self.set_default_size(saved_width, saved_height);
        imp.compact_width.set(saved_width);
        imp.compact_height.set(saved_height);

        // Build widget tree
        let overlay_container = gtk4::Overlay::new();
        let video_picture = gtk4::Picture::new();
        video_picture.set_content_fit(gtk4::ContentFit::Fill);

        overlay_container.set_child(Some(&video_picture));
        self.set_content(Some(&overlay_container));

        *imp.overlay_container.borrow_mut() = Some(overlay_container.clone());
        *imp.video_picture.borrow_mut() = Some(video_picture);

        // Apply saved shape
        let shape = settings.string("shape");
        overlay_container.add_css_class(shape.as_str());

        // Setup GStreamer pipeline
        self.setup_pipeline();

        // Setup gestures
        self.setup_drag();
        self.setup_double_click();
        self.setup_context_menu();

        // Setup actions
        self.setup_actions();

        // Save window size on resize (only when not expanded)
        let win = self.clone();
        self.connect_notify(Some("default-width"), move |_, _| {
            if !win.imp().is_expanded.get() {
                let w = win.default_width();
                let h = win.default_height();
                if w > 0 && h > 0 {
                    win.imp().compact_width.set(w);
                    win.imp().compact_height.set(h);
                    if let Some(s) = win.imp().settings.borrow().as_ref() {
                        let _ = s.set_int("window-width", w);
                        let _ = s.set_int("window-height", h);
                    }
                }
            }
        });
    }

    fn setup_pipeline(&self) {
        let imp = self.imp();

        let pipeline = match gstreamer::parse::launch(
            "pipewiresrc ! videoconvert name=converter ! videoflip name=flipper method=none ! videocrop name=cropper ! videoscale ! gtk4paintablesink name=sink sync=false"
        ) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Failed to create pipeline: {e}");
                return;
            }
        };

        let sink = pipeline.downcast_ref::<gstreamer::Bin>()
            .and_then(|bin| bin.by_name("sink"))
            .expect("Sink element not found");

        let paintable = sink.property::<gdk::Paintable>("paintable");

        if let Some(picture) = imp.video_picture.borrow().as_ref() {
            picture.set_paintable(Some(&paintable));
        }

        // Listen for caps to get video resolution
        let win = self.clone();
        if let Some(bin) = pipeline.downcast_ref::<gstreamer::Bin>() {
            if let Some(converter) = bin.by_name("converter") {
                if let Some(src_pad) = converter.static_pad("src") {
                    src_pad.connect_notify(Some("caps"), move |pad, _| {
                        if let Some(caps) = pad.current_caps() {
                            if let Some(s) = caps.structure(0) {
                                let width = s.get::<i32>("width").unwrap_or(0);
                                let height = s.get::<i32>("height").unwrap_or(0);
                                if width > 0 && height > 0 {
                                    win.imp().video_width.set(width);
                                    win.imp().video_height.set(height);
                                    // Apply saved zoom now that we have resolution
                                    let zoom = win.imp().settings.borrow()
                                        .as_ref()
                                        .map(|s| s.int("zoom-level"))
                                        .unwrap_or(1);
                                    win.apply_zoom(zoom);
                                }
                            }
                        }
                    });
                }
            }
        }

        // Apply saved flip
        let flipped = imp.settings.borrow().as_ref()
            .map(|s| s.boolean("flipped"))
            .unwrap_or(false);
        if flipped {
            if let Some(bin) = pipeline.downcast_ref::<gstreamer::Bin>() {
                if let Some(flipper) = bin.by_name("flipper") {
                    flipper.set_property("method", 4i32);
                }
            }
        }

        if let Err(e) = pipeline.set_state(gstreamer::State::Playing) {
            eprintln!("Failed to start pipeline: {e}");
        }

        *imp.pipeline.borrow_mut() = Some(pipeline);
    }

    fn setup_drag(&self) {
        let drag = gtk4::GestureDrag::new();
        drag.set_button(1);

        let win = self.clone();
        drag.connect_drag_begin(move |gesture, x, y| {
            if win.imp().is_expanded.get() {
                return;
            }
            gesture.set_state(gtk4::EventSequenceState::Claimed);
            win.begin_move_drag(
                1,
                x as i32,
                y as i32,
                gdk::CURRENT_TIME,
            );
        });

        self.add_controller(drag);
    }

    fn setup_double_click(&self) {
        let click = gtk4::GestureClick::new();
        click.set_button(1);

        let win = self.clone();
        click.connect_pressed(move |gesture, n_press, _, _| {
            if n_press == 2 {
                gesture.set_state(gtk4::EventSequenceState::Claimed);
                win.toggle_expanded();
            }
        });

        self.add_controller(click);
    }

    fn toggle_expanded(&self) {
        let imp = self.imp();
        let expanded = imp.is_expanded.get();

        if expanded {
            // Collapse back to compact
            imp.is_expanded.set(false);
            self.set_default_size(imp.compact_width.get(), imp.compact_height.get());

            if let Some(overlay) = imp.overlay_container.borrow().as_ref() {
                overlay.remove_css_class("expanded");
                // Restore shape class
                let shape = imp.settings.borrow().as_ref()
                    .map(|s| s.string("shape").to_string())
                    .unwrap_or_else(|| "circle".to_string());
                overlay.add_css_class(&shape);
            }
        } else {
            // Save compact size
            let w = self.default_width();
            let h = self.default_height();
            if w > 0 { imp.compact_width.set(w); }
            if h > 0 { imp.compact_height.set(h); }

            imp.is_expanded.set(true);

            // Get screen dimensions
            if let Some(display) = gdk::Display::default() {
                if let Some(monitor) = display.primary_monitor()
                    .or_else(|| display.monitors().item(0).and_downcast::<gdk::Monitor>())
                {
                    let geometry = monitor.geometry();
                    let padding = 60;
                    self.set_default_size(
                        geometry.width() - padding * 2,
                        geometry.height() - padding * 2,
                    );
                }
            }

            if let Some(overlay) = imp.overlay_container.borrow().as_ref() {
                // Remove shape classes, add expanded
                overlay.remove_css_class("circle");
                overlay.remove_css_class("rounded-rect");
                overlay.add_css_class("expanded");
            }
        }
    }

    fn setup_context_menu(&self) {
        // Build menu model
        let menu = gio::Menu::new();

        // Zoom section
        let zoom_section = gio::Menu::new();
        zoom_section.append(Some("1×"), Some("win.zoom::1"));
        zoom_section.append(Some("1.5×"), Some("win.zoom::2"));
        zoom_section.append(Some("2×"), Some("win.zoom::3"));
        menu.append_section(Some("Zoom"), &zoom_section);

        // Shape section
        let shape_section = gio::Menu::new();
        shape_section.append(Some("Circle"), Some("win.shape::circle"));
        shape_section.append(Some("Rounded Rectangle"), Some("win.shape::rounded-rect"));
        menu.append_section(Some("Shape"), &shape_section);

        // Mirror section
        let mirror_section = gio::Menu::new();
        mirror_section.append(Some("Mirror"), Some("win.flip"));
        menu.append_section(None, &mirror_section);

        // About section
        let about_section = gio::Menu::new();
        about_section.append(Some("About"), Some("app.about"));
        menu.append_section(None, &about_section);

        let popover = gtk4::PopoverMenu::from_model(Some(&menu));
        popover.set_has_arrow(false);
        popover.set_parent(self);

        let right_click = gtk4::GestureClick::new();
        right_click.set_button(3);

        let win = self.clone();
        right_click.connect_pressed(move |gesture, _, x, y| {
            gesture.set_state(gtk4::EventSequenceState::Claimed);
            let rect = gdk::Rectangle::new(x as i32, y as i32, 1, 1);
            popover.set_pointing_to(Some(&rect));
            popover.popup();
            let _ = win.clone(); // keep win alive
        });

        self.add_controller(right_click);
    }

    fn setup_actions(&self) {
        let settings = self.imp().settings.borrow().clone().expect("Settings not initialized");

        // Zoom action (stateful string, target is "1", "2", or "3")
        let zoom_level = settings.int("zoom-level").to_string();
        let zoom_action = gio::SimpleAction::new_stateful(
            "zoom",
            Some(&glib::VariantTy::STRING),
            &zoom_level.to_variant(),
        );

        let win = self.clone();
        zoom_action.connect_activate(move |action, param| {
            if let Some(v) = param {
                let level_str: String = v.get().unwrap_or_default();
                let level: i32 = level_str.parse().unwrap_or(1);
                action.set_state(&level_str.to_variant());
                win.apply_zoom(level);
                if let Some(s) = win.imp().settings.borrow().as_ref() {
                    let _ = s.set_int("zoom-level", level);
                }
            }
        });
        self.add_action(&zoom_action);

        // Shape action (stateful string)
        let current_shape = settings.string("shape").to_string();
        let shape_action = gio::SimpleAction::new_stateful(
            "shape",
            Some(&glib::VariantTy::STRING),
            &current_shape.to_variant(),
        );

        let win = self.clone();
        shape_action.connect_activate(move |action, param| {
            if let Some(v) = param {
                let shape: String = v.get().unwrap_or_default();
                action.set_state(&shape.to_variant());
                win.apply_shape(&shape);
                if let Some(s) = win.imp().settings.borrow().as_ref() {
                    let _ = s.set_string("shape", &shape);
                }
            }
        });
        self.add_action(&shape_action);

        // Flip action (stateful boolean toggle)
        let flipped = settings.boolean("flipped");
        let flip_action = gio::SimpleAction::new_stateful(
            "flip",
            None,
            &flipped.to_variant(),
        );

        let win = self.clone();
        flip_action.connect_activate(move |action, _| {
            let current: bool = action.state().and_then(|v| v.get()).unwrap_or(false);
            let new_state = !current;
            action.set_state(&new_state.to_variant());
            win.apply_flip(new_state);
            if let Some(s) = win.imp().settings.borrow().as_ref() {
                let _ = s.set_boolean("flipped", new_state);
            }
        });
        self.add_action(&flip_action);
    }

    fn apply_zoom(&self, level: i32) {
        let imp = self.imp();
        let width = imp.video_width.get();
        let height = imp.video_height.get();

        if width == 0 || height == 0 {
            return;
        }

        let (left, right, top, bottom) = match level {
            2 => {
                let lr = width / 6;
                let tb = height / 6;
                (lr, lr, tb, tb)
            }
            3 => {
                let lr = width / 4;
                let tb = height / 4;
                (lr, lr, tb, tb)
            }
            _ => (0, 0, 0, 0),
        };

        if let Some(pipeline) = imp.pipeline.borrow().as_ref() {
            if let Some(bin) = pipeline.downcast_ref::<gstreamer::Bin>() {
                if let Some(cropper) = bin.by_name("cropper") {
                    cropper.set_property("left", left);
                    cropper.set_property("right", right);
                    cropper.set_property("top", top);
                    cropper.set_property("bottom", bottom);
                }
            }
        }
    }

    fn apply_shape(&self, shape: &str) {
        let imp = self.imp();
        if imp.is_expanded.get() {
            return; // Don't change shape in expanded mode
        }
        if let Some(overlay) = imp.overlay_container.borrow().as_ref() {
            overlay.remove_css_class("circle");
            overlay.remove_css_class("rounded-rect");
            overlay.add_css_class(shape);
        }
    }

    fn apply_flip(&self, flipped: bool) {
        let imp = self.imp();
        if let Some(pipeline) = imp.pipeline.borrow().as_ref() {
            if let Some(bin) = pipeline.downcast_ref::<gstreamer::Bin>() {
                if let Some(flipper) = bin.by_name("flipper") {
                    let method: i32 = if flipped { 4 } else { 0 };
                    flipper.set_property("method", method);
                }
            }
        }
    }
}

impl Drop for CamOverlayWindow {
    fn drop(&mut self) {
        if let Some(pipeline) = self.imp().pipeline.borrow().as_ref() {
            let _ = pipeline.set_state(gstreamer::State::Null);
        }
    }
}
