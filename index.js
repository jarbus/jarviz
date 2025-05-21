import init, { Visualizer } from "./pkg/jarviz.js";

async function run() {
  try {
    await init();
    console.log("WASM initialized successfully");
    
    const canvas = document.getElementById("gpu-canvas");
    const viz = new Visualizer("gpu-canvas");
    const fileInput = document.getElementById("file-input");
    
    // Add a message to the page
    const message = document.createElement("p");
    message.id = "status-message";
    message.textContent = "Ready to visualize audio";
    document.body.insertBefore(message, canvas.nextSibling);

    let animationId = null;
    let audioSource = null;

    fileInput.onchange = async () => {
      try {
        // Stop any previous visualization
        if (animationId) {
          cancelAnimationFrame(animationId);
        }
        if (audioSource) {
          audioSource.stop();
        }
        
        message.textContent = "Loading audio...";
        
        const file = fileInput.files[0];
        if (!file) return;
        
        const arrayBuffer = await file.arrayBuffer();
        const audioCtx = new AudioContext();
        
        const buf = await audioCtx.decodeAudioData(arrayBuffer);
        audioSource = audioCtx.createBufferSource();
        audioSource.buffer = buf;

        const analyser = audioCtx.createAnalyser();
        analyser.fftSize = 2048;
        audioSource.connect(analyser);
        analyser.connect(audioCtx.destination);
        
        message.textContent = `Playing: ${file.name}`;
        audioSource.start();

        const data = new Uint8Array(analyser.frequencyBinCount);
        function frame() {
          analyser.getByteTimeDomainData(data);
          
          // Log some debug info
          console.log("Audio data sample:", data[0], data[1], data[2]);
          
          viz.update(data);
          viz.render();
          animationId = requestAnimationFrame(frame);
        }
        animationId = requestAnimationFrame(frame);
        
        // Handle when audio finishes playing
        audioSource.onended = () => {
          message.textContent = "Audio playback complete";
          cancelAnimationFrame(animationId);
        };
      } catch (error) {
        console.error("Error processing audio:", error);
        message.textContent = `Error: ${error.message}`;
      }
    };
  } catch (error) {
    console.error("Initialization error:", error);
    document.body.innerHTML = `<h1>Error initializing visualizer</h1><p>${error.message}</p>`;
  }
}

run();
