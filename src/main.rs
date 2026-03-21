mod capture;
mod menu;
mod renderer;
mod settings;

use std::sync::Arc;
use std::time::Instant;

use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowAttributes, WindowLevel};

use capture::Capture;
use menu::{AppMenu, MenuAction};
use renderer::Renderer;
use settings::Settings;

#[derive(Debug, Clone, Copy, PartialEq)]
enum ResizeEdge {
    North,
    South,
    East,
    West,
    NorthWest,
    NorthEast,
    SouthWest,
    SouthEast,
}

impl ResizeEdge {
    fn to_winit(self) -> winit::window::ResizeDirection {
        match self {
            Self::North => winit::window::ResizeDirection::North,
            Self::South => winit::window::ResizeDirection::South,
            Self::East => winit::window::ResizeDirection::East,
            Self::West => winit::window::ResizeDirection::West,
            Self::NorthWest => winit::window::ResizeDirection::NorthWest,
            Self::NorthEast => winit::window::ResizeDirection::NorthEast,
            Self::SouthWest => winit::window::ResizeDirection::SouthWest,
            Self::SouthEast => winit::window::ResizeDirection::SouthEast,
        }
    }
}

struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    capture: Option<Capture>,
    app_menu: AppMenu,
    settings: Settings,
    is_expanded: bool,
    compact_size: (u32, u32),
    cursor_edge: Option<ResizeEdge>,
    cursor_pos: PhysicalPosition<f64>,
    last_click_time: Option<Instant>,
    left_pressed: bool,
    drag_initiated: bool,
    edge_at_press: Option<ResizeEdge>,
}

impl App {
    fn new() -> Self {
        let settings = Settings::load();
        Self {
            window: None,
            renderer: None,
            capture: None,
            app_menu: AppMenu::new(),
            settings,
            is_expanded: false,
            compact_size: (0, 0),
            cursor_edge: None,
            cursor_pos: PhysicalPosition::new(0.0, 0.0),
            last_click_time: None,
            left_pressed: false,
            drag_initiated: false,
            edge_at_press: None,
        }
    }

    fn update_uniforms(&mut self) {
        if let Some(renderer) = &mut self.renderer {
            renderer.uniforms.zoom_level = match self.settings.zoom_level {
                2 => 1.0,
                3 => 2.0,
                _ => 0.0,
            };
            renderer.uniforms.flipped = if self.settings.flipped { 1.0 } else { 0.0 };
            renderer.uniforms.fit_mode = match self.settings.fit_mode.as_str() {
                "contain" => 1.0,
                "fill" => 2.0,
                _ => 0.0,
            };
            renderer.uniforms.shape = if self.is_expanded {
                2.0
            } else {
                match self.settings.shape.as_str() {
                    "rounded-rect" => 1.0,
                    _ => 0.0,
                }
            };
        }
    }

    fn update_menu(&mut self) {
        if self.app_menu.visible {
            let (pixels, w, h) = self.app_menu.render_pixels(&self.settings);
            if let Some(renderer) = &mut self.renderer {
                renderer.uniforms.menu_x = self.app_menu.pos_x;
                renderer.uniforms.menu_y = self.app_menu.pos_y;
                renderer.uniforms.menu_w = w as f32;
                renderer.uniforms.menu_h = h as f32;
                renderer.upload_menu(&pixels, w, h);
                renderer.menu_visible = true;
            }
        } else if let Some(renderer) = &mut self.renderer {
            renderer.menu_visible = false;
        }
    }

    fn compute_edge(&self, x: f64, y: f64, w: f64, h: f64) -> Option<ResizeEdge> {
        const BORDER: f64 = 16.0;
        let left = x < BORDER;
        let right = x > w - BORDER;
        let top = y < BORDER;
        let bottom = y > h - BORDER;
        match (left, right, top, bottom) {
            (true, _, true, _) => Some(ResizeEdge::NorthWest),
            (_, true, true, _) => Some(ResizeEdge::NorthEast),
            (true, _, _, true) => Some(ResizeEdge::SouthWest),
            (_, true, _, true) => Some(ResizeEdge::SouthEast),
            (true, _, _, _) => Some(ResizeEdge::West),
            (_, true, _, _) => Some(ResizeEdge::East),
            (_, _, true, _) => Some(ResizeEdge::North),
            (_, _, _, true) => Some(ResizeEdge::South),
            _ => None,
        }
    }

    fn toggle_expanded(&mut self) {
        if let Some(window) = &self.window {
            if self.is_expanded {
                self.is_expanded = false;
                window.set_fullscreen(None);
            } else {
                let size = window.inner_size();
                if size.width > 0 {
                    self.compact_size = (size.width, size.height);
                }
                self.is_expanded = true;
                window.set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
            }
            self.update_uniforms();
        }
    }

