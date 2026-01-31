import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";

interface Prompt {
  id: string;
  text: string;
  repo: string | null;
  branch: string | null;
  timestamp: string;
}

interface SaveResult {
  success: boolean;
  message: string;
  prompt_preview: string;
  is_duplicate: boolean;
}

// DOM Elements
const hud = document.getElementById("hud")!;
const hudIdle = document.getElementById("hud-idle")!;
const closeBtn = document.getElementById("close-btn")!;
const searchInput = document.getElementById("search-input") as HTMLInputElement;
const promptsList = document.getElementById("prompts-list")!;
const promptCount = document.getElementById("prompt-count")!;
const statusDot = document.getElementById("status-dot")!;
const statusText = document.getElementById("status")!;
const saveToast = document.getElementById("save-toast")!;
const hotkeyDisplay = document.getElementById("hotkey-display")!;
const addBtn = document.getElementById("add-btn")!;

let prompts: Prompt[] = [];
let isExpanded = false;
let registeredHotkey = "Cmd+Shift+P";
let selectedIndex = -1;
let deleteConfirmId: string | null = null;
let deleteConfirmTimeout: number | null = null;
let editingId: string | null = null;
let isAddingNew = false;

// Toggle HUD expansion
async function toggleHud(expand?: boolean) {
  isExpanded = expand ?? !isExpanded;
  hud.classList.toggle("expanded", isExpanded);

  // Resize window to fit content
  try {
    await invoke("set_window_size", { expanded: isExpanded });
  } catch (e) {
    console.error("Failed to resize window:", e);
  }

  if (isExpanded) {
    searchInput.focus();
    loadPrompts();
  }
}

// Load prompts from backend
async function loadPrompts(query?: string) {
  renderLoading();

  try {
    prompts = await invoke<Prompt[]>("get_prompts", { query: query || null });
    selectedIndex = -1;
    renderPrompts();
    updatePromptCount();
  } catch (e) {
    console.error("Failed to load prompts:", e);
    renderError();
  }
}

// Render loading skeleton
function renderLoading() {
  promptsList.innerHTML = `
    <div class="loading-skeleton">
      <div class="skeleton-card"></div>
      <div class="skeleton-card"></div>
      <div class="skeleton-card"></div>
    </div>
  `;
}

// Render error state
function renderError() {
  promptsList.innerHTML = `
    <div class="error-state">
      <p>Failed to load prompts</p>
      <button class="retry-btn" onclick="location.reload()">Retry</button>
    </div>
  `;
}

// Update prompt count
async function updatePromptCount() {
  try {
    const count = await invoke<number>("get_prompt_count");
    promptCount.textContent = `${count} prompt${count !== 1 ? "s" : ""} saved`;
  } catch (e) {
    console.error("Failed to get count:", e);
  }
}

