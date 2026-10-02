(() => {
  const root = document.documentElement;
  const resizer = document.getElementById("chat-resizer");
  if (!resizer) return;

  const storageKey = "larptrix_chat_list_width";
  const min = 220;
  const max = 520;

  const clamp = (value) => Math.max(min, Math.min(max, value));
  const saved = Number.parseInt(localStorage.getItem(storageKey) || "", 10);
  if (Number.isFinite(saved)) root.style.setProperty("--chat-list-width", `${clamp(saved)}px`);

  let startX = 0;
  let startWidth = 0;

  const stop = () => {
    document.body.classList.remove("is-resizing");
    window.removeEventListener("pointermove", move);
    window.removeEventListener("pointerup", stop);
    window.removeEventListener("pointercancel", stop);
  };

  const move = (event) => {
    const width = clamp(startWidth + event.clientX - startX);
    root.style.setProperty("--chat-list-width", `${width}px`);
  };

  resizer.addEventListener("pointerdown", (event) => {
    if (window.matchMedia("(max-width: 760px)").matches) return;
    startX = event.clientX;
    startWidth = parseInt(getComputedStyle(root).getPropertyValue("--chat-list-width"), 10) || 280;
    document.body.classList.add("is-resizing");
    resizer.setPointerCapture?.(event.pointerId);
    window.addEventListener("pointermove", move);
    window.addEventListener("pointerup", stop);
    window.addEventListener("pointercancel", stop);
  });

  resizer.addEventListener("keydown", (event) => {
    const current = parseInt(getComputedStyle(root).getPropertyValue("--chat-list-width"), 10) || 280;
    if (event.key === "ArrowLeft" || event.key === "ArrowRight") {
      event.preventDefault();
      const next = clamp(current + (event.key === "ArrowRight" ? 20 : -20));
      root.style.setProperty("--chat-list-width", `${next}px`);
      localStorage.setItem(storageKey, String(next));
    } else if (event.key === "Home") {
      event.preventDefault();
      root.style.setProperty("--chat-list-width", `${min}px`);
      localStorage.setItem(storageKey, String(min));
    } else if (event.key === "End") {
      event.preventDefault();
      root.style.setProperty("--chat-list-width", `${max}px`);
      localStorage.setItem(storageKey, String(max));
    }
  });

  window.addEventListener("pointerup", () => {
    if (!document.body.classList.contains("is-resizing")) return;
    const width = parseInt(getComputedStyle(root).getPropertyValue("--chat-list-width"), 10) || 280;
    localStorage.setItem(storageKey, String(clamp(width)));
  });
})();
