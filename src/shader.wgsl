struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> audio_data: array<f32, 1024>;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var output: VertexOutput;
    
    // Calculate which frequency bin this vertex represents
    let bin_index = vertex_index / 2u;
    let is_top = vertex_index % 2u;
    
    // Get the frequency magnitude (normalized between 0 and 1)
    let magnitude = audio_data[bin_index];
    
    // X position: map bin index to [-1, 1]
    let x_pos = (f32(bin_index) / 512.0) * 2.0 - 1.0;
    
    // Y position: bottom of bar is always at -0.8, top depends on magnitude
    let y_pos = select(-0.8, -0.8 + magnitude * 1.6, is_top == 1u);
    
    // Position in clip space
    output.position = vec4<f32>(x_pos, y_pos, 0.0, 1.0);
    
    // Color based on frequency (blue for low, green for mid, red for high)
    let normalized_freq = f32(bin_index) / 512.0;
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
