use chrono::Local;

pub fn get_current_date() -> String {
    let now = Local::now();
    format!("{} {}", now.format("%A"), now.format("%x"))
}

pub fn get_current_time() -> String {
    Local::now().format("%H:%M:%S").to_string()
}