use winit::{
    event::{ElementState, Event, KeyEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowBuilder},
};

// Just a helper struct that holds everything we need
struct State<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface_config: wgpu::SurfaceConfiguration,
    window_size: winit::dpi::PhysicalSize<u32>,
    window: &'a Window,
    clear_color: wgpu::Color,
}

impl<'a> State<'a> {
    async fn new(window: &'a Window) -> State<'a> {
        // 1. Get the device and queue
        // Instance of wgpu. Used to work with wgpu and access the api.
        let wgpu_instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());

        // Surface - is the part of the window we draw to. A "canvas"
        let surface = wgpu_instance.create_surface(window).unwrap();

        // A handle to GPU. Needed to get the device
        let adapter = wgpu_instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        println!("{}", adapter.get_info().backend.to_str());

        // TODO: What is device and queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    label: Some("My device"),
                },
                None,
            )
            .await
            .unwrap();

        // 2. Configuring the surface
        let surface_caps = surface.get_capabilities(&adapter);
        let window_size = window.inner_size();

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_caps
                .formats
                .iter()
                .find(|f| f.is_srgb())
                .copied()
                .unwrap_or(surface_caps.formats[0]),
            width: window_size.width,
            height: window_size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: Vec::new(),
            desired_maximum_frame_latency: 2,
        };

        State {
            window,
            surface,
            device,
            queue,
            surface_config,
            window_size,
            clear_color: wgpu::Color::BLACK,
        }
    }

    fn window(&self) -> &Window {
        &self.window
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.window_size = new_size;
            self.surface_config.width = new_size.width;
            self.surface_config.height = new_size.height;
            self.surface.configure(&self.device, &self.surface_config);
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::CursorMoved { position, .. } => {
                self.clear_color = wgpu::Color {
                    r: position.x as f64 / self.window_size.width as f64,
                    g: position.y as f64 / self.window_size.height as f64,
                    b: 1.,
                    a: 1.
                };

                true
            },
            _ => false
        }
    }

    fn update(&mut self) {}

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let texture = self.surface.get_current_texture()?;
        let view = texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("My command encoder"),
            });

        let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("My render pass"),
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(self.clear_color),
                    store: wgpu::StoreOp::Store,
                },
            })],
        });

        // encoder was mutably borrowed when creating `render_pass`
        drop(render_pass);

        self.queue.submit([encoder.finish()]);

        texture.present();

        Ok(())
    }
}

pub async fn run() -> Result<(), String> {
    env_logger::init();

    // Creating a window using just `winit`
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    // Creating our state
    let mut state = State::new(&window).await;

    // Running the event loop
    event_loop
        .run(move |event, control_flow| match event {
            Event::WindowEvent {
                window_id,
                ref event,
            } if window_id == state.window.id() => {
                if !state.input(event) {
                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            event:
                                KeyEvent {
                                    state: ElementState::Pressed,
                                    physical_key: PhysicalKey::Code(KeyCode::Escape),
                                    ..
                                },
                            ..
                        } => control_flow.exit(),
                        WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size);
                        }
                        WindowEvent::RedrawRequested => {
                            state.update();

                            match state.render() {
                                Ok(_) => {}
                                Err(wgpu::SurfaceError::Lost) => state.resize(state.window_size),
                                Err(wgpu::SurfaceError::OutOfMemory) => control_flow.exit(),
                                Err(e) => eprintln!("{:#?}", e),
                            }
                        }
                        _ => {}
                    }
                }
            }
            Event::AboutToWait => {
                state.window.request_redraw();
            }
            _ => {}
        })
        .map_err(|op| op.to_string())
}
