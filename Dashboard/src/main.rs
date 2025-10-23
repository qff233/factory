slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let login_window = LoginWindow::new()?;
    let app_window = AppWindow::new()?;

    let login_handle = login_window.as_weak();
    let app_handle = app_window.as_weak();
    login_window.on_handle_login(move || {
        let ui = login_handle.unwrap();
        let username = ui.get_username_input();
        let password = ui.get_password_input();

        if username.is_empty() {
            ui.set_username_valid(false);
            return;
        } else {
            ui.set_username_valid(true);
        }

        if password.is_empty() {
            ui.set_password_valid(false);
            return;
        } else {
            ui.set_password_valid(true);
        }

        // 认证API
        if username == "admin" && password == "password" {
            let login_window = login_handle.unwrap();
            let app_window = app_handle.unwrap();
            app_window.show().unwrap();
            login_window.hide().unwrap();
        } else {
            ui.set_error_message("无效的账号或者密码".into());
            println!("Login failed!");
        }
    });

    login_window.on_handle_exit(move || {
        std::process::exit(0);
    });

    login_window.run()
}
