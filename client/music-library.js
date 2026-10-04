(() => {
  const DB_NAME = "larptrix-music-library";
  const DB_VERSION = 1;
  const STORE = "tracks";
  const MAX_FILE_SIZE = 200 * 1024 * 1024;

  const els = {};
  let dbPromise;
  let tracks = [];
  let filtered = [];
  let currentIndex = -1;
  let currentUrl = null;
  let shuffle = false;
  let repeat = false;
  let savedPosition = 0;
  let restoredPlayback = false;

  function savePlaybackState() {
    const track = tracks[currentIndex];
    try {
      localStorage.setItem("larptrix_music_state", JSON.stringify({
        trackId: track?.id || null,
        currentTime: Number.isFinite(els.audio.currentTime) ? els.audio.currentTime : 0,
        volume: Number(els.volume.value),
        shuffle,
        repeat,
      }));
    } catch {}
  }

  function loadPlaybackState() {
    try {
      const state = JSON.parse(localStorage.getItem("larptrix_music_state") || "null");
      if (!state || typeof state !== "object") return;
      savedPosition = Number.isFinite(state.currentTime) ? Math.max(0, state.currentTime) : 0;
      if (typeof state.volume === "number" && Number.isFinite(state.volume)) {
        els.volume.value = String(Math.min(1, Math.max(0, state.volume)));
      }
      shuffle = state.shuffle === true;
      repeat = state.repeat === true;
      return typeof state.trackId === "string" ? state.trackId : null;
    } catch {
      return null;
    }
  }

  const $ = (id) => document.getElementById(id);

  function openDb() {
    if (dbPromise) return dbPromise;
    dbPromise = new Promise((resolve, reject) => {
      const request = indexedDB.open(DB_NAME, DB_VERSION);
      request.onupgradeneeded = () => {
        const db = request.result;
        if (!db.objectStoreNames.contains(STORE)) {
          const store = db.createObjectStore(STORE, { keyPath: "id" });
          store.createIndex("addedAt", "addedAt");
        }
      };
      request.onsuccess = () => resolve(request.result);
      request.onerror = () => reject(request.error);
    });
    return dbPromise;
  }

  async function transaction(mode, action) {
    const db = await openDb();
    return new Promise((resolve, reject) => {
      const tx = db.transaction(STORE, mode);
      const store = tx.objectStore(STORE);
      let request;
      try {
        request = action(store);
      } catch (error) {
        reject(error);
        return;
      }
      tx.oncomplete = () => resolve(request?.result);
      tx.onerror = () => reject(tx.error || request?.error);
      tx.onabort = () => reject(tx.error || new Error("Music database transaction aborted."));
    });
  }

  async function listTracks() {
    const db = await openDb();
    return new Promise((resolve, reject) => {
      const request = db.transaction(STORE, "readonly").objectStore(STORE).index("addedAt").getAll();
      request.onsuccess = () => resolve(request.result.sort((a, b) => b.addedAt - a.addedAt));
      request.onerror = () => reject(request.error);
    });
  }

  const formatTime = (seconds) => {
    if (!Number.isFinite(seconds) || seconds < 0) return "0:00";
    const value = Math.floor(seconds);
    const minutes = Math.floor(value / 60);
    return `${minutes}:${String(value % 60).padStart(2, "0")}`;
  };

  const formatSize = (bytes) => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
  };

  const trackDuration = (track) => Number.isFinite(track.duration) ? track.duration : 0;

  function updateStats() {
    const count = tracks.length;
    const total = tracks.reduce((sum, track) => sum + trackDuration(track), 0);
    els.count.textContent = `${count} ${count === 1 ? "track" : "tracks"}`;
    els.total.textContent = formatTime(total);
    els.clear.hidden = count === 0;
  }

  function render() {
    const query = els.search.value.trim().toLowerCase();
    filtered = tracks.filter((track) => track.name.toLowerCase().includes(query));
    els.list.replaceChildren();

    filtered.forEach((track, index) => {
      const originalIndex = tracks.findIndex((item) => item.id === track.id);
      const li = document.createElement("li");
      li.className = "music-track";
      if (originalIndex === currentIndex) li.classList.add("playing");

      const number = document.createElement("span");
      number.className = "music-track-number";
      number.textContent = originalIndex === currentIndex ? "♫" : String(index + 1).padStart(2, "0");

      const copy = document.createElement("div");
      copy.className = "music-track-copy";
      const name = document.createElement("strong");
      name.textContent = track.name;
      const meta = document.createElement("span");
      meta.textContent = `${formatTime(track.duration)} · ${formatSize(track.size)}`;
      copy.append(name, meta);

      const size = document.createElement("span");
      size.className = "music-track-size";
      size.textContent = new Date(track.addedAt).toLocaleDateString();

      const play = document.createElement("button");
      play.type = "button";
      play.className = "music-track-play";
      play.title = originalIndex === currentIndex && !els.audio.paused ? "Pause" : "Play";
      play.textContent = originalIndex === currentIndex && !els.audio.paused ? "Ⅱ" : "▶";
      play.addEventListener("click", () => playTrack(originalIndex));

      const remove = document.createElement("button");
      remove.type = "button";
      remove.className = "music-track-remove";
      remove.title = "Remove from library";
      remove.textContent = "×";
      remove.addEventListener("click", async () => removeTrack(track.id));

      li.append(number, copy, size, play, remove);
      els.list.append(li);
    });

    els.empty.hidden = filtered.length !== 0;
    els.empty.textContent = query
      ? "No tracks match your search."
      : "Your library is empty. Add some music.";
    updateStats();
  }

  async function refresh() {
    tracks = await listTracks();
    if (currentIndex >= tracks.length) currentIndex = -1;
    const savedTrackId = loadPlaybackState();
    if (savedTrackId && !restoredPlayback) {
      const index = tracks.findIndex((track) => track.id === savedTrackId);
      if (index >= 0) {
        currentIndex = index;
        const track = tracks[index];
        if (currentUrl) URL.revokeObjectURL(currentUrl);
        currentUrl = URL.createObjectURL(track.blob);
        els.audio.src = currentUrl;
        els.audio.volume = Number(els.volume.value);
        els.name.textContent = track.name;
        els.meta.textContent = `${track.filename} · ${formatSize(track.size)}`;
        els.player.hidden = false;
        els.audio.addEventListener("loadedmetadata", () => {
          if (savedPosition > 0 && Number.isFinite(els.audio.duration)) {
            els.audio.currentTime = Math.min(savedPosition, Math.max(0, els.audio.duration - 0.1));
          }
          restoredPlayback = true;
          updatePlayer();
        }, { once: true });
      } else {
        restoredPlayback = true;
      }
    }
    render();
  }

  async function addFiles(fileList) {
    const files = [...fileList].filter((file) => file.type.startsWith("audio/"));
    if (!files.length) return;

    for (const file of files) {
      if (file.size > MAX_FILE_SIZE) {
        alert(`Skipping ${file.name}: maximum size is 200 MB.`);
        continue;
      }
      const duration = await readDuration(file).catch(() => 0);
      const track = {
        id: crypto.randomUUID(),
        name: file.name.replace(/\.[^.]+$/, ""),
        filename: file.name,
        type: file.type || "audio/mpeg",
        size: file.size,
        duration,
        addedAt: Date.now(),
        blob: file,
      };
      await transaction("readwrite", (store) => store.put(track));
    }

    await refresh();
  }

  function readDuration(file) {
    return new Promise((resolve, reject) => {
      const url = URL.createObjectURL(file);
      const audio = document.createElement("audio");
      audio.preload = "metadata";
      audio.onloadedmetadata = () => {
        const duration = Number.isFinite(audio.duration) ? audio.duration : 0;
        URL.revokeObjectURL(url);
        resolve(duration);
      };
      audio.onerror = () => {
        URL.revokeObjectURL(url);
        reject(new Error("Could not read audio metadata."));
      };
      audio.src = url;
    });
  }

  async function removeTrack(id) {
    const index = tracks.findIndex((track) => track.id === id);
    if (index < 0) return;
    if (!confirm(`Remove “${tracks[index].name}” from your local library?`)) return;

    if (currentIndex === index) {
      els.audio.pause();
      els.audio.removeAttribute("src");
      if (currentUrl) URL.revokeObjectURL(currentUrl);
      currentUrl = null;
      currentIndex = -1;
      updatePlayer();
    } else if (currentIndex > index) {
      currentIndex -= 1;
    }

    await transaction("readwrite", (store) => store.delete(id));
    await refresh();
  }

  function chooseNext(direction) {
    if (!tracks.length) return -1;
    if (shuffle) return Math.floor(Math.random() * tracks.length);
    if (currentIndex < 0) return direction > 0 ? 0 : tracks.length - 1;
    return (currentIndex + direction + tracks.length) % tracks.length;
  }

  function loadTrack(index, autoplay = true) {
    if (index < 0 || index >= tracks.length) return;
    currentIndex = index;
    savedPosition = 0;
    const track = tracks[index];
    if (currentUrl) URL.revokeObjectURL(currentUrl);
    currentUrl = URL.createObjectURL(track.blob);
    els.audio.src = currentUrl;
    els.audio.volume = Number(els.volume.value);
    els.name.textContent = track.name;
    els.meta.textContent = `${track.filename} · ${formatSize(track.size)}`;
    els.player.hidden = false;
    if (autoplay) {
      els.audio.play().catch(() => {});
    }
    updatePlayer();
    render();
  }

  function playTrack(index) {
    if (currentIndex === index && els.audio.src) {
      if (els.audio.paused) els.audio.play().catch(() => {});
      else els.audio.pause();
      return;
    }
    loadTrack(index, true);
  }

  function updatePlayer() {
    if (currentIndex < 0 || !tracks[currentIndex]) {
      els.player.hidden = true;
      window.larptixMusicStatus?.update?.("", false);
      els.play.textContent = "▶";
      els.time.textContent = "0:00 / 0:00";
      return;
    }
    const track = tracks[currentIndex];
    els.player.hidden = false;
    els.name.textContent = track.name;
    els.meta.textContent = `${track.filename} · ${formatSize(track.size)}`;
    els.play.textContent = els.audio.paused ? "▶" : "Ⅱ";
    els.shuffle.setAttribute("aria-pressed", String(shuffle));
    els.repeat.setAttribute("aria-pressed", String(repeat));
  }

  function next(direction = 1) {
    const index = chooseNext(direction);
    if (index >= 0) loadTrack(index, true);
  }

  function clampPosition(x, y) {
    const rect = els.library.getBoundingClientRect();
    return {
      x: Math.min(Math.max(8, x), Math.max(8, window.innerWidth - rect.width - 8)),
      y: Math.min(Math.max(8, y), Math.max(8, window.innerHeight - rect.height - 8)),
    };
  }

  function savePosition(x, y) {
    try {
      localStorage.setItem("larptrix_music_window_position", JSON.stringify({ x, y }));
    } catch {}
  }

  function applySavedPosition() {
    let saved = null;
    try {
      saved = JSON.parse(localStorage.getItem("larptrix_music_window_position") || "null");
    } catch {}
    if (!saved || !Number.isFinite(saved.x) || !Number.isFinite(saved.y)) return;
    const position = clampPosition(saved.x, saved.y);
    els.library.style.left = `${position.x}px`;
    els.library.style.top = `${position.y}px`;
    els.library.style.right = "auto";
    els.library.style.bottom = "auto";
  }

  function wireDragging() {
    const handle = $("music-library-drag-area");
    if (!handle) return;

    let dragging = false;
    let pointerId = null;
    let offsetX = 0;
    let offsetY = 0;

    const stopDragging = (event) => {
      if (!dragging) return;
      dragging = false;
      handle.classList.remove("dragging");
      if (pointerId !== null && handle.hasPointerCapture?.(pointerId)) {
        handle.releasePointerCapture(pointerId);
      }
      pointerId = null;
      const rect = els.library.getBoundingClientRect();
      savePosition(rect.left, rect.top);
      event?.preventDefault?.();
    };

    handle.addEventListener("pointerdown", (event) => {
      if (
        event.button !== 0 ||
        event.target.closest("button,input,select,textarea,a,label")
      ) return;
      const rect = els.library.getBoundingClientRect();
      dragging = true;
      pointerId = event.pointerId;
      offsetX = event.clientX - rect.left;
      offsetY = event.clientY - rect.top;
      handle.setPointerCapture?.(event.pointerId);
      handle.classList.add("dragging");
      event.preventDefault();
    });

    handle.addEventListener("pointermove", (event) => {
      if (!dragging) return;
      const position = clampPosition(event.clientX - offsetX, event.clientY - offsetY);
      els.library.style.left = `${position.x}px`;
      els.library.style.top = `${position.y}px`;
      els.library.style.right = "auto";
      els.library.style.bottom = "auto";
    });

    handle.addEventListener("pointerup", stopDragging);
    handle.addEventListener("pointercancel", stopDragging);
    handle.addEventListener("lostpointercapture", () => stopDragging());
  }


  function wire() {
    els.library = $("music-library");
    els.close = $("music-library-close");
    els.file = $("music-library-file");
    els.search = $("music-library-search");
    els.refresh = $("music-library-refresh");
    els.clear = $("music-library-clear");
    els.dropzone = $("music-library-dropzone");
    els.list = $("music-track-list");
    els.empty = $("music-library-empty");
    els.count = $("music-library-count");
    els.total = $("music-library-total");
    els.player = $("music-player");
    els.audio = $("music-audio");
    els.name = $("music-player-name");
    els.meta = $("music-player-meta");
    els.play = $("music-play");
    els.prev = $("music-prev");
    els.next = $("music-next");
    els.progress = $("music-progress");
    els.time = $("music-player-time");
    els.shuffle = $("music-shuffle");
    els.repeat = $("music-repeat");
    els.volume = $("music-volume");

    els.close?.addEventListener("click", () => setMode(false));
    els.file.addEventListener("change", () => {
      void addFiles(els.file.files);
      els.file.value = "";
    });
    els.search.addEventListener("input", render);
    els.refresh.addEventListener("click", () => void refresh());
    els.clear.addEventListener("click", async () => {
      if (!tracks.length || !confirm("Remove every track from your local library?")) return;
      await transaction("readwrite", (store) => store.clear());
      els.audio.pause();
      els.audio.removeAttribute("src");
      if (currentUrl) URL.revokeObjectURL(currentUrl);
      currentUrl = null;
      currentIndex = -1;
      await refresh();
      updatePlayer();
    });

    ["dragenter", "dragover"].forEach((type) => els.dropzone.addEventListener(type, (event) => {
      event.preventDefault();
      els.dropzone.classList.add("dragging");
    }));
    ["dragleave", "drop"].forEach((type) => els.dropzone.addEventListener(type, (event) => {
      event.preventDefault();
      els.dropzone.classList.remove("dragging");
    }));
    els.dropzone.addEventListener("drop", (event) => void addFiles(event.dataTransfer.files));

    els.play.addEventListener("click", () => {
      if (currentIndex < 0) {
        next(1);
      } else if (els.audio.paused) {
        els.audio.play().catch(() => {});
      } else {
        els.audio.pause();
      }
    });
    els.prev.addEventListener("click", () => next(-1));
    els.next.addEventListener("click", () => next(1));
    els.shuffle.addEventListener("click", () => {
      shuffle = !shuffle;
      savePlaybackState();
      updatePlayer();
    });
    els.repeat.addEventListener("click", () => {
      repeat = !repeat;
      savePlaybackState();
      updatePlayer();
    });
    els.volume.addEventListener("input", () => {
      els.audio.volume = Number(els.volume.value);
      savePlaybackState();
    });
    els.progress.addEventListener("input", () => {
      if (els.audio.duration) {
        els.audio.currentTime = (Number(els.progress.value) / 1000) * els.audio.duration;
      }
    });
    els.audio.addEventListener("timeupdate", () => {
      savePlaybackState();
      const duration = els.audio.duration || trackDuration(tracks[currentIndex] || {});
      const current = els.audio.currentTime || 0;
      els.progress.value = duration ? Math.round((current / duration) * 1000) : 0;
      els.time.textContent = `${formatTime(current)} / ${formatTime(duration)}`;
    });
    els.audio.addEventListener("play", () => {
      savePlaybackState();
      els.play.textContent = "Ⅱ";
      const track = tracks[currentIndex];
      if (track) window.larptixMusicStatus?.update?.(track.name, true);
      render();
    });
    els.audio.addEventListener("pause", () => {
      savePlaybackState();
      els.play.textContent = "▶";
      const track = tracks[currentIndex];
      if (track) window.larptixMusicStatus?.update?.(track.name, false);
      render();
    });
    els.audio.addEventListener("ended", () => {
      if (repeat) {
        els.audio.currentTime = 0;
        els.audio.play().catch(() => {});
      } else {
        next(1);
      }
    });

    wireDragging();
    applySavedPosition();

    setMode(false);
    window.addEventListener("resize", () => {
      if (!els.library.hidden) {
        const rect = els.library.getBoundingClientRect();
        const position = clampPosition(rect.left, rect.top);
        els.library.style.left = `${position.x}px`;
        els.library.style.top = `${position.y}px`;
        els.library.style.right = "auto";
        els.library.style.bottom = "auto";
        savePosition(position.x, position.y);
      }
      if (!els.player.hidden) {
        const rect = els.player.getBoundingClientRect();
        const position = clampPlayerPosition(rect.left, rect.top);
        els.player.style.left = position.x + "px";
        els.player.style.top = position.y + "px";
        els.player.style.right = "auto";
        els.player.style.bottom = "auto";
        savePlayerPosition(position.x, position.y);
      }
    });
    void refresh();
  }

  function setMode(libraryMode) {
    els.library.hidden = !libraryMode;
    if (libraryMode) applySavedPosition();
  }

  window.larptixMusicLibrary = {
    open: () => setMode(true),
    close: () => setMode(false),
    toggle: () => setMode(els.library.hidden),
  };

  document.addEventListener("DOMContentLoaded", wire, { once: true });
})();