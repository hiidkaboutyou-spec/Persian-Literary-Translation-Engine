"use strict";

const invoke = window.__TAURI__.core.invoke;

const state = {
  projectRoot: null,
  sourcePath: null,
  snapshot: null,
  translationRunning: false,
  progressTimer: null,
  literaryEvidence: null,
  currentView: "home",
  theme: "system",
  commandIndex: 0,
  editorFocus: false,
};

const $ = (id) => document.getElementById(id);

const VIEW_META = {
  home: { title: "Project", subtitle: "Open or create a translation project.", eyebrow: "Workspace", shortcut: "⌘1", icon: "⌂" },
  workflow: { title: "Workflow", subtitle: "Explicit analysis, translation and publication stages.", eyebrow: "Pipeline", shortcut: "⌘2", icon: "↝" },
  editor: { title: "Translation Editor", subtitle: "Read source and Persian side by side, then save bounded revisions.", eyebrow: "Translation", shortcut: "⌘3", icon: "✎" },
  review: { title: "Intelligence Review", subtitle: "Human decisions over extracted literary intelligence.", eyebrow: "Literary intelligence", shortcut: "⌘4", icon: "◇" },
  canon: { title: "Canon", subtitle: "Character voice and terminology that guide long-form continuity.", eyebrow: "Literary intelligence", shortcut: "⌘5", icon: "☷" },
  literary: { title: "Literary Review", subtitle: "Post-translation fidelity and Persian-naturalness evidence.", eyebrow: "Literary intelligence", shortcut: "⌘6", icon: "✦" },
  history: { title: "History", subtitle: "Bounded audit history emitted by the application layer.", eyebrow: "System", shortcut: "⌘7", icon: "↺" },
  provider: { title: "Provider", subtitle: "Session-only provider configuration; secrets are never stored in projects.", eyebrow: "System", shortcut: "⌘8", icon: "⌁" },
};

const THEME_META = {
  system: { label: "System", subtitle: "Follow macOS appearance", icon: "◐" },
  midnight: { label: "Midnight Ink", subtitle: "Deep plum editorial glass", icon: "●" },
  paper: { label: "Rose Paper", subtitle: "Warm paper and dusty rose", icon: "◒" },
  sage: { label: "Sage Manuscript", subtitle: "Soft green reading room", icon: "◓" },
};

const prefersReducedMotion = () => window.matchMedia("(prefers-reduced-motion: reduce)").matches;

function withViewTransition(update) {
  if (!prefersReducedMotion() && typeof document.startViewTransition === "function") {
    document.startViewTransition(update);
  } else {
    update();
  }
}

function setTheme(theme) {
  if (!THEME_META[theme]) return;
  state.theme = theme;
  document.body.dataset.theme = theme;
  $("theme-name").textContent = THEME_META[theme].label;
  document.querySelectorAll(".theme-swatch").forEach((button) => {
    const active = button.dataset.themeValue === theme;
    button.classList.toggle("active", active);
    button.setAttribute("aria-pressed", String(active));
  });
}

function showNotice(message, kind = "success") {
  const node = $("notice");
  node.textContent = message;
  node.className = "notice " + kind;
  window.clearTimeout(showNotice.timer);
  showNotice.timer = window.setTimeout(() => {
    node.className = "notice hidden";
  }, 6500);
}

function errorMessage(error) {
  if (error && typeof error === "object") {
    const hint = error.recovery_hint && error.recovery_hint !== "none"
      ? " · recovery: " + error.recovery_hint
      : "";
    return (error.message || error.code || JSON.stringify(error)) + hint;
  }
  return String(error);
}

async function call(command, args = {}) {
  try {
    return await invoke(command, args);
  } catch (error) {
    showNotice(errorMessage(error), "error");
    throw error;
  }
}

function textNode(tag, value, className) {
  const node = document.createElement(tag);
  if (className) node.className = className;
  node.textContent = value == null ? "" : String(value);
  return node;
}

