use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, DeviceId, ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, DeviceEvents, EventLoop},
    keyboard::PhysicalKey,
    window::{Window, WindowId},
};

use std::sync::Arc;

use crate::state::State;

#[derive(Default)]
pub struct App {
    state: Option<State>,
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn run() {
        tracing_subscriber::fmt().init();

        let mut app = Self::new();
        let event_loop = EventLoop::new().unwrap();

        event_loop.run_app(&mut app).unwrap();
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        event_loop.listen_device_events(DeviceEvents::WhenFocused);

        let window_attributes = Window::default_attributes().with_title("CSBGI");

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        self.state = Some(
            pollster::block_on(State::new(window)).expect("Failed to initialize application state"),
        );
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let state = match &mut self.state {
            Some(state) => state,
            None => return,
        };

        if state.window_id() != window_id {
            return;
        }

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }

            WindowEvent::Resized(size) => {
                state.resize(size.width, size.height);
            }

            WindowEvent::RedrawRequested => {
                state.update();
                state.prepare();
                match state.render() {
                    Ok(_) => {}
                    Err(e) => {
                        tracing::error!("{e}");
                        event_loop.exit();
                    }
                }
            }

            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(key) = event.physical_key {
                    let pressed = event.state == ElementState::Pressed;
                    state.on_key(key, pressed);
                }
            }

            WindowEvent::Focused(focused) => {
                state.on_focus_changed(focused);
            }

            WindowEvent::MouseInput {
                state: button_state,
                button,
                ..
            } => {
                state.on_mouse_button(button, button_state == ElementState::Pressed);
            }

            _ => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        if let DeviceEvent::MouseMotion { delta } = event
            && let Some(state) = &mut self.state
        {
            state.on_mouse_motion(delta);
        }
    }
}
