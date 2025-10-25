use std::sync::Arc;

use slint::{ComponentHandle, SharedString, ToSharedString};

use crate::{
    AccountInteraction, AppWindow, LoginWindow, RecipeInteraction,
    http_client::HttpClient,
    model::account::AccountLogin,
    ui::{account::register_account_interaction, recipe::register_recipe_interaction},
};
pub struct App {
    #[allow(dead_code)]
    app_window: AppWindow,
    login_window: LoginWindow,
    #[allow(dead_code)]
    http_client: Arc<HttpClient>,
}

impl App {
    pub fn new() -> Self {
        let login_window = LoginWindow::new().unwrap();
        let app_window = AppWindow::new().unwrap();

        let login_handle = login_window.as_weak();
        let app_handle = app_window.as_weak();
        let http_client = Arc::new(HttpClient::new(move || {
            let login_handle = login_handle.clone();
            let app_handle = app_handle.clone();
            slint::invoke_from_event_loop(move || {
                let login_handle = login_handle.clone();
                let app_handle = app_handle.clone();
                login_handle.unwrap().show().unwrap();
                app_handle.unwrap().hide().unwrap();
            })
            .unwrap();
        }));

        register_recipe_interaction(&app_window.global::<RecipeInteraction>());
        register_account_interaction(&app_window, &login_window, http_client.clone());

        let login_handle = login_window.as_weak();
        let app_handle = app_window.as_weak();
        let http_client_handle = http_client.clone();
        login_window.on_handle_login(move || {
            let login_ui = login_handle.unwrap();
            let app_ui = app_handle.unwrap();

            let username = login_ui.get_username_input();
            let password = login_ui.get_password_input();

            if username.is_empty() {
                login_ui.set_username_valid(false);
                return;
            } else {
                login_ui.set_username_valid(true);
            }

            if password.is_empty() {
                login_ui.set_password_valid(false);
                return;
            } else {
                login_ui.set_password_valid(true);
            }

            // 认证API
            let account_login = AccountLogin {
                username: username.to_string(),
                password: password.to_string(),
            };

            let http_client = http_client_handle.clone();
            slint::spawn_local(async move {
                match http_client.login(account_login).await {
                    Ok(account) => {
                        let account_interaction = app_ui.global::<AccountInteraction>();
                        account_interaction.set_username(account.username.to_shared_string());
                        account_interaction.set_role(account.role.to_shared_string());
                        login_ui.set_password_input(SharedString::new());
                        login_ui.set_username_input(SharedString::new());
                        app_ui.show().unwrap();
                        login_ui.hide().unwrap();
                    }
                    Err(_e) => {
                        login_ui.set_error_message("无效的账号或者密码".into());
                    }
                }
            })
            .unwrap();
        });

        login_window.on_handle_exit(move || {
            std::process::exit(0);
        });

        Self {
            app_window,
            login_window,
            http_client,
        }
    }

    pub fn run(&self) {
        tokio::task::block_in_place(|| self.login_window.run()).unwrap();
    }
}
