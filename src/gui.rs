mod db;

use db::Database;
use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Button, CheckButton, Entry, Label, Orientation,
    Revealer, RevealerTransitionType, Align, Justification, EventControllerKey, gdk,
};
use gtk4::Box as GtkBox;
use std::cell::RefCell;
use std::rc::Rc;

const APP_ID: &str = "dev.threeaday.ThreeADay";
const DAILY_GOAL_COMPLETION_COUNT: usize = 3;

struct AppState {
    db: Database,
    window: ApplicationWindow,
    task_list: GtkBox,
    progress_label: Label,
    entry: Entry,
    add_button: Button,
    completed_revealer: Revealer,
}

impl AppState {
    fn new(app: &Application) -> Result<Rc<RefCell<Self>>, anyhow::Error> {
        let db = Database::new()?;

        // Create window
        let window = ApplicationWindow::builder()
            .application(app)
            .title("ThreeADay")
            .default_width(350)
            .default_height(400)
            .resizable(false)
            .build();

        // Main container
        let main_box = GtkBox::new(Orientation::Vertical, 12);
        main_box.set_margin_top(16);
        main_box.set_margin_bottom(16);
        main_box.set_margin_start(16);
        main_box.set_margin_end(16);

        // Header with progress
        let header_box = GtkBox::new(Orientation::Vertical, 8);
        let title_label = Label::new(Some("ThreeADay"));
        title_label.add_css_class("title-1");
        
        let progress_label = Label::new(Some("Loading..."));
        progress_label.add_css_class("dim-label");
        
        header_box.append(&title_label);
        header_box.append(&progress_label);

        // Task input section
        let input_box = GtkBox::new(Orientation::Horizontal, 8);
        let entry = Entry::builder()
            .placeholder_text("Add a task...")
            .hexpand(true)
            .build();
        
        let add_button = Button::with_label("Add");
        add_button.add_css_class("suggested-action");
        
        input_box.append(&entry);
        input_box.append(&add_button);

        // Task list container
        let task_list = GtkBox::new(Orientation::Vertical, 8);
        
        // Completion celebration
        let completed_revealer = Revealer::builder()
            .transition_type(RevealerTransitionType::SlideDown)
            .transition_duration(500)
            .build();
        
        let celebration_box = GtkBox::new(Orientation::Vertical, 8);
        celebration_box.set_halign(Align::Center);
        
        let celebration_label = Label::new(Some("🎯 Goal Achieved!"));
        celebration_label.add_css_class("title-2");
        
        let subtitle_label = Label::new(Some("Great momentum! You've completed your daily goals."));
        subtitle_label.add_css_class("dim-label");
        subtitle_label.set_wrap(true);
        subtitle_label.set_justify(Justification::Center);
        
        celebration_box.append(&celebration_label);
        celebration_box.append(&subtitle_label);
        completed_revealer.set_child(Some(&celebration_box));

        // Assemble layout
        main_box.append(&header_box);
        main_box.append(&input_box);
        main_box.append(&task_list);
        main_box.append(&completed_revealer);
        
        window.set_child(Some(&main_box));

        let state = Rc::new(RefCell::new(Self {
            db,
            window,
            task_list,
            progress_label,
            entry,
            add_button,
            completed_revealer,
        }));

        Ok(state)
    }

    fn refresh_tasks(self_rc: &Rc<RefCell<Self>>) {
        let state = self_rc.borrow_mut();
        // Clear existing tasks
        while let Some(child) = state.task_list.first_child() {
            state.task_list.remove(&child);
        }

        // Load tasks from database
        match state.db.get_today_tasks() {
            Ok(tasks) => {
                let completed = tasks.iter().filter(|t| t.completed).count();
                let total = tasks.len();

                // Update progress label
                if total == 0 {
                    state.progress_label.set_text("No tasks yet. Add one below!");
                } else if completed >= DAILY_GOAL_COMPLETION_COUNT {
                    state.progress_label.set_text(&format!("🎯 {} tasks completed!", completed));
                    state.completed_revealer.set_reveal_child(true);
                } else {
                    state.progress_label.set_text(&format!("Progress: {}/{} tasks completed", completed, DAILY_GOAL_COMPLETION_COUNT));
                    state.completed_revealer.set_reveal_child(false);
                }

                // Add task widgets (limit to 3 for UI simplicity)
                let display_tasks: Vec<_> = tasks.iter().take(DAILY_GOAL_COMPLETION_COUNT).collect();
                for task in display_tasks {
                    let task_box = GtkBox::new(Orientation::Horizontal, 8);
                    task_box.set_margin_top(4);
                    task_box.set_margin_bottom(4);

                    let checkbox = CheckButton::new();
                    checkbox.set_active(task.completed);
                    
                    let task_label = Label::new(Some(&task.text));
                    task_label.set_hexpand(true);
                    task_label.set_halign(Align::Start);
                    
                    if task.completed {
                        task_label.add_css_class("dim-label");
                    }

                    task_box.append(&checkbox);
                    task_box.append(&task_label);
                    
                    state.task_list.append(&task_box);

                    // Handle checkbox toggle
                    if !task.completed {
                        let task_id = task.id;
                        let self_rc_weak = Rc::downgrade(self_rc); // Explicitly create Weak reference
                        checkbox.connect_active_notify(move |cb| {
                            if cb.is_active() {
                                if let Some(strong_self_rc) = self_rc_weak.upgrade() { // Upgrade the captured Weak
                                    let mut current_app_state_for_callback = strong_self_rc.borrow_mut();
                                    match current_app_state_for_callback.db.complete_task(task_id) {
                                        Ok(true) => { // Task was successfully completed
                                            drop(current_app_state_for_callback);
                                            AppState::refresh_tasks(&strong_self_rc);
                                        }
                                        Ok(false) => {
                                            // Task was already complete or not found
                                        }
                                        Err(e) => {
                                            eprintln!("Error completing task {} during callback: {}", task_id, e);
                                        }
                                    }
                                } else {
                                    eprintln!("AppState dropped, checkbox callback for task_id {} will not run.", task_id);
                                }
                            }
                        });
                    }
                }
                
                // Show note if there are more tasks beyond the 3 displayed
                if tasks.len() > DAILY_GOAL_COMPLETION_COUNT {
                    let more_box = GtkBox::new(Orientation::Horizontal, 8);
                    more_box.set_margin_top(8);
                    
                    let more_label = Label::new(Some(&format!("... and {} more tasks (use CLI to see all)", tasks.len() - DAILY_GOAL_COMPLETION_COUNT)));
                    more_label.add_css_class("dim-label");
                    more_label.set_halign(Align::Center);
                    
                    more_box.append(&more_label);
                    state.task_list.append(&more_box);
                }
            }
            Err(e) => {
                let error_message = state._format_task_loading_error_message(&e);
                eprintln!("GUI: {}", error_message); // Log the error
                state.progress_label.set_text(&error_message);
                state.progress_label.remove_css_class("dim-label"); // Ensure not dim
                state.progress_label.add_css_class("error-label"); // Consistent error styling
            }
        }
    }

