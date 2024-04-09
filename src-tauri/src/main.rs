// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use commands::timer::TauriTimer;
// Give us easier access to some values
use serde::Serialize;
use std::sync::{Arc, Mutex, PoisonError};
use tauri::State;
use squirrel3_lib::noise_rng::NoiseRngU64;

mod commands;

// Create a system state struct with the properties we want it to manage
#[derive(Serialize, Clone)]
pub(crate) struct SystemState {
    // An integer from 0 to 100 indicating the power percentage
    power: i32,
}
// Allows us to run SystemState::default()
impl Default for SystemState {
    fn default() -> Self {
        // Default to 100% power
        Self { power: 100 }
    }
}

// Create an AuthState to manage wether we are autenticated or not
#[derive(Serialize, Clone)]
pub(crate) struct AuthState {
    // Ensures token is not passed to the front end
    // #[serde(skip_serializing)]
    // token: Option<String>,
    // Boolean to indicate if loged in
    logged_in: bool,
}

// Allow us to run AuthState::default()
impl Default for AuthState {
    fn default() -> Self {
        Self {
            // Before we log in we don't have a token
            // token: None,
            // And we are not logged in
            logged_in: false,
        }
    }
}

#[derive(Serialize, Clone)]
pub(crate) struct SystemClock {
    #[serde(skip_serializing)]
    clock: commands::clock::TauriClock,
    hours: u8,
    minutes: u8,
    seconds: u8,
    milliseconds: usize,
}

impl SystemClock {
    fn fill_timer(&mut self) {
        let (hours, minutes, seconds) = self.clock.timer.get_digits();
        let milliseconds = self.clock.timer.get_millis();
        self.hours = hours;
        self.minutes = minutes;
        self.seconds = seconds;
        self.milliseconds = milliseconds;
        // (hours, minutes, seconds, milliseconds)
    }
}

