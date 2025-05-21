import init, { Visualizer } from "./pkg/audio_visualizer.js";

async function run() {
  await init();
  const viz = new Visualizer("gpu-canvas");
  const fileInput = document.getElementById("file-input");

  fileInput.onchange = async () => {
    const file = fileInput.files[0];
    const arrayBuffer = await file.arrayBuffer();

    const audioCtx = new AudioContext();
    const buf = await audioCtx.decodeAudioData(arrayBuffer);
    const src = audioCtx.createBufferSource();
    src.buffer = buf;

    const analyser = audioCtx.createAnalyser();
    analyser.fftSize = 2048;
    src.connect(analyser);
    analyser.connect(audioCtx.destination);
    src.start();

    const data = new Uint8Array(analyser.frequencyBinCount);
    function frame() {
      analyser.getByteTimeDomainData(data);
      viz.update(data);
      viz.render();
      requestAnimationFrame(frame);
    }
    requestAnimationFrame(frame);
  };
}
run();
