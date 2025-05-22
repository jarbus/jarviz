import init, { Visualizer } from "./pkg/jarviz.js";

// Global variables for audio context and related objects
let audioCtx = null;
let audioSource = null;
let analyser = null;
let animationId = null;

async function run() {
  try {
    await init();
    console.log("WASM initialized successfully");
    
    const canvas = document.getElementById("gpu-canvas");
    const fileInput = document.getElementById("file-input");
    const pauseBtn = document.getElementById("pause-btn");
    
    // Since Visualizer constructor is async, we need to await it
    console.log("Creating Visualizer...");
    const vizPromise = new Visualizer("gpu-canvas");
    
    // Properly await the Promise to get the actual Visualizer instance
    const viz = await vizPromise;
    console.log("Visualizer created successfully");
    
    // Add a message to the page
    const message = document.createElement("p");
    message.id = "status-message";
    message.textContent = "Ready to visualize audio";
    document.body.insertBefore(message, canvas.nextSibling);

    // Set up pause button
    pauseBtn.addEventListener("click", () => {
      togglePause(viz, pauseBtn, message);
    });
    
    // Set up resolution slider
    const resolutionSlider = document.getElementById("resolution-slider");
    const resolutionValue = document.getElementById("resolution-value");
    
    resolutionSlider.addEventListener("input", () => {
      const value = resolutionSlider.value;
      resolutionValue.textContent = `${value} points`;
      viz.setResolution(parseInt(value));
    });
    
    // Set up keyboard listener for space key
    document.addEventListener("keydown", (event) => {
      if (event.code === "Space") {
        // Prevent default space behavior (like scrolling)
        event.preventDefault();
        togglePause(viz, pauseBtn, message);
      }
    });

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
        audioCtx = new AudioContext();
        
        const buf = await audioCtx.decodeAudioData(arrayBuffer);
        audioSource = audioCtx.createBufferSource();
        audioSource.buffer = buf;

        analyser = audioCtx.createAnalyser();
        analyser.fftSize = 2048;
        audioSource.connect(analyser);
        analyser.connect(audioCtx.destination);
        
        message.textContent = `Playing: ${file.name}`;
        audioSource.start();

        // Create a buffer for the audio data
        const data = new Uint8Array(1024);
        
        // Add a debug element to show audio values
        let debugElement = document.getElementById("debug-element");
        if (!debugElement) {
          debugElement = document.createElement("div");
          debugElement.id = "debug-element";
          debugElement.style.fontSize = "10px";
          debugElement.style.fontFamily = "monospace";
          debugElement.style.marginTop = "10px";
          document.body.appendChild(debugElement);
        }
        
        // Reset pause state when loading new audio
        if (viz.isPaused()) {
          togglePause(viz, pauseBtn, message);
        }
        
        function frame() {
          analyser.getByteTimeDomainData(data);
          
          // Show some values on screen for debugging
          debugElement.textContent = `Audio data: min=${Math.min(...data)}, max=${Math.max(...data)}, avg=${(data.reduce((a, b) => a + b, 0) / data.length).toFixed(2)}`;
          
          // Try to call methods with more error handling
          let hasError = false;
          
          try {
              viz.update(data);
              viz.render();
          } catch (e) {
            console.error("Error calling viz methods:", e);
            hasError = true;
            message.textContent = `Error: ${e.message}`;
          }
          
          // Only continue the animation if there were no errors
          if (!hasError) {
            animationId = requestAnimationFrame(frame);
          } else {
            console.log("Stopping animation due to errors");
          }
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

// Function to toggle pause state
function togglePause(viz, pauseBtn, message) {
  viz.togglePause();
  const isPaused = viz.isPaused();
  
  // Update button appearance
  pauseBtn.textContent = isPaused ? "Resume (Space)" : "Pause (Space)";
  pauseBtn.classList.toggle("paused", isPaused);
  
  // Pause/resume audio playback if context exists
  if (audioCtx) {
    if (isPaused) {
      audioCtx.suspend();
    } else {
      audioCtx.resume();
    }
  }
  
  message.textContent = isPaused ? "Audio and visualization paused" : "Audio and visualization running";
}

run();