// Render prompts list
function renderPrompts() {
  if (prompts.length === 0) {
    const query = searchInput.value.trim();
    promptsList.innerHTML = `
      <div class="empty-state">
        <p>${query ? "No prompts match your search" : "No prompts saved yet"}</p>
        <p class="hint">Press <kbd>${registeredHotkey}</kbd> to save a prompt</p>
      </div>
    `;
  } else {
    promptsList.innerHTML = prompts
      .map(
        (p, index) => `
      <div class="prompt-card${index === selectedIndex ? " selected" : ""}"
           data-id="${p.id}"
           data-index="${index}"
           tabindex="0"
           role="button"
           aria-label="Prompt: ${escapeHtml(p.text.slice(0, 50))}">
        <div class="prompt-text">${escapeHtml(p.text)}</div>
        <div class="prompt-footer">
          <div class="prompt-meta">
            ${p.repo ? `<span class="repo">${escapeHtml(p.repo)}</span>` : ""}
            ${p.branch ? `<span class="branch">${escapeHtml(p.branch)}</span>` : ""}
            <span class="time">${formatTime(p.timestamp)}</span>
          </div>
          <button class="edit-btn" data-id="${p.id}" title="Edit" aria-label="Edit prompt">✎</button>
          <button class="delete-btn${deleteConfirmId === p.id ? " confirming" : ""}"
                  data-id="${p.id}"
                  title="${deleteConfirmId === p.id ? "Click again to confirm" : "Delete"}"
                  aria-label="Delete prompt">${deleteConfirmId === p.id ? "delete?" : "×"}</button>
        </div>
      </div>
    `
      )
      .join("");

    // Add click handlers for copy
    promptsList.querySelectorAll(".prompt-card").forEach((card) => {
      card.addEventListener("click", async (e) => {
        // Don't copy if clicking edit or delete button
        const target = e.target as HTMLElement;
        if (target.classList.contains("delete-btn") || target.classList.contains("edit-btn")) return;

        const id = card.getAttribute("data-id");
        const prompt = prompts.find((p) => p.id === id);
        if (prompt) {
          await copyToClipboard(prompt.text);
          card.classList.add("copied");
          showToast("Copied to clipboard", prompt.text.slice(0, 40), "success");
          setTimeout(() => card.classList.remove("copied"), 300);
        }
      });

      // Keyboard handler for cards
      card.addEventListener("keydown", async (e) => {
        const ke = e as KeyboardEvent;
        if (ke.key === "Enter" || ke.key === " ") {
          ke.preventDefault();
          const id = card.getAttribute("data-id");
          const prompt = prompts.find((p) => p.id === id);
          if (prompt) {
            await copyToClipboard(prompt.text);
            card.classList.add("copied");
            showToast("Copied to clipboard", prompt.text.slice(0, 40), "success");
            setTimeout(() => card.classList.remove("copied"), 300);
          }
        } else if (ke.key === "Delete" || ke.key === "Backspace") {
          ke.preventDefault();
          const id = card.getAttribute("data-id");
          if (id) {
            handleDeleteClick(id);
          }
        }
      });
    });

    // Add click handlers for edit
    promptsList.querySelectorAll(".edit-btn").forEach((btn) => {
      btn.addEventListener("click", (e) => {
        e.stopPropagation();
        const id = btn.getAttribute("data-id");
        if (id) {
          startEdit(id);
        }
      });
    });

    // Add click handlers for delete with confirmation
    promptsList.querySelectorAll(".delete-btn").forEach((btn) => {
      btn.addEventListener("click", (e) => {
        e.stopPropagation();
        const id = btn.getAttribute("data-id");
        if (id) {
          handleDeleteClick(id);
        }
      });
    });
  }
}

// Handle delete with confirmation
function handleDeleteClick(id: string) {
  if (deleteConfirmId === id) {
    // Second click - actually delete
    clearDeleteConfirm();
    deletePrompt(id);
  } else {
    // First click - enter confirmation mode
    clearDeleteConfirm();
    deleteConfirmId = id;
    renderPrompts();
    deleteConfirmTimeout = window.setTimeout(() => {
      clearDeleteConfirm();
      renderPrompts();
    }, 2000);
  }
}

// Clear delete confirmation state
function clearDeleteConfirm() {
  if (deleteConfirmTimeout) {
    clearTimeout(deleteConfirmTimeout);
    deleteConfirmTimeout = null;
  }
  deleteConfirmId = null;
}

// Delete a prompt
async function deletePrompt(id: string) {
  try {
    const deleted = await invoke<boolean>("delete_prompt", { id });
    if (deleted) {
      showToast("Deleted", "", "success");
      loadPrompts(searchInput.value);
    }
  } catch (e) {
    console.error("Failed to delete:", e);
    showToast("Delete failed", "", "error");
  }
}

// Copy to clipboard
async function copyToClipboard(text: string) {
  try {
    await writeText(text);
  } catch {
    // Fallback for web
    await navigator.clipboard.writeText(text);
  }
}

