import init, { Visualizer } from "./pkg/jarviz.js";

// Global variables for audio context and related objects
let audioCtx = null;
let audioSource = null;
let analyser = null;
let animationId = null;
let audioDuration = 0;
let seeking = false;
let audioBuffer = null;

async function run() {
  try {
    await init();
    console.log("WASM initialized successfully");
    
    const canvas = document.getElementById("gpu-canvas");
    const fileInput = document.getElementById("file-input");
    // Since Visualizer constructor is async, we need to await it
    
    // Set up keyboard listener for space key (desktop)
    document.addEventListener("keydown", (event) => {
      if (event.code === "Space") {
        // Prevent default space behavior (like scrolling)
        event.preventDefault();
        togglePlayPause(viz, audioCtx);
      }
    });
    
    // Set up pause button for touch devices
    const pauseBtn = document.getElementById("pause-btn");
    pauseBtn.addEventListener("click", () => {
      togglePlayPause(viz, audioCtx);
      
      // Update button appearance
      if (viz && viz.isPaused()) {
        pauseBtn.classList.add("paused");
        pauseBtn.textContent = "Resume";
      } else {
        pauseBtn.classList.remove("paused");
        pauseBtn.textContent = "Pause";
      }
    });
    
    // Helper function to toggle play/pause
    function togglePlayPause(visualizer, context) {
      if (visualizer && context) {
        visualizer.togglePause();
        if (visualizer.isPaused()) {
          context.suspend();
        } else {
          context.resume();
        }
      }
    }
    
    // Helper function to format time in MM:SS format
    function formatTime(seconds) {
      const minutes = Math.floor(seconds / 60);
      const remainingSeconds = Math.floor(seconds % 60);
      return `${minutes}:${remainingSeconds.toString().padStart(2, '0')}`;
    }
    
    // Setup seek slider
    const seekSlider = document.getElementById("seek-slider");
    const currentTimeDisplay = document.getElementById("current-time");
    const durationDisplay = document.getElementById("duration");
    
    seekSlider.addEventListener("input", function() {
      seeking = true;
      const seekPosition = audioDuration * (seekSlider.value / 100);
      currentTimeDisplay.textContent = formatTime(seekPosition);
    });
    
    seekSlider.addEventListener("change", function() {
      if (audioCtx && audioSource && audioBuffer) {
        // Stop current audio source
        audioSource.stop();
        
        // Create a new source for seeking
        audioSource = audioCtx.createBufferSource();
        audioSource.buffer = audioBuffer;
        
        // Connect the new source to the analyser
        audioSource.connect(analyser);
        analyser.connect(audioCtx.destination);
        
        // Calculate the new position
        const seekPosition = audioDuration * (seekSlider.value / 100);
        
        // Start playback from the new position
        audioSource.start(0, seekPosition);
        
        // Update the audio context's current time reference
        audioCtx.resume();
        
        // If viz is paused, unpause it
        if (viz && viz.isPaused()) {
          viz.togglePause();
          // Update pause button state
          const pauseBtn = document.getElementById("pause-btn");
          pauseBtn.classList.remove("paused");
          pauseBtn.textContent = "Pause";
        }
        
        seeking = false;
      }
    });
    console.log("Creating Visualizer...");
    const vizPromise = new Visualizer("gpu-canvas");
    
    // Properly await the Promise to get the actual Visualizer instance
    const viz = await vizPromise;
    console.log("Visualizer created successfully");
    


    fileInput.onchange = async () => {
      try {
        // Stop any previous visualization
        if (animationId) {
          cancelAnimationFrame(animationId);
        }
        if (audioSource) {
          audioSource.stop();
        }
        
        
        const file = fileInput.files[0];
        if (!file) return;
        
        const arrayBuffer = await file.arrayBuffer();
        // Create audio context with mobile-friendly settings
        const AudioContext = window.AudioContext || window.webkitAudioContext;
        audioCtx = new AudioContext({
          latencyHint: 'interactive',
          sampleRate: 44100
        });
        
        const buf = await audioCtx.decodeAudioData(arrayBuffer);
        audioBuffer = buf;  // Store buffer globally for seeking
        audioDuration = buf.duration;
        
        // Update duration display
        durationDisplay.textContent = formatTime(audioDuration);
        
        // Enable and reset the seek slider
        seekSlider.disabled = false;
        seekSlider.value = 0;
        currentTimeDisplay.textContent = "0:00";
        
        audioSource = audioCtx.createBufferSource();
        audioSource.buffer = buf;

        analyser = audioCtx.createAnalyser();
        // Use smaller FFT size on mobile for better performance
        const isMobile = /Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(navigator.userAgent);
        analyser.fftSize = isMobile ? 1024 : 2048;
        audioSource.connect(analyser);
        analyser.connect(audioCtx.destination);
        
        audioSource.start();

        // Create a buffer for the audio data (size depends on FFT size)
        const bufferSize = analyser.fftSize / 2;
        const data = new Uint8Array(bufferSize);
        
        
        // Reset pause state when loading new audio
        if (viz.isPaused()) {
          viz.togglePause();
          audioCtx.resume();
        }
        
        // Update pause button state
        const pauseBtn = document.getElementById("pause-btn");
        pauseBtn.classList.remove("paused");
        pauseBtn.textContent = "Pause";
        
        // On iOS, we need user interaction to start audio
        if (/iPhone|iPad|iPod/i.test(navigator.userAgent) && audioCtx.state === 'suspended') {
          const resumeAudio = () => {
            audioCtx.resume().then(() => {
              document.body.removeEventListener('touchstart', resumeAudio);
            });
          };
          document.body.addEventListener('touchstart', resumeAudio);
        }
        
        function frame() {
          analyser.getByteTimeDomainData(data);
          
          // Try to call methods with more error handling
          let hasError = false;
          
          try {
              viz.update(data);
              viz.render();
              
              // Update slider position if not currently seeking
              if (!seeking && audioCtx && audioSource && audioBuffer) {
                const elapsedTime = audioCtx.currentTime;
                const sliderPosition = (elapsedTime / audioDuration) * 100;
                
                // Only update if the slider position is valid
                if (sliderPosition >= 0 && sliderPosition <= 100) {
                  seekSlider.value = sliderPosition;
                  currentTimeDisplay.textContent = formatTime(elapsedTime);
                }
              }
          } catch (e) {
            console.error("Error calling viz methods:", e);
            hasError = true;
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
          cancelAnimationFrame(animationId);
          
            console.log("ended")
          // Reset the slider to the end position
          seekSlider.value = 100;
          currentTimeDisplay.textContent = formatTime(audioDuration);
          
          // Update pause button state
          const pauseBtn = document.getElementById("pause-btn");
          pauseBtn.classList.add("paused");
          pauseBtn.textContent = "Resume";
          
          // Pause the visualization
          if (viz && !viz.isPaused()) {
            viz.togglePause();
          }
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
