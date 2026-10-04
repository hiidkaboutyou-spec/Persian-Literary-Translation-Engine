# Native picker deadlock

Reproduced on the Apple Silicon application built at fe1d792 (Trusted Release
run 37038644448). Opening the app worked; clicking Choose folder & create froze
the UI. A macOS process sample showed the main dispatch queue inside
commands::pick_project_folder -> FileDialogBuilder::blocking_pick_folder ->
std::sync::mpmc::Receiver::recv -> thread::park. No project had been created.

Both folder and source pickers were synchronous Tauri commands. Their blocking
native dialogs require the main event loop which the command itself blocked.
Tauri documents that blocking pickers must not run on the main thread and shows
async commands for this use:
https://docs.rs/tauri-plugin-dialog/2.7.2/tauri_plugin_dialog/struct.FileDialogBuilder.html

Make both commands async so Tauri dispatches them off the main event loop.
Command names, arguments, return types, cancellation and local-path validation
stay unchanged. No new dependency. A compile-time Future/Send contract protects
the dispatch boundary without opening native dialogs in headless tests.

Acceptance: existing desktop Rust checks/build, then real macOS folder/file
selection, cancellation and project creation/import. Compile checks alone do
not prove interactive acceptance. Local dependency fetching was unavailable;
GitHub CI must provide the build. Rollback: revert this isolated change.
