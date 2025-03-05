use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use clipboard::{ClipboardContext, ClipboardProvider};
use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    menu::{MenuBar as Menu, MenuItem, MenuItemAttributes},
    system_tray::{SystemTray, SystemTrayBuilder, SystemTrayEvent},
    window::WindowBuilder,
};

// Struct to hold the application state
struct AppState {
    clipboard_history: VecDeque<String>,
    clipboard_context: Option<ClipboardContext>,
    last_clipboard_content: Option<String>,
    is_first_run: bool
}


fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("9clip")
        .with_visible(false)
        .build(&event_loop)
        .expect("Failed to create window");
    
    // Initialize the clipboard context
    let clipboard_context_result = ClipboardContext::new();
    let clipboard_context = match clipboard_context_result {
        Ok(ctx) => Some(ctx),
        Err(e) => {
            eprintln!("Failed to initialize clipboard context: {}", e);
            None
        }
    };

    // Initialize application state with Mutex
    let app_state = Arc::new(Mutex::new(AppState {
        clipboard_history: VecDeque::with_capacity(9),
        clipboard_context: clipboard_context,
        last_clipboard_content: None,
        is_first_run: true,
    }));

    let mut tray_menu = Menu::new();

    tray_menu.add_item(MenuItemAttributes::new("No clipboard history yet."));

    let _system_tray = SystemTrayBuilder::new(
        tao::system_tray::Icon::from_path("icon.png", None).expect("Failed to load icon"),
        Some(tray_menu),
    )
    .build(&event_loop)
    .expect("Failed to create system tray");
    
    let app_state_clone = app_state.clone();
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_millis(100));
            let mut app_state = app_state_clone.lock().unwrap();
            
             if app_state.clipboard_context.is_none() {
                let clipboard_context_result = ClipboardContext::new();
                 app_state.clipboard_context = match clipboard_context_result {
                    Ok(ctx) => Some(ctx),
                    Err(e) => {
                        eprintln!("Failed to initialize clipboard context: {}", e);
                        None
                    }
                };
             }

            if let Some(clipboard_context) = &mut app_state.clipboard_context {
                let clipboard_content_result = clipboard_context.get_contents();

                match clipboard_content_result {
                    Ok(clipboard_content) => {
                        if !clipboard_content.is_empty(){
                            if let Some(last_content) = &app_state.last_clipboard_content {
                                if clipboard_content == *last_content {
                                    continue; // Skip if the content is the same as last time
                                }
                            }

                            app_state.last_clipboard_content = Some(clipboard_content.clone());
                            app_state.clipboard_history.push_front(clipboard_content.clone());
                            if app_state.clipboard_history.len() > 9 {
                                app_state.clipboard_history.pop_back();
                            }
                            
                            if app_state.is_first_run {
                                app_state.is_first_run = false;
                            }
                        }
                        

                    }
                    Err(e) => {
                        eprintln!("Failed to get clipboard content: {}", e);
                        app_state.clipboard_context = None;
                    }
                }
            }
        }
    });

    let app_state_clone = app_state.clone();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        let mut app_state = app_state_clone.lock().unwrap();
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::SystemTrayEvent {
                event: SystemTrayEvent::MenuItemClick(id),
                ..
            } => {
                if let Some(item) = app_state
                .clipboard_history
                .get(id - 100){
                    if let Some(clipboard_context) = &mut app_state.clipboard_context {
                        match clipboard_context.set_contents(item.clone()){
                            Ok(_) => {
                                println!("Pasted from clipboard history: {}",item)
                            },
                             Err(e) => {
                               eprintln!("Failed to set clipboard context: {}", e);
                               app_state.clipboard_context = None;
                            }
                        }
                    }
                }
            },
            Event::MainEventsCleared => {
                if !app_state.is_first_run {
                   
                    let mut tray_menu = Menu::new();
                    if !app_state.clipboard_history.is_empty(){
                        for (index, item) in app_state.clipboard_history.iter().enumerate(){
                            tray_menu.add_item(MenuItemAttributes::new(&format!("{} {}", index +1 , item)).with_id(100 + index as u32));
                        }
                    } else {
                        tray_menu.add_item(MenuItemAttributes::new("No clipboard history yet."));
                    }

                     if let Some(system_tray) = &tao::system_tray::SystemTray::from_window_id(window.id()){
                         system_tray.set_menu(tray_menu);
                     }
                    
                }
                
            },
            _ => (),
        }
    });
}