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
    
    // Amplify the magnitude and set a minimum floor
    let test_magnitude = max(magnitude * 2.5, 0.05);
    
    // X position: map bin index to create symmetrical display
    // Lower frequencies on the edges, higher frequencies in the middle
    let normalized_bin = f32(bin_index) / 512.0;
    
    // Transform the bin index to create symmetry
    // For symmetrical display: high frequencies in middle, low on edges
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
    
    // Y position: bottom of bar is always at -0.8, top depends on magnitude
    // Amplify the magnitude to make it more visible
    let y_pos = select(-0.8, -0.8 + test_magnitude * 1.8, is_top == 1u);
    
    // Position in clip space
    output.position = vec4<f32>(x_pos, y_pos, 0.0, 1.0);
    
    // All colors are white
    output.color = vec4<f32>(1.0, 1.0, 1.0, 1.0);
    
    return output;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
