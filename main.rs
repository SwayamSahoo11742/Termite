// Import necessary modules and crates.
use std::*;  // Standard library for basic Rust functionality.
use process::exit;  // For gracefully exiting the program.
use time::Duration;  // To manage time durations, like frame rate control.
use crossterm::{event, execute, terminal, style, cursor};  // For terminal I/O operations (e.g., handling events, changing terminal styles).

// Modules that may include custom logic for screen handling, 3D models, and calculations.
mod screen;
mod three;
mod model;

// Configuration constants for viewport and camera settings.
const VIEWPORT_FOV: f32 = 1.7;  // Field of view for the camera.
const VIEWPORT_DISTANCE: f32 = 0.1;  // The default distance of the camera from the model.
const TARGET_DURATION_PER_FRAME: Duration = Duration::from_millis(1000 / 60);  // Target frame duration for 60 FPS.
const MOUSE_SPEED_MULTIPLIER: f32 = 30.;  // Multiplier to control mouse movement speed.
const INITIAL_DISTANCE_MULTIPLIER: f32 = 1.5;  // Initial zoom level for camera.
const SCROLL_MULTIPLIER: f32 = 0.03;  // Zoom in/out factor when scrolling.
const PAN_MULTIPLIER: f32 = 0.1;  // Factor for panning the camera view.
const HELP_MSG: &str = "\
\x1b[1mt3d\x1b[0m: Visualize .obj files in the terminal!

