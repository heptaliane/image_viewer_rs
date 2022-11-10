const invoke = window.__TAURI__.invoke;

export async function getNextImage() {
  return await invoke("get_next_image", {});
}
