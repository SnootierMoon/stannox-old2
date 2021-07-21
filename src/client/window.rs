pub struct ClientWindow {
    window: winit::window::Window,
}

pub struct ClientState {
    main: bool,
    quit: bool,

    start_time: std::time::Instant,
    time: std::time::Instant,
    frame_elapsed: std::time::Duration,

    mouse_rel: uv::Vec2,
    input_mode: InputMode,

    key_held: [bool; 255],
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum InputMode {
    Camera,
    Mouse,
}

impl ClientWindow {
    pub fn new(event_loop: &winit::event_loop::EventLoop<()>) -> Self {
        let window = winit::window::WindowBuilder::new()
            .with_inner_size(winit::dpi::LogicalSize::new(1280, 800))
            .with_title("voxel render demo")
            .build(event_loop)
            .unwrap();

        Self { window }
    }

    pub fn window(&self) -> &winit::window::Window {
        &self.window
    }

    pub fn size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.window.inner_size()
    }

    pub fn center(&self) -> winit::dpi::PhysicalPosition<u32> {
        let size = self.window.inner_size();
        winit::dpi::PhysicalPosition::new(size.width / 2, size.height / 2)
    }

    fn set_input_mode(&mut self, input_mode: InputMode) {
        match input_mode {
            InputMode::Camera => {
                self.window.set_cursor_visible(false);
                self.window.set_cursor_grab(true).unwrap();
            }
            InputMode::Mouse => {
                self.window.set_cursor_position(self.center()).unwrap();
                self.window.set_cursor_visible(true);
                self.window.set_cursor_grab(false).unwrap();
            }
        }
    }

    pub fn run<Fn: 'static + FnMut(&Self, &ClientState)>(
        mut self,
        event_loop: winit::event_loop::EventLoop<()>,
        mut input_handler: Fn,
    ) -> ! {
        let mut state = ClientState::new(&mut self);
        event_loop.run(move |event, _, control_flow| {
            state.handle_event(&mut self, event);
            if state.quit() {
                *control_flow = winit::event_loop::ControlFlow::Exit
            }
            if state.main() {
                input_handler(&self, &state);
                state.reset()
            }
        })
    }
}

impl ClientState {
    pub fn new(window: &mut ClientWindow) -> Self {
        let now = std::time::Instant::now();
        let input_mode = InputMode::Camera;
        window.set_input_mode(input_mode);
        Self {
            main: false,
            quit: false,
            start_time: now,
            time: now,
            frame_elapsed: Default::default(),
            mouse_rel: uv::Vec2::zero(),
            input_mode,
            key_held: [false; 255],
        }
    }

    pub fn handle_event(&mut self, window: &mut ClientWindow, event: winit::event::Event<()>) {
        match event {
            winit::event::Event::MainEventsCleared => {
                let new_time = std::time::Instant::now();
                self.frame_elapsed = new_time - self.time;
                self.time = new_time;
                self.main = true
            }
            winit::event::Event::LoopDestroyed => self.quit = true,
            winit::event::Event::WindowEvent { event, .. } => match event {
                winit::event::WindowEvent::Destroyed
                | winit::event::WindowEvent::CloseRequested => self.quit = true,
                winit::event::WindowEvent::KeyboardInput { input, .. } => {
                    if let Some(keycode) = input.virtual_keycode {
                        if keycode == winit::event::VirtualKeyCode::Escape
                            && self.input_mode == InputMode::Camera
                        {
                            self.set_input_mode(window, InputMode::Mouse)
                        }
                        self.key_held[keycode as usize] =
                            input.state == winit::event::ElementState::Pressed
                    }
                }
                winit::event::WindowEvent::MouseInput { state, button, .. } => {
                    if button == winit::event::MouseButton::Left
                        && state == winit::event::ElementState::Pressed
                        && self.input_mode == InputMode::Mouse
                    {
                        self.set_input_mode(window, InputMode::Camera)
                    }
                }
                _ => (),
            },
            winit::event::Event::DeviceEvent { event, .. } => match event {
                winit::event::DeviceEvent::MouseMotion { delta } => {
                    if self.input_mode == InputMode::Camera {
                        self.mouse_rel.x = delta.0 as f32;
                        self.mouse_rel.y = delta.1 as f32
                    }
                }
                _ => (),
            },
            _ => (),
        }
    }

    pub fn move_vec(&self, move_keys: &[winit::event::VirtualKeyCode; 6]) -> uv::Vec3 {
        match self.input_mode {
            InputMode::Camera => uv::Vec3::new(
                (self.key_held(move_keys[0]) as i32 - self.key_held(move_keys[1]) as i32) as f32,
                (self.key_held(move_keys[2]) as i32 - self.key_held(move_keys[3]) as i32) as f32,
                (self.key_held(move_keys[4]) as i32 - self.key_held(move_keys[5]) as i32) as f32,
            ),
            InputMode::Mouse => uv::Vec3::zero(),
        }
    }

    fn set_input_mode(&mut self, window: &mut ClientWindow, input_mode: InputMode) {
        self.input_mode = input_mode;
        window.set_input_mode(input_mode)
    }

    pub fn reset(&mut self) {
        self.main = false;
        self.mouse_rel = uv::Vec2::zero()
    }

    fn main(&self) -> bool {
        self.main || self.quit
    }
    pub fn quit(&self) -> bool {
        self.quit
    }

    pub fn elapsed(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }
    pub fn frame_elapsed(&self) -> std::time::Duration {
        self.frame_elapsed
    }

    pub fn mouse_rel(&self) -> uv::Vec2 {
        self.mouse_rel
    }

    pub fn key_held(&self, key: winit::event::VirtualKeyCode) -> bool {
        self.key_held[key as usize]
    }
}