impl Default for SystemClock {
    fn default() -> Self {
        Self {
            clock: commands::clock::TauriClock::new(),
            hours: 0,
            minutes: 0,
            seconds: 0,
            milliseconds: 0,
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub(crate) struct TimerList {
    timers: Vec<commands::timer::TauriTimer>,
}

impl TimerList {
    fn push_timer(&mut self, timer: commands::timer::TauriTimer) {
        self.timers.push(timer);
    }
}

impl Default for TimerList {
    fn default() -> Self {
        Self {
            timers: vec![]
        }
    }
}

// Create a custom error that we can retunr in results
#[derive(Debug, thiserror::Error)]
enum Error {
    // Implement std::io::Error for our Error enum
    #[error(transparent)]
    Io(#[from] std::io::Error),
    // Trying a custom error for wrong querys from typescript
    #[error("the mutex was poisoned")]
    PoisonError(String),
}

// Implement Serialize for Error
impl serde::Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

// Implement From<PoisonError> for Error to convert it into something we have setup serialization
// for
impl<T> From<PoisonError<T>> for Error {
    fn from(value: PoisonError<T>) -> Self {
        // We just convert the error to a string here
        Error::PoisonError(value.to_string())
    }
}

// Create a command that log us in
#[tauri::command]
async fn login(state_mutex: State<'_, Mutex<AuthState>>) -> Result<AuthState, Error> {
    println!("Logging in...");
    let mut state = state_mutex.lock()?;
    // Login logic
    state.logged_in = true;
    // Sends back a clone of the state
    Ok(state.clone())
}

// Create a commmand that log us out
#[tauri::command]
async fn logout(state_mutex: State<'_, Mutex<AuthState>>) -> Result<AuthState, Error> {
    println!("Logging out...");
    let mut state = state_mutex.lock()?;
    // Login logic
    state.logged_in = false;
    // Sends back a clone of the state
    Ok(state.clone())
}

// Create a commmand that shows current state
#[tauri::command]
async fn get_login(state_mutex: State<'_, Mutex<AuthState>>) -> Result<AuthState, Error> {
    let state = state_mutex.lock()?;
    Ok(state.clone())
}

#[tauri::command]
async fn get_clock(state_mutex: State<'_, Arc<Mutex<SystemClock>>>) -> Result<SystemClock, Error> {
    let state = state_mutex.lock()?;
    Ok(state.clone())
}

#[tauri::command]
async fn reset_power(
    system_state_mutex: State<'_, Arc<Mutex<SystemState>>>,
    ) -> Result<SystemState, Error> {
    let mut state = system_state_mutex.lock()?;
    state.power = 0;
    Ok(state.clone())
}

#[tauri::command]
async fn get_power(
    system_state_mutex: State<'_, Arc<Mutex<SystemState>>>,
    ) -> Result<SystemState, Error> {
    let state = system_state_mutex.lock()?;
    Ok(state.clone())
}

#[tauri::command]
async fn push_timer(
    timer_list_mutex: State<'_, Arc<Mutex<TimerList>>>,
    seconds: usize,
    ) -> Result<TauriTimer, Error> {
    let mut state = timer_list_mutex.lock()?;
    let mut new_timer = TauriTimer::new(seconds);
    let list_size = state.timers.len();
    new_timer.set_id(list_size + 1);
    let timer_clone = new_timer.clone();
    state.push_timer(new_timer);
    Ok(timer_clone)
}

#[tauri::command]
async fn get_timer_list(
    timer_list_mutex: State<'_, Arc<Mutex<TimerList>>>,
    ) -> Result<TimerList, Error> {
    let state = timer_list_mutex.lock()?;
    Ok(state.clone())
}

#[tauri::command]
async fn get_timer_by_index(
    timer_list_mutex: State<'_, Arc<Mutex<TimerList>>>,
    index: usize,
    ) -> Result<TauriTimer, Error> {
    let state = timer_list_mutex.lock()?;
    let timer: &TauriTimer;
    match state.timers.get(index) {
        Some(tmr) => {
            timer = tmr;
        },
        None => {
            return Err( Error::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "Index not found on timer list.")) )
        },
    }
    Ok(timer.clone())
}

// Setup a command intended to run once for every window created
#[tauri::command]
async fn setup(
    window: tauri::Window,
    // system_clock_mutex: State<'_, Arc<Mutex<SystemClock>>>,
    system_state_mutex: State<'_, Arc<Mutex<SystemState>>>,
    ) -> Result<(), Error> {
    println!("Setting up listeners");
    // Arc the value so we can pass it to the new thread 
    let state = Arc::clone(&system_state_mutex);
    // let clock = Arc::clone(&system_clock_mutex);
    // Spawn new thread
    std::thread::spawn(move || -> Result<(), Error> {
        // Infinite loop
        loop {
            // Sincronize the state once per second
            std::thread::sleep(std::time::Duration::from_millis(37));
            // Emit an event payload with the SystemState as its payload
            match window.emit("system_state_update",state.lock()? .clone()) {
                Ok(_) => {},
                Err(e) => {println!("Error on emit: \n{e}");},
            };
        }
    });
    Ok(())
}

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

fn main() -> Result<(), tauri::Error> {
    // Create the system state
    let system_state = Arc::new(Mutex::new(SystemState::default()));
    let mut temp_clock = SystemClock::default();
    // State struct that contains a vector of TauriTimers to
    let timer_list = Arc::new(Mutex::new(TimerList::default()));
    temp_clock.fill_timer();
    let system_clock = Arc::new(Mutex::new(SystemClock::default()));
    // Create a ney reference to it
    let arced_state = Arc::clone(&system_state);
    let arced_clock = Arc::clone(&system_clock);
    // Pass the reference into a new background thread that increments the power by 1 once per
    // second just to see that is doing something
    std::thread::spawn(move || -> Result<(), Error> {
        loop {
            std::thread::sleep(std::time::Duration::from_millis(1));
            let mut clock = arced_clock.lock()?;
            clock.clock.corner_clock_call();
            clock.fill_timer();
        }
    });
    std::thread::spawn(move || -> Result<(), Error> {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
            let mut state = arced_state.lock()?;
            let mut rand = NoiseRngU64::new(127);
            let rand = rand.get_rng_with_limit(100) as i32;
            state.power = (state.power + rand) % 101; 
        }
    });
    tauri::Builder::default()
        // Manage the AuthState
        .manage(Mutex::new(AuthState::default()))
        // Managed our arced SystemState
        .manage(system_state)
        .manage(system_clock)
        .manage(timer_list)
        // Here goes the tauri commands
        .invoke_handler(tauri::generate_handler![
                        greet,
                        login,
                        logout,
                        setup,
                        get_login,
                        get_power,
                        reset_power,
                        get_clock,
                        get_timer_list,
                        push_timer,
                        get_timer_by_index,
        ])
        // Run the app :3
        .run(tauri::generate_context!())
}
