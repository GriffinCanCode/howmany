use crate::core::types::{CodeStats, FileStats};

#[derive(Debug)]
pub struct TimeEstimates {
    pub total_time: String,
    pub code_time: String,
    pub doc_time: String,
    pub comment_time: String,
}

pub struct TimeCalculator;

impl TimeCalculator {
    pub fn new() -> Self {
        Self
    }
    
    pub fn calculate_time_estimates(&self, stats: &CodeStats) -> TimeEstimates {
        // Realistic estimates: 1 line of code = 0.2 minutes, 1 line of docs = 0.5 minutes, 1 line of comments = 0.1 minutes
        let code_minutes = (stats.total_code_lines as f64 * 0.2) as usize;
        let doc_minutes = (stats.total_doc_lines as f64 * 0.5) as usize;
        let comment_minutes = (stats.total_comment_lines as f64 * 0.1) as usize;
        let total_minutes = code_minutes + doc_minutes + comment_minutes;
        
        TimeEstimates {
            total_time: self.format_time_human(total_minutes),
            code_time: self.format_time_human(code_minutes),
            doc_time: self.format_time_human(doc_minutes),
            comment_time: self.format_time_human(comment_minutes),
        }
    }
    
    pub fn calculate_file_type_time(&self, file_stats: &FileStats) -> String {
        let total_minutes = (file_stats.code_lines as f64 * 0.2) as usize + 
                           (file_stats.doc_lines as f64 * 0.5) as usize + 
                           (file_stats.comment_lines as f64 * 0.1) as usize;
        self.format_time_human(total_minutes)
    }
    
    pub fn format_time_human(&self, minutes: usize) -> String {
        if minutes < 60 {
            format!("{}min", minutes)
        } else if minutes < 1440 { // Less than a day
            let hours = minutes / 60;
            let remaining_minutes = minutes % 60;
            if remaining_minutes == 0 {
                format!("{}h", hours)
            } else {
                format!("{}h {}min", hours, remaining_minutes)
            }
        } else {
            let days = minutes / 1440;
            let remaining_hours = (minutes % 1440) / 60;
            if remaining_hours == 0 {
                format!("{} day{}", days, if days == 1 { "" } else { "s" })
            } else {
                format!("{} day{} {}h", days, if days == 1 { "" } else { "s" }, remaining_hours)
            }
        }
    }
} 