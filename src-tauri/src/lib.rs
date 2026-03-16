pub mod arm;
pub mod auth;
pub mod commands;
pub mod foundry;
pub mod local;
pub mod m365;
pub mod studio;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      // Auth
      commands::sign_in,
      commands::sign_out,
      commands::get_auth_status,
      commands::get_auth_config,
      // Discovery (ARM)
      commands::discover_resources,
      commands::list_subscriptions,
      commands::list_deployments,
      // Foundry
      commands::list_foundry_deployments,
      commands::list_agents,
      commands::create_agent,
      commands::delete_agent,
      commands::list_files,
      commands::delete_file,
      commands::list_vector_stores,
      commands::list_fine_tuning_jobs,
      commands::list_batch_jobs,
      commands::list_connections,
      commands::list_models,
      // Chat
      commands::send_chat_message,
      commands::send_agent_message,
      // Multi-source agents
      commands::list_studio_agents,
      commands::list_m365_agents,
      commands::list_local_agents,
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