\x1b[1mUsage\x1b[0m:
    \"t3d <filepath.obj>\": Interactively view the provided .obj file.
    \"t3d --h\", \"t3d --help\", \"t3d -h\", \"t3d -help\", \"t3d\": Help and info.
    \"t3d --v\", \"t3d --version\", \"t3d -v\", \"t3d -version\": Get version info.

\x1b[1mControls\x1b[0m:
    Scroll down to zoom out, scroll up to zoom in.
    Click and drag the mouse to rotate around the model.
    Click and drag the mouse while holding [shift] to pan.

    Press [b] to toggle block mode. 
    Press [p] to toggle vertices mode. 
";

// Function to gracefully close the program by restoring terminal settings.
fn graceful_close() -> ! {
    execute!(
        io::stdout(),
        cursor::Show,  // Make the cursor visible again.
        event::DisableMouseCapture,  // Disable mouse capture in the terminal.
    ).unwrap();
    terminal::disable_raw_mode().unwrap();  // Restore terminal's original mode.
    exit(0);  // Exit the program.
}

// Function to close the program with an error message.
fn error_close(msg: &dyn fmt::Display) -> ! {
    execute!(
        io::stderr(),
        style::Print(msg)  // Print the error message to standard error.
    ).unwrap();
    graceful_close();  // Close the program after error.
}

fn main() {
    // Parse command-line arguments.
    let args: Vec<String> = env::args().collect();  // Collect arguments into a vector.
    if args.len() > 2 { error_close(&"Please supply only one file path to visualize.") }  // Error if more than one argument.
    if args.is_empty() { error_close(&"Error parsing arguments.") }  // Error if no arguments.

    // If the user requested help, display help message.
    let help_mode = args.len() == 1 || 
        ["-h", "-help", "--h", "--help"].map(String::from).contains(&args[1]);

    if help_mode {
        execute!(
            io::stdout(),
            style::Print(HELP_MSG)  // Print the help message.
        ).unwrap();
        graceful_close();  // Close the program after displaying help.
    }

    // If the user requested version information, display the version.
    let version_mode = args.len() == 1 || 
        ["-v", "-version", "--v", "--version"].map(String::from).contains(&args[1]);

    if version_mode {
        execute!(
            io::stdout(),
            style::Print(env!("CARGO_PKG_VERSION"))  // Print the program's version.
        ).unwrap();
        graceful_close();  // Close the program after displaying the version.
    }

    // Enable raw terminal mode (no line buffering, etc.) and hide the cursor for the interactive session.
    terminal::enable_raw_mode().unwrap();
    execute!(
        io::stdout(),
        cursor::Hide,  // Hide the cursor.
        event::EnableMouseCapture,  // Enable mouse tracking.
    ).unwrap();

    // Get the file path of the .obj file to visualize.
    let file_path = &args[1];
    
    // Attempt to load the model from the specified .obj file.
    let input_model = match model::Model::new_obj(file_path, three::Point::new(0., 0., 0.)) {
        Ok(model) => model,  // If successful, continue.
        Err(error) => error_close(&error)  // If error occurs, show error and exit.
    };

    // Calculate the center and diagonal of the model's bounding box.
    let bounds = input_model.world_bounds();
    let mut center = input_model.model_to_world(&three::Point::new(
        (bounds.0.x + bounds.1.x) / 2., 
        (bounds.0.y + bounds.1.y) / 2., 
        (bounds.0.z + bounds.1.z) / 2., 
    ));
    let diagonal = (
        (bounds.0.x - bounds.1.x).powi(2) +
        (bounds.0.y - bounds.1.y).powi(2) +
        (bounds.0.z - bounds.1.z).powi(2)
    ).sqrt();  // Diagonal distance to determine zoom level.

    // Set up the camera with the initial position and settings.
    let mut camera = three::Camera::new(
        center, 
        0., 0., 0.,  // Initial camera orientation (yaw, pitch, roll).
        VIEWPORT_DISTANCE, VIEWPORT_FOV,  // Initial camera distance and FOV.
    );

    // Initialize camera control variables (yaw, pitch, zoom level).
    let mut view_yaw: f32 = 0.0;
    let mut view_pitch: f32 = 0.0;
    let mut distance_to_model = diagonal * INITIAL_DISTANCE_MULTIPLIER;  // Distance scaled by model size.

    // Set initial rendering modes.
    let mut points_mode = false;  // Whether to render points (vertices) or edges.
    let mut braile_mode = true;  // Whether to render in Braille (or block mode).
    let mut pan_mode = false;  // Whether to pan the camera.

    // Initialize event tracking (mouse movements, clicks, etc.).
    let mut mouse_speed: (f32, f32) = (0., 0.);
    let mut last_mouse_position = screen::Point::new(0, 0);

    // Start the main loop that continuously renders the model.
    loop {
        let start = time::Instant::now();  // Track time for FPS calculations.
        let mut start_mouse_position = last_mouse_position;

        // Process events from the event queue.
        let mut event_count = 0;
        while event::poll(Duration::from_secs(0)).unwrap() {
            if let Ok(event) = event::read() {
                match event {
                    event::Event::Key(key_event) => {
                        let is_ctrl_c = key_event.modifiers == event::KeyModifiers::CONTROL
                            && key_event.code == event::KeyCode::Char('c');
                        
                        // Exit the program if Ctrl+C is pressed.
                        if is_ctrl_c { graceful_close() }

                        // Toggle points or edges rendering.
                        if key_event.code == event::KeyCode::Char('p') { points_mode = !points_mode }

                        // Toggle Braille or block mode for rendering.
                        if key_event.code == event::KeyCode::Char('b') { braile_mode = !braile_mode }
                    }

                    // Handle mouse events for navigation.
                    event::Event::Mouse(mouse_event) => {
                        let (x, y) = (mouse_event.column, mouse_event.row);
                        match mouse_event.kind {

                            // If mouse is clicked, record the initial position.
                            event::MouseEventKind::Down(_) => {
                                pan_mode = mouse_event.modifiers == event::KeyModifiers::SHIFT;
                                last_mouse_position.x = x as i32;
                                last_mouse_position.y = y as i32;
                                start_mouse_position = last_mouse_position;
                                event_count += 1;
                            }

                            // If the mouse is dragged, calculate movement speed.
                            event::MouseEventKind::Drag(_) => {
                                pan_mode = mouse_event.modifiers == event::KeyModifiers::SHIFT;
                                let delta_x = x as f32 - start_mouse_position.x as f32;
                                let delta_y = start_mouse_position.y as f32 - y as f32;
                                mouse_speed.0 = delta_x / camera.screen.width as f32 * MOUSE_SPEED_MULTIPLIER;
                                mouse_speed.1 = delta_y / camera.screen.width as f32 * MOUSE_SPEED_MULTIPLIER;
                                last_mouse_position = screen::Point::new(x as i32, y as i32);
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }

        // Handle camera movement based on mouse input (rotation and panning).
        if pan_mode {
            // Implement pan logic here.
        }

        // Wait for the next frame to maintain the target FPS.
        let elapsed = start.elapsed();
        if elapsed < TARGET_DURATION_PER_FRAME {
            std::thread::sleep(TARGET_DURATION_PER_FRAME - elapsed);
        }
    }
}
