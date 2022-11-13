const invoke = window.__TAURI__.invoke;

export async function moveImageOffset(moves) {
  return await invoke("move_image_offset", {moves: moves});
}
