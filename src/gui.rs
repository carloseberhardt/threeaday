mod db;

use db::Database;
use glib::clone;
use gtk4::prelude::*;
use gtk4::{
    Application, ApplicationWindow, Button, CheckButton, Entry, Label, Orientation,
    Revealer, RevealerTransitionType, Align, Justification, EventControllerKey, gdk,
};
use gtk4 as gtk;
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

    fn refresh_tasks(&mut self) {
        // Clear existing tasks
        while let Some(child) = self.task_list.first_child() {
            self.task_list.remove(&child);
        }

        // Load tasks from database
        match self.db.get_today_tasks() {
            Ok(tasks) => {
                let completed = tasks.iter().filter(|t| t.completed).count();
                let total = tasks.len();

                // Update progress label
                if total == 0 {
                    self.progress_label.set_text("No tasks yet. Add one below!");
                } else if completed >= 3 {
                    self.progress_label.set_text(&format!("🎯 {} tasks completed!", completed));
                    self.completed_revealer.set_reveal_child(true);
                } else {
                    self.progress_label.set_text(&format!("Progress: {}/3 tasks completed", completed));
                    self.completed_revealer.set_reveal_child(false);
                }

                // Add task widgets (limit to 3 for UI simplicity)
                let display_tasks: Vec<_> = tasks.iter().take(3).collect();
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
                    
                    self.task_list.append(&task_box);

                    // Handle checkbox toggle - for now just mark complete
                    if !task.completed {
                        let task_id = task.id;
                        checkbox.connect_active_notify(move |cb| {
                            if cb.is_active() {
                                if let Ok(mut db) = Database::new() {
                                    let _ = db.complete_task(task_id);
                                    // For now, user needs to restart app to see changes
                                    // In a full implementation, we'd use a more sophisticated refresh mechanism
                                }
                            }
                        });
                    }
                }
                
                // Show note if there are more tasks beyond the 3 displayed
                if tasks.len() > 3 {
                    let more_box = GtkBox::new(Orientation::Horizontal, 8);
                    more_box.set_margin_top(8);
                    
                    let more_label = Label::new(Some(&format!("... and {} more tasks (use CLI to see all)", tasks.len() - 3)));
                    more_label.add_css_class("dim-label");
                    more_label.set_halign(Align::Center);
                    
                    more_box.append(&more_label);
                    self.task_list.append(&more_box);
                }
            }
            Err(e) => {
                self.progress_label.set_text(&format!("Error loading tasks: {}", e));
            }
        }
    }

    fn add_task(&mut self, text: &str) -> Result<(), anyhow::Error> {
        if text.trim().is_empty() {
            return Ok(());
        }

        // Check if we already have 3 tasks for today
        let tasks = self.db.get_today_tasks()?;
        if tasks.len() >= 3 {
            // Show a gentle reminder instead of adding
            self.progress_label.set_text("💡 Focus on your 3 tasks first! (Use CLI to add more)");
            self.entry.set_text("");
            return Ok(());
        }

        self.db.add_task(text.trim())?;
        self.entry.set_text("");
        self.refresh_tasks();
        
        Ok(())
    }
}

fn setup_callbacks(state: Rc<RefCell<AppState>>) {
    // Add button callback
    let add_button = state.borrow().add_button.clone();
    let entry_clone = state.borrow().entry.clone();
    
    add_button.connect_clicked(glib::clone!(@strong state, @strong entry_clone => move |_| {
        let text = entry_clone.text().to_string();
        if let Err(e) = state.borrow_mut().add_task(&text) {
            eprintln!("Error adding task: {}", e);
        }
    }));

    // Entry activation (Enter key)
    let entry = state.borrow().entry.clone();
    entry.connect_activate(glib::clone!(@strong state => move |entry| {
        let text = entry.text().to_string();
        if let Err(e) = state.borrow_mut().add_task(&text) {
            eprintln!("Error adding task: {}", e);
        }
    }));

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
    state.borrow_mut().refresh_tasks();
    
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