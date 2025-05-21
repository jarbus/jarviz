use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

#[wasm_bindgen]
pub struct Visualizer {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::RenderPipeline,
    vertex_buf: wgpu::Buffer,
    data_buf: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    surface: wgpu::Surface,
    surface_config: wgpu::SurfaceConfiguration,
}

#[wasm_bindgen]
impl Visualizer {
    #[wasm_bindgen(constructor)]
    pub async fn new(canvas_id: &str) -> Visualizer {
        console_error_panic_hook::set_once();
        let window = web_sys::window().unwrap();
        let doc = window.document().unwrap();
        let canvas: HtmlCanvasElement = doc.get_element_by_id(canvas_id).unwrap().dyn_into().unwrap();
        
        // Store canvas dimensions before creating the surface
        let canvas_width = canvas.width();
        let canvas_height = canvas.height();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::GL, // Use WebGL explicitly
            dx12_shader_compiler: Default::default(),
        });
        
        // For WebGL, we need to use the canvas with SurfaceTarget
        let surface = instance.create_surface_from_canvas(canvas).expect("Failed to create surface");
        
        // Request adapter with fallback options
        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }).await.expect("Failed to find an appropriate adapter");

        // Log adapter info for debugging
        web_sys::console::log_1(&format!("Using adapter: {:?}", adapter.get_info().name).into());

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::downlevel_webgl2_defaults(),
            },
            None
        ).await.expect("Failed to request device");
        
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats[0]; // Choose the first available format
        
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: canvas_width,
            height: canvas_height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
        };
        surface.configure(&device, &surface_config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: None,
            vertex: wgpu::VertexState { module: &shader, entry_point: "vs_main", buffers: &[] },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineStrip, // Use line strip to connect points
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        // empty vertex and data buffers
        let vertex_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: 4, // dummy
            usage: wgpu::BufferUsages::VERTEX,
            mapped_at_creation: false,
        });

        // Create buffer for audio data (256 vec4<f32> values = 1024 f32 values)
        // Each vec4 is 16 bytes (4 floats * 4 bytes)
        let data_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Audio Data Buffer"),
            size: (256 * 16) as u64, // 256 vec4s * 16 bytes per vec4
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                count: None,
            }],
            label: None,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: data_buf.as_entire_binding() }],
            label: None,
        });

        Visualizer { device, queue, pipeline, vertex_buf, data_buf, bind_group, surface, surface_config }
    }

    pub fn update(&self, data: &[u8]) {
        // Convert u8 audio data to vec4<f32> and normalize to [-1.0, 1.0]
        // We need 256 vec4s to store 1024 samples
        let mut float_data = vec![0.0f32; 1024]; // Temporary buffer
        
        // Only use the first 1024 samples or pad with zeros if fewer
        let samples_to_use = std::cmp::min(data.len(), 1024);
        for i in 0..samples_to_use {
            // Normalize and amplify slightly for better visibility
            float_data[i] = ((data[i] as f32 / 128.0) - 1.0) * 1.2;
        }
        
        // Log the first few samples for debugging
        web_sys::console::log_1(&format!("Audio samples: {:?}", &float_data[0..5]).into());
        
        // Create aligned vec4 data (256 vec4s = 1024 floats)
        #[repr(C)]
        #[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
        struct Vec4 {
            x: f32,
            y: f32,
            z: f32,
            w: f32,
        }
        
        let mut aligned_data = vec![Vec4 { x: 0.0, y: 0.0, z: 0.0, w: 0.0 }; 256];
        for i in 0..256 {
            let base = i * 4;
            aligned_data[i] = Vec4 {
                x: float_data[base],
                y: float_data[base + 1],
                z: float_data[base + 2],
                w: float_data[base + 3],
            };
        }
        
        // Copy aligned data to the buffer
        self.queue.write_buffer(&self.data_buf, 0, bytemuck::cast_slice(&aligned_data));
    }

    pub fn render(&self) {
        let frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(e) => {
                web_sys::console::error_1(&format!("Failed to get current texture: {:?}", e).into());
                return;
            }
        };
        
        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { 
            label: Some("Render Encoder") 
        });
        
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Main Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
            
            rpass.set_pipeline(&self.pipeline);
            rpass.set_bind_group(0, &self.bind_group, &[]);
            
            // Draw line segments for the waveform
            rpass.draw(0..1024, 0..1);
            
            web_sys::console::log_1(&"Drawing waveform".into());
        }
        
        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
    }
}