function setView(name) {
  const meta = VIEW_META[name] || VIEW_META.home;
  state.currentView = name;
  withViewTransition(() => {
    document.querySelectorAll(".nav-item").forEach((button) => {
      const active = button.dataset.view === name;
      button.classList.toggle("active", active);
      if (active) button.setAttribute("aria-current", "page");
      else button.removeAttribute("aria-current");
    });
    document.querySelectorAll("[data-view-panel]").forEach((panel) => {
      panel.classList.toggle("active", panel.dataset.viewPanel === name);
    });
    $("view-title").textContent = meta.title;
    $("view-subtitle").textContent = meta.subtitle;
    $("view-eyebrow").textContent = meta.eyebrow;
  });
  const activePanel = document.querySelector('[data-view-panel="' + name + '"]');
  if (activePanel) activePanel.scrollIntoView({ block: "start", behavior: prefersReducedMotion() ? "auto" : "smooth" });
}

function setProjectEnabled(enabled) {
  [
    "refresh-project", "verify-source", "pick-import-source", "run-analysis", "run-advanced",
    "start-translation", "resume-translation", "pause-translation", "export-project",
    "refresh-review", "refresh-characters", "save-character", "refresh-glossary",
    "save-glossary", "load-chapter", "run-literary-review", "load-literary-review",
    "refresh-history"
  ].forEach((id) => {
    $(id).disabled = !enabled;
  });
  $("import-source").disabled = !enabled || !state.sourcePath;
}

function snapshotItem(label, value) {
  const item = document.createElement("div");
  item.className = "snapshot-item";
  item.append(textNode("small", label), textNode("strong", value));
  return item;
}

function renderSnapshot(snapshot) {
  state.snapshot = snapshot;
  $("current-project").textContent = snapshot ? snapshot.name : "None";
  $("status-pill").textContent = snapshot ? snapshot.status : "No project";
  $("status-pill").className = snapshot ? "status-pill" : "status-pill muted";

  const grid = $("snapshot-grid");
  grid.replaceChildren();
  if (!snapshot) {
    grid.className = "snapshot-grid empty-state";
    grid.textContent = "Open a project to see its state.";
    $("warnings").replaceChildren();
    return;
  }
  grid.className = "snapshot-grid";
  grid.append(
    snapshotItem("Status", snapshot.status),
    snapshotItem("Next action", snapshot.next_action),
    snapshotItem("Target language", snapshot.target_language),
    snapshotItem("Source", snapshot.source ? snapshot.source.title : "Not imported"),
    snapshotItem("Review", snapshot.review.total + " total · " + snapshot.review.pending + " pending"),
    snapshotItem("Canon", snapshot.canon.characters + " characters · " + snapshot.canon.glossary_entries + " terms"),
    snapshotItem(
      "Translation",
      snapshot.translation
        ? snapshot.translation.completed_chapters + "/" + snapshot.translation.total_chapters + " chapters"
        : "Not started"
    ),
    snapshotItem("Export", snapshot.export ? snapshot.export.format + " · " + snapshot.export.relative_path : "Not created"),
  );

  const warnings = $("warnings");
  warnings.replaceChildren();
  (snapshot.warnings || []).forEach((warning) => {
    warnings.append(textNode("div", warning, "warning"));
  });

  if (snapshot.translation) {
    renderProgress(snapshot.translation);
  }
}

async function refreshSnapshot() {
  if (!state.projectRoot) return;
  const snapshot = await call("open_project", { projectRoot: state.projectRoot });
  renderSnapshot(snapshot);
}

function renderProgress(progress) {
  const percent = Math.max(0, Math.min(1, Number(progress.percent || 0)));
  $("progress-bar").style.width = (percent * 100).toFixed(1) + "%";
  const completed = progress.completed_chapters ?? 0;
  const total = progress.total_chapters ?? 0;
  $("progress-label").textContent = (progress.state || "unknown") + " · " + completed + "/" + total;
}

