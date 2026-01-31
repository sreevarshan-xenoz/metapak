#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Package, PackageSource};

    #[test]
    fn test_package_creation() {
        let pkg = Package {
            name: "test-package".to_string(),
            version: "1.0.0".to_string(),
            description: "A test package".to_string(),
            source: PackageSource::Pacman,
            is_installed: false,
        };

        assert_eq!(pkg.name, "test-package");
        assert_eq!(pkg.version, "1.0.0");
        assert_eq!(pkg.description, "A test package");
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
        assert!(app.show_password_prompt);
        assert_eq!(app.password_input, "");
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
        
        // Add a test package to results
        let test_pkg = Package {
            name: "test-package".to_string(),
            version: "1.0.0".to_string(),
            description: "A test package".to_string(),
            source: PackageSource::Pacman,
            is_installed: false,
        };
        
        app.results.push(test_pkg.clone());
        app.selected_index = Some(0);

        // Initially not selected
        assert_eq!(app.selected_packages.len(), 0);

        // Toggle selection
        app.toggle_selection();
        assert_eq!(app.selected_packages.len(), 1);
        assert!(app.selected_packages.contains_key("test-package"));

        // Toggle again to deselect
        app.toggle_selection();
        assert_eq!(app.selected_packages.len(), 0);
        assert!(!app.selected_packages.contains_key("test-package"));
    }

    #[test]
    fn test_navigation() {
        let mut app = App::new();
        
        // Add some test packages
        for i in 0..3 {
            app.results.push(Package {
                name: format!("package-{}", i),
                version: "1.0.0".to_string(),
                description: format!("Test package {}", i),
                source: PackageSource::Pacman,
                is_installed: false,
            });
        }

        // Initially no selection
        assert!(app.selected_index.is_none());

        // Move to next (should wrap to 0)
        app.next();
        assert_eq!(app.selected_index, Some(0));

        // Move to next
        app.next();
        assert_eq!(app.selected_index, Some(1));

        // Move to next (should wrap to 0 when reaching end)
        app.next();
        assert_eq!(app.selected_index, Some(2));

        app.next();
        assert_eq!(app.selected_index, Some(0)); // Wrap around

        // Move to previous
        app.previous();
        assert_eq!(app.selected_index, Some(2));

        app.previous();
        assert_eq!(app.selected_index, Some(1));

        app.previous();
        assert_eq!(app.selected_index, Some(0));

        app.previous();
        assert_eq!(app.selected_index, Some(2)); // Wrap around
    }
}