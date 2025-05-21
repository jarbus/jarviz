[[block]] struct AudioData { samples: array<f32>; };
[[group(0), binding(0)]] var<uniform> audio: AudioData;

[[stage(vertex)]] fn vs_main([[builtin(vertex_index)]] idx: u32) -> [[builtin(position)]] vec4<f32> {
    let x = f32(idx) / f32(arrayLength(&audio.samples) - 1) * 2.0 - 1.0;
    let y = audio.samples[idx];
    return vec4<f32>(x, y, 0.0, 1.0);
}

[[stage(fragment)]] fn fs_main() -> [[location(0)]] vec4<f32> {
    return vec4<f32>(0.2, 0.8, 1.0, 1.0);
}