function numberOrNull(id) {
  const value = $(id).value.trim();
  if (!value) return null;
  const number = Number(value);
  return Number.isFinite(number) && number > 0 ? Math.floor(number) : null;
}

function translationInput() {
  return {
    provider: $("translation-provider").value,
    model: $("translation-model").value.trim() || null,
    targetLanguage: $("target-language").value.trim() || "fa",
    styleProfile: $("style-profile").value,
    adultContentConfirmed: $("adult-confirmed").checked,
    maxChapters: numberOrNull("translation-max"),
  };
}

async function pollProgress() {
  if (!state.projectRoot) return;
  try {
    const progress = await invoke("get_progress", { projectRoot: state.projectRoot });
    renderProgress(progress);
  } catch (_) {
    // A progress artifact may not exist during the first instant of a run.
  }
}

function startProgressPolling() {
  stopProgressPolling();
  state.progressTimer = window.setInterval(pollProgress, 1000);
}

function stopProgressPolling() {
  if (state.progressTimer) {
    window.clearInterval(state.progressTimer);
    state.progressTimer = null;
  }
}

async function runTranslation(command) {
  if (!state.projectRoot || state.translationRunning) return;
  state.translationRunning = true;
  startProgressPolling();
  try {
    const progress = await call(command, {
      projectRoot: state.projectRoot,
      input: translationInput(),
    });
    renderProgress(progress);
    await refreshSnapshot();
    showNotice(command === "start_translation" ? "Translation run completed." : "Translation resume completed.");
  } finally {
    state.translationRunning = false;
    stopProgressPolling();
  }
}

function renderReviewItems(items) {
  const list = $("review-list");
  list.replaceChildren();
  if (!items.length) {
    list.className = "list empty-state";
    list.textContent = "No matching review items.";
    return;
  }
  list.className = "list";
  items.forEach((item) => {
    const row = document.createElement("div");
    row.className = "list-row";
    row.append(textNode("strong", item.subject));
    row.append(textNode("div", item.kind + " · " + item.status + " · rev " + item.revision + " · " + item.evidence_count + " evidence", "meta"));
    const actions = document.createElement("div");
    actions.className = "actions";
    const candidates = item.status === "pending"
      ? [["approve", "Approve"], ["reject", "Reject"], ["defer", "Defer"]]
      : [["reopen", "Reopen"]];
    candidates.forEach(([action, label]) => {
      const button = textNode("button", label);
      button.addEventListener("click", async () => {
        const reviewer = $("reviewer-name").value.trim() || "desktop-user";
        await call("decide_review_item", {
          projectRoot: state.projectRoot,
          itemId: item.id,
          action,
          reviewer,
          reason: "desktop review decision",
        });
        await loadReviewItems();
        await refreshSnapshot();
      });
      actions.append(button);
    });
    row.append(actions);
    list.append(row);
  });
}

async function loadReviewItems() {
  if (!state.projectRoot) return;
  const status = $("review-status-filter").value || null;
  const items = await call("list_review_items", {
    projectRoot: state.projectRoot,
    kindFilter: null,
    statusFilter: status,
  });
  renderReviewItems(items);
}

function renderCharacters(items) {
  const list = $("character-list");
  list.replaceChildren();
  if (!items.length) {
    list.append(textNode("div", "No canonical characters yet.", "empty-state"));
    return;
  }
  items.forEach((item) => {
    const row = document.createElement("button");
    row.className = "list-row";
    row.append(textNode("strong", item.name));
    row.append(textNode("div", item.voice_notes || "No voice notes", "meta"));
    row.addEventListener("click", () => {
      $("character-name").value = item.name;
      $("character-voice").value = item.voice_notes;
      $("character-personality").value = item.personality_notes;
    });
    list.append(row);
  });
}

async function loadCharacters() {
  if (!state.projectRoot) return;
  renderCharacters(await call("list_characters", { projectRoot: state.projectRoot }));
}

