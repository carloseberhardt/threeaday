use threeaday_core::{Database, Result, utils::*};
use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Button, CheckButton, Entry, Label, Orientation,
    Revealer, RevealerTransitionType, Justification, EventControllerKey, gdk,
};
use gtk4::Box as GtkBox;
use std::cell::RefCell;
use std::rc::Rc;

const APP_ID: &str = "dev.threeaday.ThreeADay";

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
    fn new(app: &Application) -> Result<Rc<RefCell<Self>>> {
        let db = Database::new()?;
        let window = ApplicationWindow::builder()
            .application(app)
            .title("ThreeADay")
            .default_width(400)
            .default_height(300)
            .build();

        let main_box = GtkBox::new(Orientation::Vertical, 12);
        main_box.set_margin_top(20);
        main_box.set_margin_bottom(20);
        main_box.set_margin_start(20);
        main_box.set_margin_end(20);

        // Progress label
        let progress_label = Label::new(None);
        progress_label.set_justify(Justification::Center);
        progress_label.add_css_class("progress-label");
        main_box.append(&progress_label);

        // Task list
        let task_list = GtkBox::new(Orientation::Vertical, 8);
        task_list.add_css_class("task-list");
        main_box.append(&task_list);

        // Add task section
        let add_section = GtkBox::new(Orientation::Horizontal, 8);
        add_section.set_hexpand(true);
        
        let entry = Entry::new();
        entry.set_placeholder_text(Some("Add a new task..."));
        entry.set_hexpand(true);
        add_section.append(&entry);
        
        let add_button = Button::with_label("Add Task");
        add_button.add_css_class("suggested-action");
        add_section.append(&add_button);
        
        main_box.append(&add_section);

        // Completed revealer for celebration
        let completed_revealer = Revealer::new();
        completed_revealer.set_transition_type(RevealerTransitionType::SlideDown);
        completed_revealer.set_reveal_child(false);
        
        let completed_label = Label::new(Some("🎉 Daily goal achieved! Great job! 🎉"));
        completed_label.add_css_class("success-label");
        completed_revealer.set_child(Some(&completed_label));
        main_box.append(&completed_revealer);

        window.set_child(Some(&main_box));
        
        let state = Rc::new(RefCell::new(AppState {
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
                // Update progress label
                let completed_count = tasks.iter().filter(|t| t.completed).count();
                let total_count = tasks.len();
                
                let progress_text = if is_daily_goal_achieved(completed_count) {
                    format!("🎯 Daily goal achieved! ({} tasks completed)", completed_count)
                } else {
                    format!("Progress: {}/{} tasks completed", completed_count, total_count)
                };
                
                state.progress_label.set_text(&progress_text);
                state.progress_label.remove_css_class("error-label");
                state.progress_label.add_css_class("progress-label");
                
                // Show celebration if goal achieved
                state.completed_revealer.set_reveal_child(is_daily_goal_achieved(completed_count));
                
                // Show only first 3 tasks (GUI enforces focus)
                let display_tasks = tasks.iter().take(3);
                let remaining_count = tasks.len().saturating_sub(3);
                
                for task in display_tasks {
                    let task_box = GtkBox::new(Orientation::Horizontal, 8);
                    task_box.set_hexpand(true);
                    task_box.add_css_class("task-item");
                    
                    let checkbox = CheckButton::new();
                    checkbox.set_active(task.completed);
                    checkbox.set_sensitive(!task.completed);
                    task_box.append(&checkbox);
                    
                    let task_label = Label::new(Some(&task.text));
                    task_label.set_hexpand(true);
                    task_label.set_xalign(0.0);
                    
                    if task.completed {
                        task_label.add_css_class("completed-task");
                    }
                    
                    task_box.append(&task_label);
                    state.task_list.append(&task_box);

                    // Handle checkbox toggle
                    if !task.completed {
                        let task_id = task.id;
                        let self_rc_weak = Rc::downgrade(self_rc);
                        checkbox.connect_active_notify(move |cb| {
                            if cb.is_active() {
                                if let Some(strong_self_rc) = self_rc_weak.upgrade() {
                                    let mut current_app_state = strong_self_rc.borrow_mut();
                                    match current_app_state.db.complete_task(task_id) {
                                        Ok(true) => {
                                            drop(current_app_state);
                                            Self::refresh_tasks(&strong_self_rc);
                                        }
                                        Ok(false) => {
                                            // Task was already complete or not found
                                        }
                                        Err(e) => {
                                            eprintln!("Error completing task {}: {}", task_id, e);
                                        }
                                    }
                                }
                            }
                        });
                    }
                }
                
                // Show note if there are more tasks beyond the 3 displayed
                if remaining_count > 0 {
                    let more_label = Label::new(Some(&format!("... and {} more task(s) (use CLI to see all)", remaining_count)));
                    more_label.add_css_class("dim-label");
                    state.task_list.append(&more_label);
                }
                
                // Show encouragement if no tasks
                if tasks.is_empty() {
                    let empty_label = Label::new(Some("No tasks yet. Add your first task below!"));
                    empty_label.add_css_class("dim-label");
                    state.task_list.append(&empty_label);
                }
            }
            Err(e) => {
                let error_message = format!("Error loading tasks: {}", e);
                state.progress_label.set_text(&error_message);
                state.progress_label.remove_css_class("progress-label");
                state.progress_label.add_css_class("error-label");
                eprintln!("{}", error_message);
            }
        }
    }

    fn add_task(self_rc: &Rc<RefCell<Self>>, text: &str) -> Result<()> {
        if text.trim().is_empty() {
            return Ok(());
        }
        
        let mut state = self_rc.borrow_mut();
        
        // Check if we already have 3 tasks (GUI enforces focus)
        let tasks = state.db.get_today_tasks()?;
        if tasks.len() >= DAILY_GOAL_COMPLETION_COUNT {
            state.progress_label.set_text("Focus on your 3 tasks first! Complete some before adding more.");
            state.progress_label.remove_css_class("progress-label");
            state.progress_label.add_css_class("error-label");
            return Ok(());
        }
        
        state.db.add_task(text)?;
        state.entry.set_text("");
        
        drop(state);
        Self::refresh_tasks(self_rc);
        
        Ok(())
    }

    fn handle_add_task_error(&self, error: &anyhow::Error) {
        let error_message = format!("Error adding task: {}", error);
        self.progress_label.set_text(&error_message);
        self.progress_label.remove_css_class("progress-label");
        self.progress_label.add_css_class("error-label");
        eprintln!("{}", error_message);
    }
}

