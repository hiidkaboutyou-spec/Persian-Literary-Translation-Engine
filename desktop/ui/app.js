"use strict";

const invoke = window.__TAURI__.core.invoke;

const state = {
  projectRoot: null,
  sourcePath: null,
  snapshot: null,
  translationRunning: false,
  progressTimer: null,
  literaryEvidence: null,
  activeCommandIndex: 0,
  paletteReturnFocus: null,
};

const $ = (id) => document.getElementById(id);

const viewCommands = [
  { view: "home", label: "Project", detail: "Open, create, import and inspect the book.", icon: "⌂", shortcut: "⌘1" },
  { view: "workflow", label: "Workflow", detail: "Analyze, translate, pause, resume and export.", icon: "↝", shortcut: "⌘2" },
  { view: "review", label: "Intelligence", detail: "Make human decisions over extracted literary evidence.", icon: "◇", shortcut: "⌘3" },
  { view: "canon", label: "Canon", detail: "Edit durable character voice and terminology.", icon: "✦", shortcut: "⌘4" },
  { view: "editor", label: "Translation Editor", detail: "Revise English and Persian paragraph pairs.", icon: "¶", shortcut: "⌘5" },
  { view: "literary", label: "Literary Review", detail: "Inspect fidelity, naturalness and revision evidence.", icon: "≈", shortcut: "⌘6" },
  { view: "history", label: "History", detail: "Read the durable project audit trail.", icon: "↺", shortcut: "⌘7" },
  { view: "provider", label: "Provider", detail: "Configure session-only model access.", icon: "⌁", shortcut: "⌘8" },
];

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
  document.querySelectorAll(".nav-item").forEach((button) => {
    button.classList.toggle("active", button.dataset.view === name);
  });
  document.querySelectorAll("[data-view-panel]").forEach((panel) => {
    panel.classList.toggle("active", panel.dataset.viewPanel === name);
  });
  const titles = {
    home: ["Project", "Open the book workspace or begin a new long-form translation."],
    workflow: ["Workflow", "Move deliberately from literary analysis to translation and publication."],
    review: ["Intelligence", "Review extracted literary evidence before anything becomes canon."],
    canon: ["Canon", "Shape the durable voice, character and terminology context for the whole book."],
    editor: ["Translation Editor", "Work paragraph by paragraph with English and Persian visible together."],
    literary: ["Literary Review", "Inspect fidelity and Persian-naturalness evidence before accepting revisions."],
    history: ["History", "Follow the durable audit trail of decisions, revisions and publication actions."],
    provider: ["Provider", "Use model access for this app session without storing secrets in the project."],
  };
  const [title, subtitle] = titles[name] || titles.home;
  $("view-title").textContent = title;
  $("view-subtitle").textContent = subtitle;
  document.documentElement.dataset.view = name;
  closeCommandPalette(false);
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