function renderGlossary(items) {
  const list = $("glossary-list");
  list.replaceChildren();
  if (!items.length) {
    list.append(textNode("div", "No canonical terminology yet.", "empty-state"));
    return;
  }
  items.forEach((item) => {
    const row = document.createElement("button");
    row.className = "list-row";
    row.append(textNode("strong", item.source_term + " → " + item.preferred_translation));
    row.append(textNode("div", item.context || "No context note", "meta"));
    row.addEventListener("click", () => {
      $("glossary-source").value = item.source_term;
      $("glossary-target").value = item.preferred_translation;
      $("glossary-context").value = item.context;
    });
    list.append(row);
  });
}

async function loadGlossary() {
  if (!state.projectRoot) return;
  renderGlossary(await call("list_glossary_entries", { projectRoot: state.projectRoot }));
}

function renderChapter(chapter) {
  $("chapter-meta").replaceChildren(
    textNode("p", chapter.title + " · " + chapter.paragraphs.length + " paragraphs · style " + chapter.style_profile)
  );
  const editor = $("paragraph-editor");
  editor.replaceChildren();
  editor.className = state.editorFocus ? "paragraph-editor focus-persian" : "paragraph-editor";

  chapter.paragraphs.forEach((paragraph, index) => {
    const block = document.createElement("div");
    block.className = "paragraph";

    const sourcePane = document.createElement("div");
    sourcePane.className = "paragraph-pane source-pane";
    const sourceKicker = document.createElement("div");
    sourceKicker.className = "paragraph-kicker";
    sourceKicker.append(textNode("span", "Source"), textNode("span", "#" + (index + 1)));
    sourcePane.append(sourceKicker, textNode("div", paragraph.source, "source-text"));

    const translationPane = document.createElement("div");
    translationPane.className = "paragraph-pane translation-pane";
    const translationKicker = document.createElement("div");
    translationKicker.className = "paragraph-kicker";
    translationKicker.append(textNode("span", "Persian revision"), textNode("span", "editable"));

    const textarea = document.createElement("textarea");
    textarea.className = "translation-text";
    textarea.dir = "rtl";
    textarea.value = paragraph.translated;
    textarea.setAttribute("aria-label", "Persian translation paragraph " + (index + 1));

    const saveRow = document.createElement("div");
    saveRow.className = "save-row";
    const save = textNode("button", "Saved");
    save.disabled = true;

    textarea.addEventListener("input", () => {
      const dirty = textarea.value !== paragraph.translated;
      block.classList.toggle("dirty", dirty);
      save.disabled = !dirty;
      save.textContent = dirty ? "Save revision" : "Saved";
    });

    save.addEventListener("click", async () => {
      if (textarea.value === paragraph.translated) return;
      const chapterIndex = Number($("editor-chapter-index").value) - 1;
      await call("apply_manual_edit", {
        projectRoot: state.projectRoot,
        chapterIndex,
        paragraphId: paragraph.paragraph_id,
        newText: textarea.value,
        reviewer: $("reviewer-name").value.trim() || "desktop-user",
      });
      paragraph.translated = textarea.value;
      block.classList.remove("dirty");
      save.disabled = true;
      save.textContent = "Saved";
      showNotice("Manual revision saved; quality evidence is now stale until reviewed again.");
    });

    saveRow.append(save);
    translationPane.append(translationKicker, textarea, saveRow);
    block.append(sourcePane, translationPane);
    editor.append(block);
  });
}

