struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

// Use vec4 for proper 16-byte alignment in uniform buffers
struct AudioData {
    // 256 vec4s = 1024 floats
    data: array<vec4<f32>, 256>,
}

@group(0) @binding(0)
var<uniform> audio_data: AudioData;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var output: VertexOutput;
    
    // Calculate which frequency bin this vertex represents
    let bin_index = vertex_index / 2u;
    let is_top = vertex_index % 2u;
    
    // Get the frequency magnitude (normalized between 0 and 1)
    // Calculate which vec4 and which component to access
    let vec_index = bin_index / 4u;
    let component_index = bin_index % 4u;
    
    // Get the appropriate component from the vec4
    let magnitude = select(
        select(
            select(
                audio_data.data[vec_index].x,
                audio_data.data[vec_index].y,
                component_index == 1u
            ),
            audio_data.data[vec_index].z,
            component_index == 2u
        ),
        audio_data.data[vec_index].w,
        component_index == 3u
    );
    
    // Force a minimum magnitude for testing
    let test_magnitude = max(magnitude, 0.05);
    
    // X position: map bin index to create symmetrical display
    // Lower frequencies on the edges, higher frequencies in the middle
    let normalized_bin = f32(bin_index) / 512.0;
    
    // Transform the bin index to create symmetry
    // For symmetrical display: low frequencies on both ends, high in middle
    var x_pos: f32;
    if (normalized_bin < 0.5) {
        // Left side (low to mid frequencies)
        x_pos = -1.0 + normalized_bin * 2.0;
    } else {
        // Right side (mid to low frequencies)
        x_pos = 3.0 - normalized_bin * 2.0 - 1.0;
    }
    
    // Y position: bottom of bar is always at -0.8, top depends on magnitude
    // Amplify the magnitude to make it more visible
    let y_pos = select(-0.8, -0.8 + test_magnitude * 1.6, is_top == 1u);
    
    // Position in clip space
    output.position = vec4<f32>(x_pos, y_pos, 0.0, 1.0);
    
    // Color based on frequency position
    // For symmetrical display, we want the same colors on both sides
    let normalized_freq = abs(x_pos); // Distance from center
    output.color = vec4<f32>(
        normalized_freq,           // Red increases with frequency
        1.0 - abs(normalized_freq - 0.5) * 2.0,  // Green peaks in the middle
        1.0 - normalized_freq,     // Blue decreases with frequency
        1.0
    );
    
    return output;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
