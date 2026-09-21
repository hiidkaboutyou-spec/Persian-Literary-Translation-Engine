"use strict";

const invoke = window.__TAURI__.core.invoke;

const state = {
  projectRoot: null,
  sourcePath: null,
  snapshot: null,
  translationRunning: false,
  progressTimer: null,
  literaryEvidence: null,
  pilotSummary: null,
  pilotTarget: null,
  pilotChapter: null,
};

const $ = (id) => document.getElementById(id);

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
    home: ["Project", "Open or create a translation project."],
    workflow: ["Workflow", "Explicit analysis, translation and publication stages."],
    review: ["Intelligence Review", "Human decisions over extracted literary intelligence."],
    canon: ["Canon", "Character voice and terminology that guide long-form continuity."],
    editor: ["Translation Editor", "Paragraph-level Persian revision with durable history."],
    literary: ["Literary Review", "Post-translation fidelity and naturalness evidence."],
    pilot: ["Pilot Review", "Bounded whole-book sampling with explicit local human decisions."],
    history: ["History", "Bounded audit history emitted by the application layer."],
    provider: ["Provider", "Session-only provider configuration; secrets are never stored in projects."],
  };
  const [title, subtitle] = titles[name] || titles.home;
  $("view-title").textContent = title;
  $("view-subtitle").textContent = subtitle;
}