function renderLiteraryEvidence(evidence) {
  state.literaryEvidence = evidence;
  const output = $("literary-output");
  output.replaceChildren();
  output.className = "list";

  if (!evidence || !evidence.artifact) {
    output.className = "list empty-state";
    output.textContent = "No review loaded.";
    return;
  }

  const artifact = evidence.artifact;
  output.append(
    textNode("strong", artifact.title || ("Chapter " + (artifact.chapter_index + 1))),
    textNode(
      "div",
      evidence.stale
        ? "This review is stale. Re-run literary review before accepting any proposal."
        : "Evidence is current. Suggested revisions still require an explicit human action.",
      evidence.stale ? "warning" : "meta"
    )
  );

  const findings = artifact.report?.findings || [];
  if (!findings.length) {
    output.append(textNode("div", "No findings in this review.", "empty-state"));
    return;
  }

  findings.forEach((finding) => {
    const row = document.createElement("div");
    row.className = "list-row";
    row.append(
      textNode("strong", finding.dimension + " · " + finding.severity),
      textNode("div", finding.summary, "meta")
    );

    const suggested = finding.revision_proposal?.suggested_text;
    const targetIndices = finding.target_indices || [];
    if (!evidence.stale && suggested && targetIndices.length === 1) {
      row.append(textNode("div", "Suggested replacement: " + suggested, "meta"));
      const accept = textNode("button", "Accept suggested revision");
      accept.addEventListener("click", async () => {
        const chapterIndex = Number(artifact.chapter_index);
        await call("accept_literary_review_revision", {
          projectRoot: state.projectRoot,
          chapterIndex,
          findingId: finding.id,
          reviewer: $("reviewer-name").value.trim() || "desktop-user",
        });
        showNotice("Suggested paragraph revision accepted. The old review is now stale; re-run review to verify it.");
        const refreshed = await call("get_literary_review", {
          projectRoot: state.projectRoot,
          chapterIndex,
        });
        renderLiteraryEvidence(refreshed);
      });
      row.append(accept);
    }
    output.append(row);
  });
}

function renderHistory(items) {
  const list = $("history-list");
  list.replaceChildren();
  if (!items.length) {
    list.className = "list empty-state";
    list.textContent = "No history entries.";
    return;
  }
  list.className = "list";
  [...items].reverse().forEach((item) => {
    const row = document.createElement("div");
    row.className = "list-row";
    row.append(textNode("strong", item.event));
    row.append(textNode("div", item.detail, "meta"));
    row.append(textNode("div", item.timestamp, "meta"));
    list.append(row);
  });
}

function commandDefinitions() {
  const navigation = Object.entries(VIEW_META).map(([view, meta]) => ({
    id: "nav-" + view,
    title: meta.title,
    subtitle: "Go to " + meta.eyebrow.toLowerCase(),
    icon: meta.icon,
    shortcut: meta.shortcut,
    keywords: view + " " + meta.title + " " + meta.eyebrow,
    run: () => setView(view),
  }));

  const actions = [
    {
      id: "open-project",
      title: "Open project folder",
      subtitle: "Choose an existing translation workspace",
      icon: "⌂",
      keywords: "open existing project folder",
      run: () => $("pick-open-root").click(),
    },
    {
      id: "create-project",
      title: "Create new project",
      subtitle: "Choose a folder and create a workspace",
      icon: "+",
      keywords: "new create project",
      run: () => $("create-project").click(),
    },
    {
      id: "import-source",
      title: "Choose source book",
      subtitle: "Import TXT, Markdown, DOCX, EPUB or text PDF",
      icon: "↥",
      keywords: "import source manuscript book",
      available: () => Boolean(state.projectRoot) && !$("pick-import-source").disabled,
      run: () => $("pick-import-source").click(),
    },
    {
      id: "run-analysis",
      title: "Run deterministic analysis",
      subtitle: "Extract manuscript intelligence without provider calls",
      icon: "◇",
      keywords: "analysis intelligence offline deterministic",
      available: () => Boolean(state.projectRoot) && !$("run-analysis").disabled,
      run: () => $("run-analysis").click(),
    },
    {
      id: "start-translation",
      title: "Start translation",
      subtitle: "Use the current translation configuration",
      icon: "→",
      keywords: "translate start run",
      available: () => Boolean(state.projectRoot) && !$("start-translation").disabled,
      run: () => $("start-translation").click(),
    },
    {
      id: "literary-review",
      title: "Run literary review",
      subtitle: "Generate bounded fidelity and naturalness evidence",
      icon: "✦",
      keywords: "review literary fidelity naturalness",
      available: () => Boolean(state.projectRoot) && !$("run-literary-review").disabled,
      run: () => { setView("literary"); $("run-literary-review").click(); },
    },
    {
      id: "export",
      title: "Export publication",
      subtitle: "Use the currently selected DOCX or EPUB format",
      icon: "↗",
      keywords: "export publish epub docx",
      available: () => Boolean(state.projectRoot) && !$("export-project").disabled,
      run: () => $("export-project").click(),
    },
    {
      id: "focus-persian",
      title: state.editorFocus ? "Show source + Persian" : "Focus Persian editor",
      subtitle: "Toggle the source column in the translation editor",
      icon: "◫",
      keywords: "editor focus persian source split",
      run: () => toggleEditorFocus(),
    },
  ];

  const themes = Object.entries(THEME_META).map(([theme, meta]) => ({
    id: "theme-" + theme,
    title: "Theme: " + meta.label,
    subtitle: meta.subtitle,
    icon: meta.icon,
    keywords: "theme appearance color " + theme + " " + meta.label,
    run: () => setTheme(theme),
  }));

  return [...navigation, ...actions, ...themes];
}

