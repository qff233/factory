use std::sync::Arc;

use slint::{ComponentHandle, ToSharedString, format};

use crate::{AccountInteraction, AppWindow, LoginWindow, http_client::HttpClient};

pub fn register_account_interaction(
    app_window: &AppWindow,
    login_window: &LoginWindow,
    http_client: Arc<HttpClient>,
) {
    let app_handle = app_window.as_weak();
    let app_ui = app_handle.unwrap();
    let inner = app_ui.global::<AccountInteraction>();
    inner.on_logout(move || {
        let http_client = http_client.clone();
        let app_handle = app_handle.clone();

        slint::spawn_local(async move {
            let app_ui = app_handle.unwrap();
            let inner = app_ui.global::<AccountInteraction>();
            match http_client.logout().await {
                Ok(message) => {
                    inner.set_dialog_text(message.to_shared_string());
                }
                Err(e) => {
                    inner.set_dialog_text(format!("{:?}", e));
                }
            }
        })
        .unwrap();
    });

    let app_handle = app_window.as_weak();
    let login_handle = login_window.as_weak();
    inner.on_check_logout(move || {
        let app_ui = app_handle.unwrap();
        let login_ui = login_handle.unwrap();

        login_ui.show().unwrap();
        app_ui.hide().unwrap();
    });
}
