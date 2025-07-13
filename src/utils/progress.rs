use indicatif::{ProgressBar, ProgressStyle, MultiProgress};
use std::time::Duration;

pub struct ProgressReporter {
    multi_progress: MultiProgress,
    main_progress: ProgressBar,
}

impl ProgressReporter {
    pub fn new() -> Self {
        let multi_progress = MultiProgress::new();
        let main_progress = multi_progress.add(ProgressBar::new(0));
        
        main_progress.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} files ({eta})")
                .unwrap()
                .progress_chars("#>-")
        );
        
        Self {
            multi_progress,
            main_progress,
        }
    }
    
    pub fn set_total_files(&self, total: u64) {
        self.main_progress.set_length(total);
        self.main_progress.set_message("Analyzing files...");
    }
    
    pub fn increment(&self) {
        self.main_progress.inc(1);
    }
    
    pub fn set_message(&self, message: &str) {
        self.main_progress.set_message(message.to_string());
    }
    
    pub fn finish(&self) {
        self.main_progress.finish_with_message("Analysis complete!");
    }
    
    pub fn finish_and_clear(&self) {
        self.main_progress.finish_and_clear();
    }
    
    pub fn create_spinner(&self, message: &str) -> ProgressBar {
        let spinner = self.multi_progress.add(ProgressBar::new_spinner());
        spinner.set_style(
            ProgressStyle::default_spinner()
                .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
                .template("{spinner:.cyan} {msg}")
                .unwrap()
        );
        spinner.set_message(message.to_string());
        spinner.enable_steady_tick(Duration::from_millis(100));
        spinner
    }
    
    pub fn join(&self) {
        // MultiProgress doesn't have a join method in this version
        // The progress bars will be automatically managed
    }
}

pub struct FileProgress {
    pub current_file: String,
    pub files_processed: usize,
    pub total_files: usize,
    pub lines_processed: usize,
    pub bytes_processed: u64,
}

impl FileProgress {
    pub fn new(total_files: usize) -> Self {
        Self {
            current_file: String::new(),
            files_processed: 0,
            total_files,
            lines_processed: 0,
            bytes_processed: 0,
        }
    }
    
    pub fn update_file(&mut self, file_path: &str) {
        self.current_file = file_path.to_string();
        self.files_processed += 1;
    }
    
    pub fn add_lines(&mut self, lines: usize) {
        self.lines_processed += lines;
    }
    
    pub fn add_bytes(&mut self, bytes: u64) {
        self.bytes_processed += bytes;
    }
    
    pub fn percentage(&self) -> f64 {
        if self.total_files == 0 {
            100.0
        } else {
            (self.files_processed as f64 / self.total_files as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_file_progress() {
        let mut progress = FileProgress::new(10);
        
        assert_eq!(progress.percentage(), 0.0);
        
        progress.update_file("test.rs");
        progress.add_lines(100);
        progress.add_bytes(1000);
        
        assert_eq!(progress.files_processed, 1);
        assert_eq!(progress.lines_processed, 100);
        assert_eq!(progress.bytes_processed, 1000);
        assert_eq!(progress.percentage(), 10.0);
    }
    
    #[test]
    fn test_empty_progress() {
        let progress = FileProgress::new(0);
        assert_eq!(progress.percentage(), 100.0);
    }
} 