let visibleCommands = [];

function renderCommandResults(query = "") {
  const normalized = query.trim().toLowerCase();
  visibleCommands = commandDefinitions().filter((command) => {
    const haystack = (command.title + " " + command.subtitle + " " + (command.keywords || "")).toLowerCase();
    return !normalized || haystack.includes(normalized);
  });
  state.commandIndex = Math.max(0, Math.min(state.commandIndex, Math.max(0, visibleCommands.length - 1)));

  const results = $("command-results");
  results.replaceChildren();
  if (!visibleCommands.length) {
    results.append(textNode("div", "No matching command.", "palette-empty"));
    return;
  }

  visibleCommands.forEach((command, index) => {
    const available = !command.available || command.available();
    const button = document.createElement("button");
    button.type = "button";
    button.className = "command-item" + (index === state.commandIndex ? " active" : "");
    button.setAttribute("role", "option");
    button.setAttribute("aria-selected", String(index === state.commandIndex));
    if (!available) button.disabled = true;

    const icon = textNode("span", command.icon || "•", "command-icon");
    const copy = document.createElement("span");
    copy.className = "command-copy";
    copy.append(
      textNode("strong", command.title),
      textNode("small", available ? command.subtitle : command.subtitle + " · unavailable until the project state allows it")
    );
    button.append(icon, copy, textNode("span", command.shortcut || "", "command-shortcut"));
    button.addEventListener("mouseenter", () => {
      state.commandIndex = index;
      renderCommandResults($("command-search").value);
    });
    button.addEventListener("click", () => runPaletteCommand(command));
    results.append(button);
  });
}

function openCommandPalette() {
  const dialog = $("command-palette");
  state.commandIndex = 0;
  $("command-search").value = "";
  renderCommandResults("");
  if (!dialog.open) dialog.showModal();
  window.setTimeout(() => $("command-search").focus(), 0);
}

function closeCommandPalette() {
  const dialog = $("command-palette");
  if (dialog.open) dialog.close();
}

function runPaletteCommand(command) {
  if (command.available && !command.available()) {
    showNotice("That action is not available in the current project state.", "error");
    return;
  }
  closeCommandPalette();
  command.run();
}

function toggleEditorFocus() {
  state.editorFocus = !state.editorFocus;
  const editor = $("paragraph-editor");
  editor.classList.toggle("focus-persian", state.editorFocus);
  const button = $("editor-focus-toggle");
  button.setAttribute("aria-pressed", String(state.editorFocus));
  button.textContent = state.editorFocus ? "Show source + Persian" : "Focus Persian";
  if (state.currentView !== "editor") setView("editor");
}

document.querySelectorAll(".nav-item").forEach((button) => {
  button.addEventListener("click", () => setView(button.dataset.view));
});

document.querySelectorAll(".theme-swatch").forEach((button) => {
  button.addEventListener("click", () => setTheme(button.dataset.themeValue));
});

