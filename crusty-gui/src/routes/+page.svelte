<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  // Car control functions
  async function moveForward() {
    await invoke("forward", { speed: speed, ip: ipAddress });
  }

  async function moveBackward() {
    await invoke("backward", { speed: speed, ip: ipAddress });
  }

  async function stopCar() {
    await invoke("stop", { speed: speed, ip: ipAddress });
  }

  async function turnLeft() {
    await invoke("left", { speed: speed, ip: ipAddress });
  }

  async function turnRight() {
    await invoke("right", { speed: speed, ip: ipAddress });
  }

  // State management
  let speed = $state(50);
  let ipAddress = $state("192.168.0.2");

  // Handle key presses for keyboard control
  async function handleKeyDown(event: KeyboardEvent) {
    if (event.key === "ArrowUp") {
      await moveForward();
    } else if (event.key === "ArrowDown") {
      await moveBackward();
    } else if (event.key === "ArrowLeft") {
      await turnLeft();
    } else if (event.key === "ArrowRight") {
      await turnRight();
    } else if (event.key === " ") {
      // Spacebar
      stopCar();
    }
  }
</script>

<svelte:window on:keydown={handleKeyDown} />

<main class="container p-4 mx-auto max-w-md">
  <div class="rounded-lg p-6 shadow-lg">
    <h1 class="text-2xl font-bold text-center mb-6">Remote Car Controller</h1>

    <!-- Speed Control -->
    <div class="mb-6">
      <label for="speed" class="block text-sm font-medium mb-1">Ip</label>
      <input class="input" id="speed" type="text" bind:value={ipAddress} />
    </div>

    <!-- Speed Control -->
    <div class="mb-6">
      <label for="speed" class="block text-sm font-medium mb-1"
        >Speed: {speed}%</label
      >
      <input
        id="speed"
        type="range"
        min="0"
        max="100"
        bind:value={speed}
        class="w-full"
      />
    </div>

    <!-- Control Pad -->
    <div class="grid grid-cols-3 gap-2 mb-6">
      <!-- Top row -->
      <div></div>
      <button
        aria-label="moveForward"
        onclick={moveForward}
        class="bg-blue-500 hover:bg-blue-600 text-white py-4 rounded-md flex items-center justify-center"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          class="h-6 w-6"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M5 15l7-7 7 7"
          />
        </svg>
      </button>
      <div></div>

      <!-- Middle row -->
      <button
        aria-label="turnLeft"
        onclick={turnLeft}
        class="bg-blue-500 hover:bg-blue-600 text-white py-4 rounded-md flex items-center justify-center"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          class="h-6 w-6"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M15 19l-7-7 7-7"
          />
        </svg>
      </button>
      <button
        aria-label="stopCar"
        onclick={stopCar}
        class="bg-red-500 hover:bg-red-600 text-white py-4 rounded-md flex items-center justify-center"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          class="h-6 w-6"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M18 12H6"
          />
        </svg>
      </button>
      <button
        aria-label="turnRight"
        onclick={turnRight}
        class="bg-blue-500 hover:bg-blue-600 text-white py-4 rounded-md flex items-center justify-center"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          class="h-6 w-6"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M9 5l7 7-7 7"
          />
        </svg>
      </button>

      <!-- Bottom row -->
      <div></div>
      <button
        aria-label="moveBackward"
        onclick={moveBackward}
        class="bg-blue-500 hover:bg-blue-600 text-white py-4 rounded-md flex items-center justify-center"
      >
        <svg
          xmlns="http://www.w3.org/2000/svg"
          class="h-6 w-6"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M19 9l-7 7-7-7"
          />
        </svg>
      </button>
      <div></div>
    </div>

    <!-- Keyboard Controls Info -->
    <div class="p-3 rounded-md text-sm">
      <h2 class="font-bold mb-2">Keyboard Controls:</h2>
      <ul class="space-y-1">
        <li>↑ - Move Forward</li>
        <li>↓ - Move Backward</li>
        <li>← - Turn Left</li>
        <li>→ - Turn Right</li>
        <li>Space - Stop</li>
      </ul>
    </div>
  </div>
</main>
