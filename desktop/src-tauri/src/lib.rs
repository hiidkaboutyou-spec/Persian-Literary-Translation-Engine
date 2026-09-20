mod commands;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::app_capabilities,
            commands::pick_project_folder,
            commands::pick_source_file,
            commands::set_session_openai_key,
            commands::clear_session_openai_key,
            commands::test_provider_configuration,
            commands::create_project,
            commands::open_project,
            commands::import_book,
            commands::verify_source,
            commands::analyze_project,
            commands::run_advanced_analysis,
            commands::list_review_items,
            commands::decide_review_item,
            commands::list_characters,
            commands::upsert_character,
            commands::list_glossary_entries,
            commands::upsert_glossary_entry,
            commands::start_translation,
            commands::resume_translation,
            commands::request_pause,
            commands::get_progress,
            commands::get_translated_chapter,
            commands::apply_manual_edit,
            commands::get_pilot_audit,
            commands::get_pilot_review_summary,
            commands::record_pilot_review,
            commands::run_literary_review,
            commands::get_literary_review,
            commands::accept_literary_review_revision,
            commands::export_project,
            commands::project_history,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Persian Literary Translation Engine desktop app");
}
