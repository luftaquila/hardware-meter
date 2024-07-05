const { invoke } = window.__TAURI__.tauri;

let select_device;

window.addEventListener("DOMContentLoaded", () => {
  document.querySelector("#refresh-port").addEventListener("click", async e => {
    document.querySelector("#device").innerHTML = await invoke("refresh_port", {});
  })

  document.querySelector(".config").addEventListener("change", async e => {
    await invoke("config", {});
  })
});
