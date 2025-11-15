use std::time::Duration;

pub trait ToString {
    fn to_string(&self) -> String;
}

impl ToString for Duration {
    fn to_string(&self) -> String {
        let hours = self.as_secs() / 3600;
        let minutes = (self.as_secs() % 3600) / 60;
        let seconds = self.as_secs() % 60;

        let mut parts = Vec::new();

        if hours > 0 {
            parts.push(format!("{}h", hours));
        }
        if minutes > 0 {
            parts.push(format!("{}m", minutes));
        }
        if seconds > 0 || parts.is_empty() {
            parts.push(format!("{}s", seconds));
        }

        parts.join(" ")
    }
}