// Show toast notification
function showToast(message: string, preview: string, type: "success" | "error" | "duplicate" = "success") {
  const toastIcon = saveToast.querySelector(".toast-icon")!;
  const toastText = saveToast.querySelector(".toast-text")!;
  const toastPreview = saveToast.querySelector(".toast-preview")!;

  // Set icon based on type
  if (type === "success") {
    toastIcon.textContent = "✓";
    toastIcon.className = "toast-icon success";
  } else if (type === "duplicate") {
    toastIcon.textContent = "●";
    toastIcon.className = "toast-icon duplicate";
  } else {
    toastIcon.textContent = "✕";
    toastIcon.className = "toast-icon error";
  }

  toastText.textContent = message;

  if (preview) {
    toastPreview.textContent = preview.length > 50 ? preview.slice(0, 50) + "..." : preview;
    (toastPreview as HTMLElement).style.display = "block";
  } else {
    (toastPreview as HTMLElement).style.display = "none";
  }

  saveToast.classList.add("visible");
  setTimeout(() => saveToast.classList.remove("visible"), 2500);
}

// Format timestamp
function formatTime(timestamp: string): string {
  const date = new Date(timestamp);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffMins = Math.floor(diffMs / 60000);
  const diffHours = Math.floor(diffMs / 3600000);
  const diffDays = Math.floor(diffMs / 86400000);

  if (diffMins < 1) return "just now";
  if (diffMins < 60) return `${diffMins}m ago`;
  if (diffHours < 24) return `${diffHours}h ago`;
  if (diffDays < 7) return `${diffDays}d ago`;

  return date.toLocaleDateString();
}

// Escape HTML
function escapeHtml(text: string): string {
  const div = document.createElement("div");
  div.textContent = text;
  return div.innerHTML;
}

// Update status indicator
function setStatus(status: "ready" | "saving" | "error", message?: string) {
  statusDot.className = "status-dot " + status;
  statusText.textContent = message || status;
}

// Show add/edit form
function showEditor(existingText = "", promptId: string | null = null) {
  editingId = promptId;
  isAddingNew = !promptId;

  const editorHtml = `
    <div class="prompt-editor" id="prompt-editor">
      <textarea id="editor-textarea" placeholder="Enter your prompt...">${escapeHtml(existingText)}</textarea>
      <div class="editor-actions">
        <button class="cancel-btn" id="editor-cancel">Cancel</button>
        <button class="save-btn" id="editor-save">${promptId ? "Update" : "Add"}</button>
      </div>
    </div>
  `;

  // Insert at top of prompts list
  promptsList.insertAdjacentHTML("afterbegin", editorHtml);

  const textarea = document.getElementById("editor-textarea") as HTMLTextAreaElement;
  const cancelBtn = document.getElementById("editor-cancel")!;
  const saveBtn = document.getElementById("editor-save")!;

  textarea.focus();
  textarea.setSelectionRange(textarea.value.length, textarea.value.length);

  cancelBtn.addEventListener("click", hideEditor);

  saveBtn.addEventListener("click", async () => {
    const text = textarea.value.trim();
    if (!text) {
      showToast("Prompt cannot be empty", "", "error");
      return;
    }

    try {
      if (editingId) {
        // Update existing
        const updated = await invoke<boolean>("update_prompt", { id: editingId, text });
        if (updated) {
          showToast("Updated", text.slice(0, 40), "success");
        }
      } else {
        // Create new
        const created = await invoke<boolean>("create_prompt", { text });
        if (created) {
          showToast("Added", text.slice(0, 40), "success");
        } else {
          showToast("Already exists", text.slice(0, 40), "duplicate");
        }
      }
      hideEditor();
      loadPrompts(searchInput.value);
    } catch (e) {
      console.error("Failed to save:", e);
      showToast("Save failed", "", "error");
    }
  });

  // Ctrl/Cmd+Enter to save
  textarea.addEventListener("keydown", (e) => {
    if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) {
      e.preventDefault();
      saveBtn.click();
    } else if (e.key === "Escape") {
      e.preventDefault();
      hideEditor();
    }
  });
}