function setStatusPill(value) {
  const pill = $("status-pill");
  const label = pill.querySelector(".status-text");
  if (label) {
    label.textContent = value || "No project";
  } else {
    pill.textContent = value || "No project";
  }
  pill.className = value ? "status-pill" : "status-pill muted";
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
  setStatusPill(snapshot ? snapshot.status : null);

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
  editor.className = "paragraph-editor";

  chapter.paragraphs.forEach((paragraph) => {
    const block = document.createElement("div");
    block.className = "paragraph";
    block.append(textNode("div", paragraph.source, "source-text"));
    const textarea = document.createElement("textarea");
    textarea.className = "translation-text";
    textarea.dir = "rtl";
    textarea.value = paragraph.translated;
    const save = textNode("button", "Save revision");
    save.addEventListener("click", async () => {
      if (textarea.value === paragraph.translated) {
        showNotice("No text change to save.", "error");
        return;
      }
      const chapterIndex = Number($("editor-chapter-index").value) - 1;
      await call("apply_manual_edit", {
        projectRoot: state.projectRoot,
        chapterIndex,
        paragraphId: paragraph.paragraph_id,
        newText: textarea.value,
        reviewer: $("reviewer-name").value.trim() || "desktop-user",
      });
      paragraph.translated = textarea.value;
      showNotice("Manual revision saved; quality evidence is now stale until reviewed again.");
    });
    block.append(textarea, save);
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
  list.className = "list timeline";
  [...items].reverse().forEach((item) => {
    const row = document.createElement("div");
    row.className = "list-row";
    row.append(textNode("strong", item.event));
    row.append(textNode("div", item.detail, "meta"));
    row.append(textNode("div", item.timestamp, "meta"));
    list.append(row);
  });
}

function matchingCommands(query) {
  const wanted = query.trim().toLocaleLowerCase();
  if (!wanted) return viewCommands;
  return viewCommands.filter((command) => {
    return (command.label + " " + command.detail).toLocaleLowerCase().includes(wanted);
  });
}

function renderCommandResults() {
  const results = $("command-results");
  const commands = matchingCommands($("command-search").value);
  state.activeCommandIndex = Math.max(0, Math.min(state.activeCommandIndex, Math.max(0, commands.length - 1)));
  results.replaceChildren();

  if (!commands.length) {
    results.append(textNode("div", "No matching workspace view.", "empty-state"));
    return;
  }

  commands.forEach((command, index) => {
    const button = document.createElement("button");
    button.type = "button";
    button.className = "command-result" + (index === state.activeCommandIndex ? " selected" : "");
    button.setAttribute("role", "option");
    button.setAttribute("aria-selected", index === state.activeCommandIndex ? "true" : "false");

    const icon = textNode("span", command.icon, "command-result-icon");
    const copy = document.createElement("span");
    copy.append(textNode("strong", command.label), textNode("small", command.detail));
    const shortcut = textNode("kbd", command.shortcut);
    button.append(icon, copy, shortcut);

    button.addEventListener("mouseenter", () => {
      state.activeCommandIndex = index;
      renderCommandResults();
    });
    button.addEventListener("click", () => setView(command.view));
    results.append(button);
  });
}

function openCommandPalette() {
  const palette = $("command-palette");
  if (palette.classList.contains("open")) return;
  state.paletteReturnFocus = document.activeElement;
  state.activeCommandIndex = 0;
  $("command-search").value = "";
  renderCommandResults();
  palette.classList.add("open");
  palette.setAttribute("aria-hidden", "false");
  window.requestAnimationFrame(() => $("command-search").focus());
}

function closeCommandPalette(restoreFocus = true) {
  const palette = $("command-palette");
  if (!palette.classList.contains("open")) return;
  palette.classList.remove("open");
  palette.setAttribute("aria-hidden", "true");
  if (restoreFocus && state.paletteReturnFocus && typeof state.paletteReturnFocus.focus === "function") {
    state.paletteReturnFocus.focus();
  }
  state.paletteReturnFocus = null;
}

function moveCommandSelection(delta) {
  const commands = matchingCommands($("command-search").value);
  if (!commands.length) return;
  state.activeCommandIndex = (state.activeCommandIndex + delta + commands.length) % commands.length;
  renderCommandResults();
  const selected = $("command-results").querySelector(".command-result.selected");
  if (selected) selected.scrollIntoView({ block: "nearest" });
}

document.querySelectorAll(".nav-item").forEach((button) => {
  button.addEventListener("click", () => setView(button.dataset.view));
});

$("open-palette").addEventListener("click", openCommandPalette);
document.querySelector("[data-close-palette]").addEventListener("click", () => closeCommandPalette());
$("command-search").addEventListener("input", () => {
  state.activeCommandIndex = 0;
  renderCommandResults();
});

document.addEventListener("keydown", (event) => {
  const commandKey = event.metaKey || event.ctrlKey;

  if (commandKey && event.key.toLocaleLowerCase() === "k") {
    event.preventDefault();
    if ($("command-palette").classList.contains("open")) closeCommandPalette();
    else openCommandPalette();
    return;
  }

  if ($("command-palette").classList.contains("open")) {
    if (event.key === "Escape") {
      event.preventDefault();
      closeCommandPalette();
    } else if (event.key === "ArrowDown") {
      event.preventDefault();
      moveCommandSelection(1);
    } else if (event.key === "ArrowUp") {
      event.preventDefault();
      moveCommandSelection(-1);
    } else if (event.key === "Enter") {
      event.preventDefault();
      const selected = $("command-results").querySelector(".command-result.selected");
      if (selected) selected.click();
    }
    return;
  }

  if (commandKey && /^[1-8]$/.test(event.key)) {
    event.preventDefault();
    const command = viewCommands[Number(event.key) - 1];
    if (command) setView(command.view);
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

setProjectEnabled(false);
renderSnapshot(null);
renderCommandResults();
setView("home");
