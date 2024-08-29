use std::sync::Arc;

use egui::ahash::HashMap;
use egui_wgpu::wgpu::{
    self, InstanceDescriptor, PowerPreference, RequestAdapterOptions, TextureFormat,
};
use wgpu::{
    Adapter, Backends, CommandEncoder, CommandEncoderDescriptor, CompositeAlphaMode, Device, Queue,
    Surface, SurfaceConfiguration, TextureView, TextureViewDescriptor,
};
use winit::{
    dpi::PhysicalSize,
    error::EventLoopError,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

use twgpu::device_descriptor;

use crate::components::{
    map::TwGpuComponent,
    ui::{context::UiContext, left_panel::LeftPanelUi, UiComponent},
    AppComponent,
};

pub struct WgpuContext<'w> {
    pub adapter: Adapter,
    pub device: Device,
    pub queue: Queue,
    pub surface: Surface<'w>,
    pub config: SurfaceConfiguration,
}

pub struct RenderContext {
    pub command_encoders: HashMap<&'static str, CommandEncoder>,
    pub surface_view: TextureView,
}

pub struct App<'w> {
    window: Arc<Window>,
    event_loop: EventLoop<()>,

    wgpu_context: WgpuContext<'w>,

    // components: Vec<Box<dyn AppComponent>>,
    twgpu: TwGpuComponent,
    egui: UiComponent,
}

impl<'w> App<'w> {
    pub async fn new(width: u32, height: u32) -> Self {
        let event_loop = EventLoop::new().unwrap();
        let window = Arc::new(
            winit::window::WindowBuilder::new()
                .build(&event_loop)
                .unwrap(),
        );
        let _ = window.request_inner_size(PhysicalSize::new(width, height));

        let instance = egui_wgpu::wgpu::Instance::new(InstanceDescriptor {
            backends: Backends::VULKAN,
            ..InstanceDescriptor::default()
        });

        let surface = instance
            .create_surface(window.clone())
            .expect("Failed to create surface!");

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Failed to find an appropriate adapter");

        let mut device_descriptor = device_descriptor(&adapter);
        device_descriptor.required_limits.max_bind_groups = 3;

        let (device, queue) = adapter
            .request_device(&device_descriptor, None)
            .await
            .expect("Failed to create device");

        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let swapchain_format = *swapchain_capabilities
            .formats
            .iter()
            .find(|&d| Self::wanted_formats().contains(d))
            .expect("failed to select proper surface texture format!");

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width,
            height,
            present_mode: wgpu::PresentMode::AutoVsync,
            desired_maximum_frame_latency: 0,
            alpha_mode: CompositeAlphaMode::Auto,
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        let wgpu_context = WgpuContext {
            adapter,
            device,
            queue,
            surface,
            config,
        };

        let mut ui_context = UiContext::new();

        ui_context.add_renderable(LeftPanelUi::new());

        let twgpu = TwGpuComponent::new(width, height, &wgpu_context);
        let egui = UiComponent::new(ui_context, &window, &wgpu_context);

        Self {
            window,
            event_loop,
            wgpu_context,

            twgpu,
            egui,
        }
    }

    pub async fn run(mut self) -> Result<(), EventLoopError> {
        self.event_loop.run(|event, target| {
            target.set_control_flow(ControlFlow::Poll);

            match event {
                Event::WindowEvent {
                    event: window_event,
                    ..
                } => {
                    // for component in self.components.iter_mut() {
                    //     component.on_window_event(&window_event, &self.wgpu_context);
                    // }

                    self.egui.on_user_input(&self.window, &window_event);
                    if !self.egui.is_handling_input() {
                        self.twgpu.on_user_input(&self.window, &window_event);
                    }

                    if let WindowEvent::RedrawRequested = window_event {
                        let surface_texture = self.wgpu_context.surface.get_current_texture().ok();
                        let mut render_context = None;

                        if let Some(frame) = &surface_texture {
                            let surface_view =
                                frame.texture.create_view(&TextureViewDescriptor::default());
                            let twgpu = self.wgpu_context.device.create_command_encoder(
                                &CommandEncoderDescriptor {
                                    label: self.twgpu.label(),
                                },
                            );
                            let egui = self.wgpu_context.device.create_command_encoder(
                                &CommandEncoderDescriptor {
                                    label: self.egui.label(),
                                },
                            );

                            let command_encoders = HashMap::from_iter([
                                (self.twgpu.label().unwrap(), twgpu),
                                (self.egui.label().unwrap(), egui),
                            ]);

                            render_context = Some(RenderContext {
                                command_encoders,
                                surface_view,
                            })
                        }

                        self.twgpu.on_render(
                            &self.window,
                            render_context.as_mut(),
                            &self.wgpu_context,
                        );
                        self.egui.on_render(
                            &self.window,
                            render_context.as_mut(),
                            &self.wgpu_context,
                        );

                        if render_context.is_some() {
                            let twgpu_encoder = render_context
                                .as_mut()
                                .unwrap()
                                .command_encoders
                                .remove(self.twgpu.label().unwrap())
                                .unwrap();
                            let egui_encoder = render_context
                                .as_mut()
                                .unwrap()
                                .command_encoders
                                .remove(self.egui.label().unwrap())
                                .unwrap();
                            self.wgpu_context.queue.submit(Some(twgpu_encoder.finish()));
                            self.wgpu_context.queue.submit(Some(egui_encoder.finish()));
                            surface_texture.unwrap().present();
                            self.window.request_redraw();
                        }
                    }

                    match window_event {
                        WindowEvent::Resized(size) => {
                            self.wgpu_context.config.width = size.width;
                            self.wgpu_context.config.height = size.height;
                            self.wgpu_context
                                .surface
                                .configure(&self.wgpu_context.device, &self.wgpu_context.config);

                            self.twgpu.on_resize(size);
                            self.egui.on_resize(size);
                        }
                        WindowEvent::CloseRequested => target.exit(),
                        _ => {}
                    }
                }

                _ => (),
            }
        })
    }

    pub fn wanted_formats() -> &'static [TextureFormat] {
        &[TextureFormat::Bgra8Unorm, TextureFormat::Rgba8Unorm]
    }
}