$("command-trigger").addEventListener("click", openCommandPalette);
$("top-command-trigger").addEventListener("click", openCommandPalette);
$("editor-focus-toggle").addEventListener("click", toggleEditorFocus);

$("command-search").addEventListener("input", (event) => {
  state.commandIndex = 0;
  renderCommandResults(event.target.value);
});

$("command-palette").addEventListener("click", (event) => {
  if (event.target === $("command-palette")) closeCommandPalette();
});

document.addEventListener("keydown", (event) => {
  const modifier = event.metaKey || event.ctrlKey;
  if (modifier && event.key.toLowerCase() === "k") {
    event.preventDefault();
    openCommandPalette();
    return;
  }

  if (modifier && /^[1-8]$/.test(event.key)) {
    const views = ["home", "workflow", "editor", "review", "canon", "literary", "history", "provider"];
    event.preventDefault();
    setView(views[Number(event.key) - 1]);
    return;
  }

  if (!$("command-palette").open) return;
  if (event.key === "ArrowDown" || event.key === "ArrowUp") {
    event.preventDefault();
    if (!visibleCommands.length) return;
    const delta = event.key === "ArrowDown" ? 1 : -1;
    state.commandIndex = (state.commandIndex + delta + visibleCommands.length) % visibleCommands.length;
    renderCommandResults($("command-search").value);
    const active = $("command-results").querySelector(".command-item.active");
    if (active) active.scrollIntoView({ block: "nearest" });
  } else if (event.key === "Enter") {
    event.preventDefault();
    const command = visibleCommands[state.commandIndex];
    if (command) runPaletteCommand(command);
  }
});

$("pick-open-root").addEventListener("click", async () => {
  const root = await call("pick_project_folder");
  if (!root) return;
  const snapshot = await call("open_project", { projectRoot: root });
  state.projectRoot = root;
  state.sourcePath = null;
  $("source-path").textContent = "No file selected";
  setProjectEnabled(true);
  renderSnapshot(snapshot);
  showNotice("Project opened.");
});

$("create-project").addEventListener("click", async () => {
  const root = await call("pick_project_folder");
  if (!root) return;
  const name = $("new-project-name").value.trim() || "Untitled Translation";
  const snapshot = await call("create_project", {
    projectRoot: root,
    name,
    sourcePath: null,
  });
  state.projectRoot = root;
  setProjectEnabled(true);
  renderSnapshot(snapshot);
  showNotice("Project created.");
});

$("refresh-project").addEventListener("click", refreshSnapshot);

$("pick-import-source").addEventListener("click", async () => {
  const source = await call("pick_source_file");
  if (!source) return;
  state.sourcePath = source;
  $("source-path").textContent = source;
  $("import-source").disabled = false;
});

$("import-source").addEventListener("click", async () => {
  if (!state.projectRoot || !state.sourcePath) return;
  const snapshot = await call("import_book", {
    projectRoot: state.projectRoot,
    sourcePath: state.sourcePath,
  });
  renderSnapshot(snapshot);
  showNotice("Source imported.");
});

$("verify-source").addEventListener("click", async () => {
  const status = await call("verify_source", { projectRoot: state.projectRoot });
  showNotice("Source verification: " + status, status === "fresh" ? "success" : "error");
});

$("run-analysis").addEventListener("click", async () => {
  renderSnapshot(await call("analyze_project", { projectRoot: state.projectRoot }));
  showNotice("Deterministic analysis completed.");
});

$("run-advanced").addEventListener("click", async () => {
  const snapshot = await call("run_advanced_analysis", {
    projectRoot: state.projectRoot,
    input: {
      provider: $("advanced-provider").value,
      model: $("advanced-model").value.trim() || null,
      maxUnits: numberOrNull("advanced-max"),
      cache: true,
    },
  });
  renderSnapshot(snapshot);
  showNotice("Advanced analysis completed.");
});