    fn handle_menu_action(&mut self, action: MenuAction) {
        match action {
            MenuAction::SetZoom(level) => self.settings.zoom_level = level,
            MenuAction::SetShape(s) => self.settings.shape = s,
            MenuAction::SetFit(f) => self.settings.fit_mode = f,
            MenuAction::ToggleMirror => self.settings.flipped = !self.settings.flipped,
        }
        self.update_uniforms();
        self.settings.save();
        // Re-render menu to show updated state
        self.update_menu();
    }
}

impl winit::application::ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let attrs = WindowAttributes::default()
            .with_title("Cam Overlay")
            .with_decorations(false)
            .with_transparent(true)
            .with_inner_size(PhysicalSize::new(
                self.settings.window_width,
                self.settings.window_height,
            ))
            .with_window_level(WindowLevel::AlwaysOnTop);

        let window = Arc::new(
            event_loop
                .create_window(attrs)
                .expect("Failed to create window"),
        );
        self.compact_size = (self.settings.window_width, self.settings.window_height);

        match Renderer::new(window.clone()) {
            Ok(renderer) => self.renderer = Some(renderer),
            Err(e) => {
                eprintln!("Failed to create renderer: {e}");
                event_loop.exit();
                return;
            }
        }

        match Capture::new() {
            Ok(capture) => self.capture = Some(capture),
            Err(e) => {
                eprintln!("Failed to start camera capture: {e:#}");
                // Continue without camera — shows blank window
            }
        }

        self.update_uniforms();
        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                self.settings.save();
                event_loop.exit();
            }

            WindowEvent::Resized(size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer.resize(size.width, size.height);
                }
                if !self.is_expanded && size.width > 0 && size.height > 0 {
                    self.compact_size = (size.width, size.height);
                    self.settings.window_width = size.width;
                    self.settings.window_height = size.height;
                }
            }

            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_pos = position;

                // Update menu hover
                if self.app_menu.visible {
                    self.app_menu
                        .update_hover(position.x as f32, position.y as f32);
                    self.update_menu();
                }

                if let Some(window) = &self.window {
                    let size = window.inner_size();
                    self.cursor_edge = self.compute_edge(
                        position.x,
                        position.y,
                        size.width as f64,
                        size.height as f64,
                    );

                    if self.left_pressed && !self.drag_initiated && !self.app_menu.visible {
                        self.drag_initiated = true;
                        if let Some(edge) = self.edge_at_press {
                            let _ = window.drag_resize_window(edge.to_winit());
                        } else if !self.is_expanded {
                            let _ = window.drag_window();
                        }
                    }
                }
            }

            WindowEvent::MouseInput { state, button, .. } => match button {
                MouseButton::Left => {
                    if state == ElementState::Pressed {
                        // If menu is visible, check for menu click first
                        if self.app_menu.visible {
                            let action = self.app_menu.click(
                                self.cursor_pos.x as f32,
                                self.cursor_pos.y as f32,
                                &self.settings,
                            );
                            if let Some(action) = action {
                                self.handle_menu_action(action);
                            } else {
                                // Clicked outside menu items, close menu
                                self.app_menu.hide();
                                self.update_menu();
                            }
                            return;
                        }

                        // Double-click detection
                        let now = Instant::now();
                        if let Some(last) = self.last_click_time {
                            if now.duration_since(last).as_millis() < 400 {
                                self.toggle_expanded();
                                self.last_click_time = None;
                                return;
                            }
                        }
                        self.last_click_time = Some(now);

                        self.left_pressed = true;
                        self.drag_initiated = false;
                        self.edge_at_press = self.cursor_edge;
                    } else {
                        self.left_pressed = false;
                        self.drag_initiated = false;
                    }
                }
                MouseButton::Right => {
                    if state == ElementState::Pressed {
                        if self.app_menu.visible {
                            self.app_menu.hide();
                        } else {
                            self.app_menu
                                .show(self.cursor_pos.x as f32, self.cursor_pos.y as f32);
                        }
                        self.update_menu();
                    }
                }
                _ => {}
            },

            WindowEvent::RedrawRequested => {
                // Pull latest frame from camera
                if let Some(capture) = &mut self.capture {
                    if let Some(frame) = capture.try_pull_frame() {
                        if let Some(renderer) = &mut self.renderer {
                            renderer.upload_frame(&frame);
                        }
                    }
                }

                if let Some(renderer) = &mut self.renderer {
                    match renderer.render() {
                        Ok(()) => {}
                        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                            if let Some(window) = &self.window {
                                let size = window.inner_size();
                                renderer.resize(size.width, size.height);
                            }
                        }
                        Err(e) => eprintln!("Render error: {e}"),
                    }
                }

                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }

            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

fn main() -> anyhow::Result<()> {
    let event_loop = EventLoop::new()?;
    let mut app = App::new();
    event_loop.run_app(&mut app)?;
    Ok(())
}
