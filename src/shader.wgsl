struct AudioData { 
    samples: array<f32, 1024>
};

@group(0) @binding(0) var<uniform> audio: AudioData;

@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> @builtin(position) vec4<f32> {
    let x = f32(idx) / 1023.0 * 2.0 - 1.0;
    let y = audio.samples[idx] * 0.5;  // Scale to fit in viewport
    return vec4<f32>(x, y, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(0.2, 0.8, 1.0, 1.0);
}
