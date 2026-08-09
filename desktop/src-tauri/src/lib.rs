mod commands;
mod files;
mod genius;
mod hotkeys;
mod lrclib;
mod mini;
mod oauth;
mod presence;
mod proxy;
mod state;
mod together;
mod tray;
mod yandex;

use presence::DiscordState;
use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .register_asynchronous_uri_scheme_protocol(proxy::SCHEME, |_app, request, responder| {
            proxy::handle(request, responder);
        })
        .manage(AppState::default())
        .manage(DiscordState::default())
        .manage(together::TogetherState::default())
        .setup(|app| {
            for (_, window) in app.webview_windows() {
                if let Ok(icon) =
                    tauri::image::Image::from_bytes(include_bytes!("../icons/256x256.png"))
                {
                    let _ = window.set_icon(icon);
                }
            }
            tray::build(app.handle())?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" && tray::close_to_tray() {
                    api.prevent_close();
                    tray::hide(window.app_handle());
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::prefetch_stream,
            commands::clear_stream_cache,
            commands::auth_login,
            commands::auth_me,
            commands::auth_logout,
            oauth::oauth_device_start,
            oauth::oauth_cancel,
            commands::get_wave,
            commands::get_wheel,
            commands::get_artist,
            commands::get_artist_tracks,
            commands::get_album,
            commands::wave_feedback,
            commands::search_tracks,
            commands::get_liked_tracks,
            commands::get_liked_ids,
            commands::get_disliked_ids,
            commands::set_like,
            commands::set_dislike,
            commands::get_stations,
            commands::station_info,
            commands::set_station_settings,
            commands::get_playlists,
            commands::get_playlist_tracks,
            commands::playlist_memberships,
            commands::playlist_add_track,
            commands::playlist_remove_track,
            commands::playlist_move_track,
            commands::playlist_create,
            commands::playlist_rename,
            commands::playlist_set_visibility,
            commands::playlist_delete,
            commands::playlist_clear,
            commands::playlist_recommendations,
            commands::search_suggest,
            commands::get_similar_tracks,
            commands::get_track,
            commands::get_track_info,
            commands::get_stream,
            commands::get_lyrics,
            commands::download_track,
            commands::download_image,
            commands::ai_check_artists,
            commands::ai_check_tracks,
            commands::default_download_dir,
            commands::find_local_track,
            commands::remove_local_track,
            commands::downloads_info,
            genius::genius_check,
            genius::genius_search,
            genius::genius_search_people,
            genius::genius_song,
            genius::genius_lookup,
            genius::genius_artist,
            genius::genius_clear_cache,
            lrclib::lrclib_lookup,
            lrclib::lrclib_clear_cache,
            presence::validate_discord_app,
            presence::update_discord_presence,
            presence::clear_discord_presence,
            presence::discord_presence_status,
            presence::reconnect_discord_presence,
            tray::set_close_to_tray,
            tray::set_tray_tooltip,
            tray::hide_to_tray,
            tray::show_from_tray,
            tray::quit_app,
            hotkeys::set_global_hotkeys,
            hotkeys::clear_global_hotkeys,
            mini::enter_mini_player,
            mini::resize_mini_player,
            mini::exit_mini_player,
            together::commands::together_host,
            together::commands::together_join,
            together::commands::together_leave,
            together::commands::together_send,
            together::commands::together_status,
            together::commands::together_log_path,
            files::export_text_file,
            files::read_text_file,
            files::open_external,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Mashiro");
}
