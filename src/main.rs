use std::io::{self, Write};
use std::time::Duration;
use crossterm::{
    cursor, execute, event::{self, Event, KeyCode, KeyEvent},
    style::{self, Color, Stylize},
    terminal::{Clear, ClearType, enable_raw_mode, disable_raw_mode},
};
use ctrlc;

const WORK_TIME: u64 = 25 * 60; // 25 minutes
const BREAK_TIME: u64 = 5 * 60; // 5 minutes

struct Timer {
    duration: u64,
    elapsed: u64,
}

impl Timer {
    fn new(duration: u64) -> Timer {
        Timer {
            duration,
            elapsed: 0,
        }
    }

    fn get_progress(&self) -> f32 {
        self.elapsed as f32 / self.duration as f32 * 100.0
    }

    fn format_time(&self) -> String {
        let remaining = self.duration - self.elapsed;
        let minutes = remaining / 60;
        let seconds = remaining % 60;
        format!("{:02}:{:02}", minutes, seconds)
    }
}

fn clear_screen() -> io::Result<()> {
    execute!(io::stdout(), Clear(ClearType::All), cursor::MoveTo(0, 0))
}

fn draw_progress_bar(progress: f32, time: &str, message: &str, first_draw: bool) -> io::Result<()> {
    let width = 50;
    let filled = (progress * width as f32 / 100.0) as usize;
    let empty = width - filled;

    if first_draw {
        display_header()?;
    }
    
    // Move cursor to specific positions for each line
    execute!(io::stdout(), cursor::MoveTo(0, 3))?;
    execute!(io::stdout(), Clear(ClearType::FromCursorDown))?;

    // Progress bar line
    print!("[");
    execute!(io::stdout(), 
        style::PrintStyledContent("=".repeat(filled).with(Color::Green)))?;
    execute!(io::stdout(), 
        style::PrintStyledContent("-".repeat(empty).with(Color::DarkGrey)))?;
    print!("] {}% {}", progress as u32, time);

    // Message line
    execute!(io::stdout(), cursor::MoveTo(0, 5))?;
    print!("{}", message);

    // Controls line
    execute!(io::stdout(), cursor::MoveTo(0, 7))?;
    print!("Controls: 'q' to quit, 'r' to reset timer");
    
    io::stdout().flush()
}

enum TimerResult {
    Completed,
    Quit,
    Reset,
}

fn run_timer(duration: u64, type_name: &str) -> io::Result<TimerResult> {
    let mut timer = Timer::new(duration);
    enable_raw_mode()?;
    execute!(io::stdout(), cursor::Hide)?;  // Hide cursor at the start

    let message = format!("Current session: {}", type_name);
    draw_progress_bar(timer.get_progress(), &timer.format_time(), &message, true)?;

    let result = loop {
        if event::poll(Duration::from_secs(1))? {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Char('q') | KeyCode::Char('Q') => {
                        break TimerResult::Quit;
                    }
                    KeyCode::Char('r') | KeyCode::Char('R') => {
                        break TimerResult::Reset;
                    }
                    _ => {}
                }
            }
        }
        timer.elapsed += 1;
        if timer.elapsed >= timer.duration {
            break TimerResult::Completed;
        }
        draw_progress_bar(timer.get_progress(), &timer.format_time(), &message, false)?;
    };

    execute!(io::stdout(), cursor::Show)?;
    disable_raw_mode()?;

    match result {
        TimerResult::Quit => {
            display_header()?;
            println!("Pomodoro session ended. See you next time!");
        }
        TimerResult::Reset => {
            display_header()?;
            println!("Timer reset.");
        }
        TimerResult::Completed => {
            print!("\x07");
            io::stdout().flush()?;
        }
    }
    Ok(result)
}

fn display_header() -> io::Result<()> {
    clear_screen()?;
    println!("\n🍅 Tìmeadair - Pomodoro Timer\n");
    Ok(())
}

fn prompt_session(session_type: &str) -> io::Result<bool> {
    display_header()?;
    execute!(io::stdout(), cursor::Show)?;
    print!("Start {} session? [Y/n]: ", session_type);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().is_empty() || input.trim().to_lowercase().starts_with('y'))
}

fn main() -> io::Result<()> {
    ctrlc::set_handler(move || {
        let _ = execute!(io::stdout(), cursor::Show);
        let _ = disable_raw_mode();
        let _ = display_header();
        println!("Pomodoro session ended. See you next time!");
        std::process::exit(0);
    }).expect("Error setting Ctrl-C handler");

    loop {
        if !prompt_session("work")? {
            display_header()?;
            println!("Pomodoro session ended. See you next time!");
            break;
        }
        match run_timer(WORK_TIME, "Work")? {
            TimerResult::Completed => {
                // Continue to break prompt
            }
            TimerResult::Quit => break,
            TimerResult::Reset => continue, // Go back to work session prompt
        }
        if prompt_session("break")? {
            // Break session
            match run_timer(BREAK_TIME, "Break")? {
                TimerResult::Completed => {
                    // Continue to next work session
                }
                TimerResult::Quit => break,
                TimerResult::Reset => continue, // Go back to work session prompt
            }
        }
    }

    execute!(io::stdout(), cursor::Show)?;
    Ok(())
}