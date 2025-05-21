import init, { Visualizer } from "./pkg/jarviz.js";

async function run() {
  try {
    await init();
    console.log("WASM initialized successfully");
    
    const canvas = document.getElementById("gpu-canvas");
    
    // Since Visualizer constructor is async, we need to await it
    console.log("Creating Visualizer...");
    const vizPromise = new Visualizer("gpu-canvas");
    console.log("Visualizer promise:", vizPromise);
    
    // Properly await the Promise to get the actual Visualizer instance
    const viz = await vizPromise;
    console.log("Visualizer resolved:", viz);
    
    // More detailed logging of the Visualizer object
    console.log("Visualizer object:", viz);
    console.log("Visualizer prototype:", Object.getPrototypeOf(viz));
    console.log("Visualizer methods:", Object.getOwnPropertyNames(Object.getPrototypeOf(viz)));
    console.log("Visualizer own properties:", Object.getOwnPropertyNames(viz));
    console.log("update method type:", typeof viz.update);
    console.log("render method type:", typeof viz.render);
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

        // Create a buffer for the audio data
        const data = new Uint8Array(1024);
        
        // Add a debug element to show audio values
        const debugElement = document.createElement("div");
        debugElement.style.fontSize = "10px";
        debugElement.style.fontFamily = "monospace";
        debugElement.style.marginTop = "10px";
        document.body.appendChild(debugElement);
        
        function frame() {
          analyser.getByteTimeDomainData(data);
          
          // Check if we're getting real audio data
          const sum = data.reduce((a, b) => a + b, 0);
          const avg = sum / data.length;
          console.log("Audio data avg:", avg, "samples:", data[0], data[1], data[2]);
          
          // Show some values on screen for debugging
          debugElement.textContent = `Audio data: min=${Math.min(...data)}, max=${Math.max(...data)}, avg=${avg.toFixed(2)}`;
          
          // Try to call methods with more error handling
          try {
            if (typeof viz.update === 'function') {
              viz.update(data);
            } else {
              console.error("viz.update is not a function, it's a:", typeof viz.update);
              console.log("Full viz object:", viz);
            }
            
            // Only render if we're not already rendering
            if (typeof viz.render === 'function') {
              // Use requestAnimationFrame to throttle render calls
              viz.render();
            } else {
              console.error("viz.render is not a function, it's a:", typeof viz.render);
              console.log("Full viz object:", viz);
            }
          } catch (e) {
            console.error("Error calling viz methods:", e);
            console.error("Error details:", e.stack);
          }
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