fn setup_css() {
    let css_provider = gtk4::CssProvider::new();
    css_provider.load_from_data(r#"
        .progress-label {
            font-size: 14px;
            font-weight: bold;
            margin-bottom: 12px;
        }
        
        .error-label {
            color: #e74c3c;
            font-size: 14px;
            font-weight: bold;
            margin-bottom: 12px;
        }
        
        .success-label {
            color: #27ae60;
            font-size: 16px;
            font-weight: bold;
            margin: 12px 0;
        }
        
        .task-item {
            padding: 8px;
            margin: 4px 0;
            border-radius: 4px;
            background-color: alpha(@theme_bg_color, 0.5);
        }
        
        .completed-task {
            text-decoration: line-through;
            opacity: 0.7;
        }
        
        .dim-label {
            opacity: 0.7;
            font-style: italic;
        }
    "#);
    
    gtk4::style_context_add_provider_for_display(
        &gdk::Display::default().expect("Error initializing gtk css provider."),
        &css_provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn setup_callbacks(state: &Rc<RefCell<AppState>>) {
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

    // Escape key handling
    let window = state.borrow().window.clone();
    let key_controller = EventControllerKey::new();
    key_controller.connect_key_pressed(move |_, key, _, _| {
        if key == gdk::Key::Escape {
            window.close();
            glib::Propagation::Stop
        } else {
            glib::Propagation::Proceed
        }
    });
    
    state.borrow().window.add_controller(key_controller);
}

pub fn run_gui() -> Result<()> {
    let app = Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(move |app| {
        setup_css();
        
        match AppState::new(app) {
            Ok(state) => {
                setup_callbacks(&state);
                AppState::refresh_tasks(&state);
                
                // Focus on the window and entry field
                state.borrow().window.present();
                state.borrow().entry.grab_focus();
            }
            Err(e) => {
                eprintln!("Failed to initialize GUI: {}", e);
                app.quit();
            }
        }
    });

    app.run();
    Ok(())
}