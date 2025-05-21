struct AudioData { 
    samples: array<f32, 1024>
};

@group(0) @binding(0) var<uniform> audio: AudioData;

@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> @builtin(position) vec4<f32> {
    // Make sure we don't go out of bounds with a safer index calculation
    let index = min(idx, 1023u);
    let x = f32(index) / 1023.0 * 2.0 - 1.0;
    
    // Amplify the signal to make it more visible
    let y = audio.samples[index] * 3.0;
    
    return vec4<f32>(x, y, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    // Bright green for maximum visibility
    return vec4<f32>(0.0, 1.0, 0.0, 1.0);
}
