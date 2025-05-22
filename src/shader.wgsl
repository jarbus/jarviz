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
    
    // For a line, we only need the top points of what were previously bars
    // Each vertex is a single point in the line
    let bin_index = vertex_index;
    
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
    
    // Amplify the magnitude and set a minimum floor
    let test_magnitude = max(magnitude * 2.5, 0.05);
    
    // X position: map bin index to create symmetrical display
    var x_pos: f32;
    
    // Map bin_index from [0,511] to position on x-axis
    // For perfect symmetry with high frequencies in middle:
    // - Bins 0-255: Low to high frequencies on left side
    // - Bins 256-511: High to low frequencies on right side
    if (bin_index < 256u) {
        // Left side: map [0,255] to [-1.0,0.0]
        x_pos = -1.0 + (f32(bin_index) / 255.0);
    } else {
        // Right side: map [256,511] to [0.0,1.0]
        x_pos = (f32(bin_index) - 256.0) / 255.0;
    }
    
    // Y position: only the top of what was previously a bar
    // Amplify the magnitude to make it more visible
    let y_pos = -0.8 + test_magnitude * 1.8;
    
    // Position in clip space
    output.position = vec4<f32>(x_pos, y_pos, 0.0, 1.0);
    
    // White line
    output.color = vec4<f32>(1.0, 1.0, 1.0, 1.0);
    
    return output;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