function setProjectEnabled(enabled) {
  [
    "refresh-project", "verify-source", "pick-import-source", "run-analysis", "run-advanced",
    "start-translation", "resume-translation", "pause-translation", "export-project",
    "refresh-review", "refresh-characters", "save-character", "refresh-glossary",
    "save-glossary", "load-chapter", "run-literary-review", "load-literary-review",
    "refresh-pilot-review", "refresh-history"
  ].forEach((id) => {
    $(id).disabled = !enabled;
  });
  $("import-source").disabled = !enabled || !state.sourcePath;
  $("record-pilot-review").disabled = !enabled || !state.pilotTarget;
  $("open-pilot-editor").disabled = !enabled || !state.pilotTarget;
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

function nonNegativeNumberOrNull(id) {
  const value = $(id).value.trim();
  if (!value) return null;
  const number = Number(value);
  return Number.isFinite(number) && number >= 0 ? Math.floor(number) : null;
}

function enumLabel(value) {
  return String(value || "")
    .split("_")
    .filter(Boolean)
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(" ");
}

function optionalSpan(startId, endId) {
  const startRaw = $(startId).value.trim();
  const endRaw = $(endId).value.trim();
  if (!startRaw || !endRaw) return null;
  return {
    start_char: Number(startRaw),
    end_char: Number(endRaw),
  };
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

function renderChapter(chapter, focusParagraphId = null) {
  $("chapter-meta").replaceChildren(
    textNode("p", chapter.title + " · " + chapter.paragraphs.length + " paragraphs · style " + chapter.style_profile)
  );
  const editor = $("paragraph-editor");
  editor.replaceChildren();
  editor.className = "paragraph-editor";

  chapter.paragraphs.forEach((paragraph) => {
    const block = document.createElement("div");
    block.className = paragraph.paragraph_id === focusParagraphId ? "paragraph pilot-focused" : "paragraph";
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
    if (paragraph.paragraph_id === focusParagraphId) {
      window.requestAnimationFrame(() => block.scrollIntoView({ block: "center" }));
    }
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

function pilotTargetStatus(targetState) {
  const record = targetState.current_record;
  if (record) return enumLabel(record.outcome);
  if (targetState.stale_record_count > 0) return "Needs re-review";
  return "Not reviewed";
}

function clearPilotSelection() {
  state.pilotTarget = null;
  state.pilotChapter = null;
  $("record-pilot-review").disabled = true;
  $("open-pilot-editor").disabled = true;
  const context = $("pilot-context");
  context.className = "pilot-context empty-state";
  context.textContent = "Select a review target.";
}

function renderPilotContext(targetState, chapter) {
  const context = $("pilot-context");
  context.replaceChildren();
  context.className = "pilot-context";

  const target = targetState.target;
  const paragraphs = chapter.paragraphs || [];
  const targetIndex = target.paragraph_id
    ? paragraphs.findIndex((paragraph) => paragraph.paragraph_id === target.paragraph_id)
    : (paragraphs.length ? 0 : -1);

  context.append(
    textNode("strong", (chapter.title || ("Chapter " + (target.chapter_index + 1))) + " · " + pilotTargetStatus(targetState)),
    textNode("div", "Reasons: " + (target.reasons || []).map(enumLabel).join(", "), "meta"),
    textNode("div", "Target ID: " + targetState.target_id, "meta")
  );

  if (targetIndex < 0) {
    context.append(textNode("div", "The current target paragraph could not be resolved in the local translated chapter. Refresh the audit before recording a decision.", "warning danger-warning"));
    return false;
  }

  const start = Math.max(0, targetIndex - 1);
  const end = Math.min(paragraphs.length, targetIndex + 2);
  paragraphs.slice(start, end).forEach((paragraph, offset) => {
    const absoluteIndex = start + offset;
    const block = document.createElement("div");
    block.className = absoluteIndex === targetIndex
      ? "pilot-context-paragraph selected"
      : "pilot-context-paragraph";
    block.append(
      textNode(
        "small",
        absoluteIndex === targetIndex
          ? "Selected paragraph"
          : (absoluteIndex < targetIndex ? "Previous context" : "Next context")
      ),
      textNode("div", paragraph.source, "source-text"),
      textNode("div", paragraph.translated, "pilot-translation-text")
    );
    context.append(block);
  });

  if (!paragraphs.length) {
    context.append(textNode("div", "The translated chapter has no paragraphs to inspect.", "empty-state"));
  }

  const record = targetState.current_record;
  if (record) {
    context.append(
      textNode("div", "Current human record: " + enumLabel(record.outcome) + " · " + record.reviewer, "meta")
    );
    if (record.note) context.append(textNode("div", "Reviewer note: " + record.note, "meta"));
  }
  if (targetState.stale_record_count > 0) {
    context.append(textNode("div", targetState.stale_record_count + " stale prior record(s) remain in append-only history.", "warning"));
  }
  return true;
}

async function selectPilotTarget(targetState) {
  state.pilotTarget = targetState;
  const chapter = await call("get_translated_chapter", {
    projectRoot: state.projectRoot,
    chapterIndex: targetState.target.chapter_index,
  });
  state.pilotChapter = chapter;
  const inspectable = renderPilotContext(targetState, chapter);
  $("record-pilot-review").disabled = !inspectable;
  $("open-pilot-editor").disabled = !inspectable;
}

function renderPilotSummary(summary) {
  state.pilotSummary = summary;
  const audit = summary.audit;

  const grid = $("pilot-summary-grid");
  grid.replaceChildren();
  grid.className = "snapshot-grid";
  grid.append(
    snapshotItem("Mechanical export", audit.mechanically_export_ready ? "Ready" : "Blocked"),
    snapshotItem("Sample review", summary.sample_review_complete ? "Complete" : "Open"),
    snapshotItem("Targets", String(summary.targets.length)),
    snapshotItem("Reviewed", summary.reviewed_current + "/" + summary.targets.length),
    snapshotItem("Resolved", String(summary.resolved_current)),
    snapshotItem("Needs revision", String(summary.needs_revision_current)),
    snapshotItem("Missing", String(summary.missing_current)),
    snapshotItem("Stale records", String(summary.stale_records))
  );

  const issues = $("pilot-issues");
  issues.replaceChildren();
  (audit.issues || []).forEach((issue) => {
    const location = issue.chapter_index == null ? "Book" : "Chapter " + (issue.chapter_index + 1);
    issues.append(
      textNode(
        "div",
        location + " · " + enumLabel(issue.code) + (issue.blocking ? " · blocking" : " · review"),
        issue.blocking ? "warning danger-warning" : "warning"
      )
    );
  });

  const list = $("pilot-target-list");
  list.replaceChildren();
  if (!summary.targets.length) {
    list.className = "list empty-state";
    list.textContent = "No current review targets for this selection.";
    clearPilotSelection();
    return;
  }
  list.className = "list";

  summary.targets.forEach((targetState) => {
    const target = targetState.target;
    const row = document.createElement("div");
    row.className = "list-row";
    row.append(
      textNode("strong", "Chapter " + (target.chapter_index + 1) + " · " + pilotTargetStatus(targetState)),
      textNode("div", (target.reasons || []).map(enumLabel).join(" · "), "meta"),
      textNode("div", target.paragraph_id ? "Paragraph " + target.paragraph_id : "Chapter-level target", "meta")
    );
    if (targetState.stale_record_count > 0) {
      row.append(textNode("div", targetState.stale_record_count + " stale prior record(s)", "meta"));
    }
    const actions = document.createElement("div");
    actions.className = "actions";
    const inspect = textNode("button", "Inspect");
    inspect.addEventListener("click", () => selectPilotTarget(targetState));
    actions.append(inspect);
    row.append(actions);
    list.append(row);
  });
}

async function loadPilotReview(preserveTargetId = null) {
  if (!state.projectRoot) return;
  const maxReviewTargets = nonNegativeNumberOrNull("pilot-max-targets");
  const summary = await call("get_pilot_review_summary", {
    projectRoot: state.projectRoot,
    maxReviewTargets,
  });
  renderPilotSummary(summary);

  const targetId = preserveTargetId || state.pilotTarget?.target_id;
  if (targetId) {
    const refreshed = summary.targets.find((candidate) => candidate.target_id === targetId);
    if (refreshed) {
      await selectPilotTarget(refreshed);
      return;
    }
  }
  clearPilotSelection();
}

function buildPilotFindings(outcome) {
  const sourceSpan = optionalSpan("pilot-source-start", "pilot-source-end");
  const targetSpan = optionalSpan("pilot-target-start", "pilot-target-end");
  const note = $("pilot-finding-note").value.trim();
  const includeFinding = Boolean(sourceSpan || targetSpan || note || outcome === "needs_revision");
  if (!includeFinding) return [];
  return [{
    dimension: $("pilot-finding-dimension").value,
    severity: $("pilot-finding-severity").value,
    source_span: sourceSpan,
    target_span: targetSpan,
    note,
  }];
}

function resetPilotFindingForm() {
  $("pilot-note").value = "";
  $("pilot-finding-note").value = "";
  ["pilot-source-start", "pilot-source-end", "pilot-target-start", "pilot-target-end"].forEach((id) => {
    $(id).value = "";
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

document.querySelectorAll(".nav-item").forEach((button) => {
  button.addEventListener("click", () => setView(button.dataset.view));
});

$("pick-open-root").addEventListener("click", async () => {
  const root = await call("pick_project_folder");
  if (!root) return;
  const snapshot = await call("open_project", { projectRoot: root });
  state.projectRoot = root;
  state.sourcePath = null;
  state.pilotSummary = null;
  state.pilotTarget = null;
  state.pilotChapter = null;
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
  state.pilotSummary = null;
  state.pilotTarget = null;
  state.pilotChapter = null;
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

$("refresh-pilot-review").addEventListener("click", async () => {
  await loadPilotReview();
  showNotice("Pilot audit and human review queue refreshed.");
});

$("record-pilot-review").addEventListener("click", async () => {
  if (!state.pilotTarget) return;
  const outcome = $("pilot-outcome").value;
  const targetId = state.pilotTarget.target_id;
  await call("record_pilot_review", {
    projectRoot: state.projectRoot,
    maxReviewTargets: nonNegativeNumberOrNull("pilot-max-targets"),
    targetId,
    submission: {
      reviewer: $("pilot-reviewer").value.trim(),
      outcome,
      findings: buildPilotFindings(outcome),
      note: $("pilot-note").value,
    },
  });
  resetPilotFindingForm();
  await loadPilotReview(targetId);
  showNotice("Human pilot review recorded in the local append-only ledger.");
});

$("open-pilot-editor").addEventListener("click", () => {
  if (!state.pilotTarget || !state.pilotChapter) return;
  $("editor-chapter-index").value = state.pilotTarget.target.chapter_index + 1;
  setView("editor");
  renderChapter(state.pilotChapter, state.pilotTarget.target.paragraph_id || null);
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