// Hide editor
function hideEditor() {
  const editor = document.getElementById("prompt-editor");
  if (editor) {
    editor.remove();
  }
  editingId = null;
  isAddingNew = false;
}

// Start editing a prompt
function startEdit(promptId: string) {
  const prompt = prompts.find((p) => p.id === promptId);
  if (prompt) {
    hideEditor();
    showEditor(prompt.text, promptId);
  }
}

// Window dragging
const appWindow = getCurrentWindow();

// Event Listeners - detect click vs drag
let mouseDownTime = 0;
let mouseDownPos = { x: 0, y: 0 };

hudIdle.addEventListener("mousedown", async (e) => {
  mouseDownTime = Date.now();
  mouseDownPos = { x: e.clientX, y: e.clientY };

  // Start drag immediately
  await appWindow.startDragging();
});

hudIdle.addEventListener("click", (e) => {
  const dx = Math.abs(e.clientX - mouseDownPos.x);
  const dy = Math.abs(e.clientY - mouseDownPos.y);
  const elapsed = Date.now() - mouseDownTime;

  // Only expand if it was a quick click without much movement
  if (elapsed < 200 && dx < 5 && dy < 5) {
    toggleHud(true);
  }
});

// Draggable header when expanded
const hudHeader = document.querySelector(".hud-header");
hudHeader?.addEventListener("mousedown", async () => {
  await appWindow.startDragging();
});

closeBtn.addEventListener("click", (e) => {
  e.stopPropagation();
  toggleHud(false);
});

// Add button
addBtn.addEventListener("click", (e) => {
  e.stopPropagation();
  if (!isAddingNew) {
    showEditor();
  }
});

// Debounced search
let searchTimeout: number;
searchInput.addEventListener("input", () => {
  clearTimeout(searchTimeout);
  searchTimeout = window.setTimeout(() => {
    loadPrompts(searchInput.value);
  }, 150);
});

// Keyboard shortcuts
document.addEventListener("keydown", (e) => {
  if (e.key === "Escape" && isExpanded) {
    toggleHud(false);
    return;
  }

  // Arrow key navigation when expanded
  if (isExpanded && prompts.length > 0) {
    if (e.key === "ArrowDown") {
      e.preventDefault();
      selectedIndex = Math.min(selectedIndex + 1, prompts.length - 1);
      focusSelectedCard();
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      selectedIndex = Math.max(selectedIndex - 1, 0);
      focusSelectedCard();
    }
  }
});

// Focus the selected card
function focusSelectedCard() {
  const cards = promptsList.querySelectorAll(".prompt-card");
  cards.forEach((card, i) => {
    card.classList.toggle("selected", i === selectedIndex);
  });
  if (selectedIndex >= 0 && cards[selectedIndex]) {
    (cards[selectedIndex] as HTMLElement).focus();
  }
}

// Listen for Tauri events
listen<SaveResult>("prompt-saved", (event) => {
  const result = event.payload;
  if (result.is_duplicate) {
    showToast("Already saved", result.prompt_preview, "duplicate");
  } else if (result.success) {
    showToast("Saved", result.prompt_preview, "success");
    if (isExpanded) {
      loadPrompts(searchInput.value);
    }
    updatePromptCount();
  }
  setStatus("ready");
});

listen<string>("prompt-error", (event) => {
  showToast("Error", event.payload, "error");
  setStatus("ready");
});

listen<string>("hotkey-registered", (event) => {
  registeredHotkey = event.payload;
  hotkeyDisplay.textContent = registeredHotkey;
  setStatus("ready", "ready");
});

listen<string>("hotkey-error", (event) => {
  setStatus("error", "hotkey failed");
  console.error("Hotkey error:", event.payload);
});

listen("toggle-hud", () => {
  toggleHud();
});

// Initial setup
setStatus("ready");
updatePromptCount();
