struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

// Use vec4 for proper 16-byte alignment in uniform buffers
struct AudioData {
    // 256 vec4s = 1024 floats
    data: array<vec4<f32>, 256>,
}

struct ResolutionData {
    resolution: u32,
    _padding1: u32,
    _padding2: u32,
    _padding3: u32,
}

@group(0) @binding(0)
var<uniform> audio_data: AudioData;

@group(0) @binding(1)
var<uniform> resolution_data: ResolutionData;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var output: VertexOutput;
    
    // Get the current resolution
    let resolution = resolution_data.resolution;
    let half_resolution = resolution / 2u;
    
    // For a line, we only need the top points of what were previously bars
    // Each vertex is a single point in the line
    let bin_index = vertex_index;
    
    // Calculate the data index based on the resolution
    // We need to map the vertex_index to the appropriate data point
    let data_index = min(bin_index * 512u / (resolution * 2u), 511u);
    
    // Get the frequency magnitude (normalized between 0 and 1)
    // Calculate which vec4 and which component to access
    let vec_index = data_index / 4u;
    let component_index = data_index % 4u;
    
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
    // Since we're only showing top 50% of frequencies, we can reduce the amplification
    let test_magnitude = max(magnitude * 2.0, 0.05);
    
    // X position: map bin index to create symmetrical display
    var x_pos: f32;
    
    // Map bin_index from [0, resolution*2-1] to position on x-axis
    // For perfect symmetry with high frequencies in middle:
    // - First half: Low to high frequencies on left side
    // - Second half: High to low frequencies on right side
    if (bin_index < resolution) {
        // Left side: map [0, resolution-1] to [-1.0,0.0]
        x_pos = -1.0 + (f32(bin_index) / f32(resolution - 1u));
    } else {
        // Right side: map [resolution, resolution*2-1] to [0.0,1.0]
        x_pos = (f32(bin_index) - f32(resolution)) / f32(resolution - 1u);
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
