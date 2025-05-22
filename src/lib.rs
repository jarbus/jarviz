use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;
use rustfft::{FftPlanner, num_complex::Complex};

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
    paused: bool,
}

#[wasm_bindgen]
impl Visualizer {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas_id: String) -> wasm_bindgen::JsValue {
        let canvas_id_owned = canvas_id.clone();
        wasm_bindgen_futures::future_to_promise(async move {
            let visualizer = Self::create(&canvas_id_owned).await;
            Ok(visualizer.into())
        }).into()
    }
    
    async fn create(canvas_id: &str) -> Visualizer {
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

        // Create bind group layout first
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                count: None,
            }],
            label: None,
        });
        
        // Create pipeline layout using the bind group layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });
        
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
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
                topology: wgpu::PrimitiveTopology::TriangleStrip, // Use triangle strip for frequency bars
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

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry { binding: 0, resource: data_buf.as_entire_binding() }],
            label: None,
        });

        Visualizer { 
            device, 
            queue, 
            pipeline, 
            vertex_buf, 
            data_buf, 
            bind_group, 
            surface, 
            surface_config,
            paused: false,
        }
    }

    #[wasm_bindgen(js_name = "update")]
    pub fn update(&self, data: &[u8]) {
        // If paused, don't update the visualization
        if self.paused {
            return;
        }
        
        // Debug the raw audio data
        let data_len = data.len();
        let data_sum: u32 = data.iter().map(|&x| x as u32).sum();
        let data_avg = if data_len > 0 { data_sum as f32 / data_len as f32 } else { 0.0 };
        web_sys::console::log_1(&format!("Raw audio data: len={}, avg={:.2}, first few=[{}, {}, {}]", 
            data_len, data_avg, 
            if data_len > 0 { data[0] } else { 0 },
            if data_len > 1 { data[1] } else { 0 },
            if data_len > 2 { data[2] } else { 0 }).into());
        
        // Convert u8 audio data to f32 and normalize to [-1.0, 1.0]
        let mut time_domain = vec![0.0f32; 1024]; // Temporary buffer
        
        // Only use the first 1024 samples or pad with zeros if fewer
        let samples_to_use = std::cmp::min(data.len(), 1024);
        for i in 0..samples_to_use {
            // Normalize to [-1.0, 1.0] and apply a window function
            let sample = ((data[i] as f32 / 128.0) - 1.0);
            
            // Apply Hann window to reduce spectral leakage
            let window = 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / 1024.0).cos());
            time_domain[i] = sample * window;
        }
        
        // Prepare FFT input (complex numbers)
        let mut fft_input: Vec<Complex<f32>> = time_domain.iter()
            .map(|&x| Complex { re: x, im: 0.0 })
            .collect();
        
        // Create FFT planner and perform forward FFT
        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(1024);
        fft.process(&mut fft_input);
        
        // Extract magnitudes from complex FFT results
        // We only need the first half (512 points) due to Nyquist theorem
        let mut frequency_data = [0.0f32; 1024];
        let mut magnitude_pairs = Vec::with_capacity(256);
        
        // Calculate magnitudes for all frequency bins
        for i in 0..256 {
            // Calculate magnitude (absolute value of complex number)
            let magnitude = fft_input[i].norm().sqrt() / 32.0; // Adjust scaling factor
            
            // Apply some scaling to make the visualization more visible
            let scaled_magnitude = magnitude.min(1.0);
            
            // Store the frequency bin index and its magnitude
            magnitude_pairs.push((i, scaled_magnitude));
        }
        
        // Sort by magnitude (ascending order - smaller values first)
        magnitude_pairs.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // Rearrange frequency bins for symmetrical display
        // Smaller magnitudes on the edges, larger magnitudes in the middle
        for (position, (original_index, magnitude)) in magnitude_pairs.iter().enumerate() {
            // Map position from [0,255] to positions on both sides of the visualization
            // Smaller magnitudes (early in the sorted list) go to the edges
            // Larger magnitudes (later in the sorted list) go to the middle
            
            // Left side: position 0 goes to index 0, position 255 goes to 255
            let left_index = position;
            
            // Right side: position 0 goes to index 511, position 255 goes to 256
            let right_index = 511 - position;
            
            // Place the magnitude at both the left and right positions for symmetry
            frequency_data[left_index] = *magnitude;
            frequency_data[right_index] = *magnitude;
        }
        
        // Check if we have any non-zero values
        let max_magnitude = frequency_data.iter().fold(0.0f32, |a, &b| a.max(b));
        web_sys::console::log_1(&format!("FFT data - Max magnitude: {}", max_magnitude).into());
        
        // Log a few specific frequency bins to see if they have values
        web_sys::console::log_1(&format!("Frequency bins [10, 50, 100, 200]: [{:.4}, {:.4}, {:.4}, {:.4}]", 
            frequency_data[10], frequency_data[50], frequency_data[100], frequency_data[200]).into());
        
        // Create properly aligned data for the shader (vec4 array)
        let mut aligned_data = [0.0f32; 1024];
        for i in 0..1024 {
            aligned_data[i] = if i < 512 { frequency_data[i] } else { 0.0 };
        }
        
        // Copy aligned data to the buffer
        self.queue.write_buffer(&self.data_buf, 0, bytemuck::cast_slice(&aligned_data));
    }

    #[wasm_bindgen(js_name = "togglePause")]
    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
        web_sys::console::log_1(&format!("Visualization paused: {}", self.paused).into());
    }
    
    #[wasm_bindgen(js_name = "isPaused")]
    pub fn is_paused(&self) -> bool {
        self.paused
    }
    
    #[wasm_bindgen(js_name = "render")]
    pub fn render(&self) {
        // Use a scope to ensure all rendering is complete before presenting
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            // Get the next texture to render to
            let frame = match self.surface.get_current_texture() {
                Ok(frame) => frame,
                Err(e) => {
                    web_sys::console::error_1(&format!("Failed to get current texture: {:?}", e).into());
                    return;
                }
            };
            
            // Create a view of the texture
            let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
            
            // Create a command encoder to issue GPU commands
            let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { 
                label: Some("Render Encoder") 
            });
            
            // Create a render pass
            {
                let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Main Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.0, // Pure black background
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
                
                // Draw frequency bars (2 vertices per bar, 512 bars = 1024 vertices)
                web_sys::console::log_1(&"Drawing frequency bars - 512 bars with 1024 vertices".into());
                rpass.draw(0..1024, 0..1);
                web_sys::console::log_1(&"Draw call completed".into());
            }
            
            // Submit the work to the GPU
            self.queue.submit(std::iter::once(encoder.finish()));
            
            // Present the frame - this must be done after submitting the work
            frame.present();
        }));
        
        if let Err(e) = result {
            web_sys::console::error_1(&format!("Render panic: {:?}", e).into());
        }
    }
}
