use egui::{ClippedPrimitive, TexturesDelta};
use egui_wgpu::{Renderer, RendererOptions, ScreenDescriptor};

use crate::graphics::{GraphicsContext, builder::RenderPassExt};

pub struct EguiPass {
    renderer: Renderer,
}

pub struct EguiPassInput<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub encoder: &'a mut wgpu::CommandEncoder,
    pub color_view: &'a wgpu::TextureView,
    pub size_in_pixels: [u32; 2],
    pub pixels_per_point: f32,
    pub paint_jobs: &'a [ClippedPrimitive],
    pub textures_delta: &'a mut TexturesDelta,
}

impl EguiPass {
    pub fn new(graphics: &GraphicsContext) -> Self {
        Self {
            renderer: Renderer::new(
                graphics.device(),
                graphics.surface_format(),
                RendererOptions::default(),
            ),
        }
    }

    pub fn record(&mut self, input: EguiPassInput<'_>) {
        for (id, deltas) in &input.textures_delta.set {
            for delta in deltas {
                self.renderer
                    .update_texture(input.device, input.queue, *id, delta);
            }
        }

        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: input.size_in_pixels,
            pixels_per_point: input.pixels_per_point,
        };

        let user_command_buffers = self.renderer.update_buffers(
            input.device,
            input.queue,
            input.encoder,
            input.paint_jobs,
            &screen_descriptor,
        );

        if !user_command_buffers.is_empty() {
            input.queue.submit(user_command_buffers);
        }

        let mut rpass = input
            .encoder
            .render_pass_builder()
            .label("Debug UI Render Pass")
            .color(input.color_view)
            .load()
            .store()
            .build()
            .forget_lifetime();

        self.renderer
            .render(&mut rpass, input.paint_jobs, &screen_descriptor);

        drop(rpass);

        for id in &input.textures_delta.free {
            self.renderer.free_texture(id);
        }

        input.textures_delta.clear();
    }
}
