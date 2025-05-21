struct AudioData { 
    samples: array<vec4<f32>, 256>  // Use vec4 for proper alignment (256 * 4 = 1024)
};

@group(0) @binding(0) var<uniform> audio: AudioData;

@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> @builtin(position) vec4<f32> {
    // Make sure we don't go out of bounds with a safer index calculation
    let index = min(idx, 1023u);
    let x = f32(index) / 1023.0 * 2.0 - 1.0;
    
    // Calculate which vec4 element and which component to use
    let vec_index = index / 4u;
    let component_index = index % 4u;
    
    // Get the appropriate component from the vec4
    var y: f32;
    if (component_index == 0u) {
        y = audio.samples[vec_index].x * 5.0; // Increased amplitude
    } else if (component_index == 1u) {
        y = audio.samples[vec_index].y * 5.0;
    } else if (component_index == 2u) {
        y = audio.samples[vec_index].z * 5.0;
    } else {
        y = audio.samples[vec_index].w * 5.0;
    }
    
    return vec4<f32>(x, y, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    // Bright magenta for maximum visibility
    return vec4<f32>(1.0, 0.0, 1.0, 1.0);
}
