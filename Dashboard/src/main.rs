slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let ui = LoginWindow::new()?;

    let ui_handle = ui.as_weak();
    ui.on_handle_login(move || {
        let ui = ui_handle.unwrap();
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
            println!("Login successful!");
        } else {
            ui.set_error_message("无效的账号或者密码".into());
            println!("Login failed!");
        }
    });

    ui.on_handle_exit(move || {
        std::process::exit(0);
    });

    ui.run()
}
