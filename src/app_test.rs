#[cfg(test)]
mod tests {
    use crate::app::{App, InputMode};
    use crate::models::{Package, PackageSource};

    fn create_test_package(name: &str, source: PackageSource, installed: bool) -> Package {
        Package {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            description: format!("Test package {}", name),
            source,
            is_installed: installed,
            is_outdated: false,
            installed_size: Some(1024),
            download_size: Some(512),
            groups: vec![],
            licenses: vec!["MIT".to_string()],
            maintainers: vec![],
            keywords: vec![],
            url: None,
            depends_on: vec![],
            required_by: vec![],
            opt_depends: vec![],
            conflicts: vec![],
            replaces: vec![],
            provides: vec![],
            votes: Some(10),
            popularity: Some(5.5),
            first_submitted: Some(1609459200),
            last_updated: Some(1640995200),
            package_base_id: None,
            num_votes: Some(10),
        }
    }

    #[test]
    fn test_package_creation() {
        let pkg = create_test_package("test-package", PackageSource::Pacman, false);

        assert_eq!(pkg.name, "test-package");
        assert_eq!(pkg.version, "1.0.0");
        assert_eq!(pkg.description, "Test package test-package");
        assert_eq!(pkg.source, PackageSource::Pacman);
        assert!(!pkg.is_installed);
    }

    #[test]
    fn test_app_initialization() {
        let app = App::new();
        
        assert_eq!(app.search_input, "");
        assert!(matches!(app.input_mode, InputMode::Normal));
        assert_eq!(app.results.len(), 0);
        assert!(!app.should_quit);
        assert!(!app.is_loading);
        assert!(app.selected_index.is_none());
        assert!(app.pending_command.is_none());
        assert!(app.error_message.is_none());
        assert!(app.available_updates.is_none());
        // show_password_prompt defaults to true on Unix, false on Windows
        #[cfg(not(target_os = "windows"))]
        assert!(app.show_password_prompt);
        #[cfg(target_os = "windows")]
        assert!(!app.show_password_prompt);
        assert!(app.password_input.is_empty());
        assert_eq!(app.selected_packages.len(), 0);
        assert!(!app.show_confirm_prompt);
        assert_eq!(app.packages_pending_confirmation.len(), 0);
        assert!(!app.show_console);
        assert_eq!(app.console_buffer.len(), 0);
        assert!(app.command_stdin_tx.is_none());
    }

    #[test]
    fn test_toggle_selection() {
        let mut app = App::new();

        let test_pkg = create_test_package("test-package", PackageSource::Pacman, false);

        app.results.push(test_pkg.clone());
        app.selected_index = Some(0);

        assert_eq!(app.selected_packages.len(), 0);

        app.toggle_selection();
        assert_eq!(app.selected_packages.len(), 1);
        assert!(app.selected_packages.contains_key("test-package"));

        app.toggle_selection();
        assert_eq!(app.selected_packages.len(), 0);
        assert!(!app.selected_packages.contains_key("test-package"));
    }

    #[test]
    fn test_navigation() {
        let mut app = App::new();

        for i in 0..3 {
            app.results.push(create_test_package(&format!("package-{}", i), PackageSource::Pacman, false));
        }

        assert!(app.selected_index.is_none());

        app.next();
        assert_eq!(app.selected_index, Some(0));

        app.next();
        assert_eq!(app.selected_index, Some(1));

        app.next();
        assert_eq!(app.selected_index, Some(2));

        app.next();
        assert_eq!(app.selected_index, Some(0));

        app.previous();
        assert_eq!(app.selected_index, Some(2));

        app.previous();
        assert_eq!(app.selected_index, Some(1));

        app.previous();
        assert_eq!(app.selected_index, Some(0));

        app.previous();
        assert_eq!(app.selected_index, Some(2));
    }
}