$("start-translation").addEventListener("click", () => runTranslation("start_translation"));
$("resume-translation").addEventListener("click", () => runTranslation("resume_translation"));
$("pause-translation").addEventListener("click", async () => {
  await call("request_pause", { projectRoot: state.projectRoot });
  showNotice("Pause requested. The runtime will stop at a safe checkpoint.");
});

$("export-project").addEventListener("click", async () => {
  const record = await call("export_project", {
    projectRoot: state.projectRoot,
    format: $("export-format").value,
  });
  $("export-result").textContent = record.relative_path;
  await refreshSnapshot();
  showNotice(record.format.toUpperCase() + " export completed.");
});

$("refresh-review").addEventListener("click", loadReviewItems);
$("review-status-filter").addEventListener("change", loadReviewItems);

$("refresh-characters").addEventListener("click", loadCharacters);
$("save-character").addEventListener("click", async () => {
  const name = $("character-name").value.trim();
  if (!name) {
    showNotice("Character name is required.", "error");
    return;
  }
  await call("upsert_character", {
    projectRoot: state.projectRoot,
    profile: {
      name,
      voice_notes: $("character-voice").value,
      personality_notes: $("character-personality").value,
    },
    replaceExisting: true,
  });
  await loadCharacters();
  await refreshSnapshot();
  showNotice("Character canon saved.");
});

$("refresh-glossary").addEventListener("click", loadGlossary);
$("save-glossary").addEventListener("click", async () => {
  const source = $("glossary-source").value.trim();
  const target = $("glossary-target").value.trim();
  if (!source || !target) {
    showNotice("Source term and preferred Persian translation are required.", "error");
    return;
  }
  await call("upsert_glossary_entry", {
    projectRoot: state.projectRoot,
    entry: {
      source_term: source,
      preferred_translation: target,
      context: $("glossary-context").value,
    },
    replaceExisting: true,
  });
  await loadGlossary();
  await refreshSnapshot();
  showNotice("Glossary canon saved.");
});

$("load-chapter").addEventListener("click", async () => {
  const chapterIndex = Math.max(0, Number($("editor-chapter-index").value || 1) - 1);
  renderChapter(await call("get_translated_chapter", {
    projectRoot: state.projectRoot,
    chapterIndex,
  }));
});

$("run-literary-review").addEventListener("click", async () => {
  const summary = await call("run_literary_review", {
    projectRoot: state.projectRoot,
    input: {
      provider: $("literary-provider").value,
      model: $("literary-model").value.trim() || null,
      semanticAlignment: $("semantic-alignment").checked,
      maxChapters: numberOrNull("literary-max"),
    },
  });
  $("literary-output").className = "list";
  $("literary-output").replaceChildren(
    textNode("strong", "Literary review completed"),
    textNode("div", summary.reviewed_chapters + " chapter(s) reviewed · " + summary.findings + " finding(s)", "meta")
  );
  showNotice("Literary review completed.");
});

$("load-literary-review").addEventListener("click", async () => {
  const chapterIndex = Math.max(0, Number($("literary-chapter-index").value || 1) - 1);
  const evidence = await call("get_literary_review", {
    projectRoot: state.projectRoot,
    chapterIndex,
  });
  renderLiteraryEvidence(evidence);
});

$("refresh-history").addEventListener("click", async () => {
  renderHistory(await call("project_history", { projectRoot: state.projectRoot }));
});

$("set-key").addEventListener("click", async () => {
  const key = $("openai-key").value;
  await call("set_session_openai_key", { apiKey: key });
  $("openai-key").value = "";
  showNotice("OpenAI key is active for this app session only.");
});

$("clear-key").addEventListener("click", async () => {
  await call("clear_session_openai_key");
  $("openai-key").value = "";
  showNotice("Session key cleared.");
});

$("check-provider").addEventListener("click", async () => {
  await call("test_provider_configuration", {
    translationProvider: $("provider-check-translation").value,
    analysisProvider: $("provider-check-analysis").value,
  });
  showNotice("Local provider configuration is available.");
});

setTheme("system");
setProjectEnabled(false);
renderSnapshot(null);
setView("home");
