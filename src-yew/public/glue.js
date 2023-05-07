const invoke = window.__TAURI__.invoke;

export async function nextImage(moves) {
  return await invoke("next_image", {moves: moves});
}

export async function prevImage(moves) {
  return await invoke("prev_image", {moves: moves});
}

export async function nextDirectory() {
  return await invoke("next_directory");
}

export async function prevDirectory() {
  return await invoke("prev_directory");
}