    fn add_task(self_rc: &Rc<RefCell<Self>>, text: &str) -> Result<(), anyhow::Error> {
        let mut state = self_rc.borrow_mut();
        if text.trim().is_empty() {
            return Ok(());
        }

        // Check if we already have 3 tasks for today
        let tasks = state.db.get_today_tasks()?;
        if tasks.len() >= DAILY_GOAL_COMPLETION_COUNT {
            // Show a gentle reminder instead of adding
            state.progress_label.set_text("💡 Focus on your 3 tasks first! (Use CLI to add more)");
            state.entry.set_text("");
            return Ok(());
        }

        state.db.add_task(text.trim())?;
        state.entry.set_text("");
        // Release the mutable borrow before calling refresh_tasks,
        // as refresh_tasks will also need to borrow_mut().
        drop(state);
        AppState::refresh_tasks(self_rc);
        
        Ok(())
    }

    fn _format_task_loading_error_message(&self, error: &anyhow::Error) -> String {
        format!("Error loading tasks: {}", error)
    }

    /// Handles the UI updates when an error occurs during task addition.
    ///
    /// This includes logging the error to the console, displaying a
    /// formatted error message in the progress_label, and adjusting
    /// CSS classes on the label for visual feedback.
    fn handle_add_task_error(&self, error: &anyhow::Error) {
        let error_message = format!("Error adding task: {}", error);
        eprintln!("{}", error_message); // Keep console log for debugging
        self.progress_label.set_text(&error_message);
        self.progress_label.remove_css_class("dim-label"); // Ensure not dim
        self.progress_label.add_css_class("error-label"); // Example class
    }
}

fn setup_callbacks(state: Rc<RefCell<AppState>>) {
    // Add button callback
    let add_button = state.borrow().add_button.clone();
    let entry_clone = state.borrow().entry.clone();
    
    add_button.connect_clicked(glib::clone!(
        #[strong] state,
        #[strong] entry_clone,
        move |_| {
            let text = entry_clone.text().to_string();
            if let Err(e) = AppState::add_task(&state, &text) {
                state.borrow().handle_add_task_error(&e);
            }
        }
    ));

    // Entry activation (Enter key)
    let entry = state.borrow().entry.clone();
    entry.connect_activate(glib::clone!(
        #[strong] state,
        move |entry| {
            let text = entry.text().to_string();
            if let Err(e) = AppState::add_task(&state, &text) {
                state.borrow().handle_add_task_error(&e);
            }
        }
    ));

    // Window close behavior - allow closing
    let window = state.borrow().window.clone();
    window.connect_close_request(move |_| {
        glib::Propagation::Proceed
    });

    // Add Escape key to close window
    let key_controller = EventControllerKey::new();
    let window_clone = state.borrow().window.clone();
    key_controller.connect_key_pressed(move |_, key, _, _| {
        if key == gdk::Key::Escape {
            window_clone.close();
            glib::Propagation::Stop
        } else {
            glib::Propagation::Proceed
        }
    });
    state.borrow().window.add_controller(key_controller);
}

fn build_ui(app: &Application) -> Result<(), anyhow::Error> {
    let state = AppState::new(app)?;
    
    // Set up callbacks
    setup_callbacks(state.clone());
    
    // Initial task refresh
    AppState::refresh_tasks(&state);
    
    // Show window and give it focus
    let window = &state.borrow().window;
    window.present();
    window.set_focus_visible(true);
    
    Ok(())
}

fn main() -> glib::ExitCode {
    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(|app| {
        if let Err(e) = build_ui(app) {
            eprintln!("Error building UI: {}", e);
        }
    });

    app.run()
}

#[cfg(test)]
mod tests {
    use super::*; // If AppState or other items from gui.rs are needed, though not for this simple test.
    use anyhow::anyhow;

    // The format_task_loading_error free function and its test test_format_task_loading_error_message
    // have been removed as per the task description. The formatting logic is now
    // a private method in AppState and is considered trivial for direct unit testing,
    // its effect being implicitly covered by observing the GUI label.
}