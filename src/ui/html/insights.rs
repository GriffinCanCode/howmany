use crate::core::types::{CodeStats, FileStats};


pub struct InsightsGenerator;

impl InsightsGenerator {
    pub fn new() -> Self {
        Self
    }
    
    pub fn generate_funny_insights(&self, stats: &CodeStats, _individual_files: &[(String, FileStats)]) -> String {
        let mut insights = Vec::new();
        
        // Documentation insights
        if stats.total_doc_lines > stats.total_code_lines {
            insights.push(format!(
                r#"<div class="insight">
                    <div class="insight-title">📚 Documentation Overachiever</div>
                    You've written more documentation ({} lines) than actual code ({} lines). 
                    Either you're incredibly thorough or procrastinating on the hard stuff!
                </div>"#,
                stats.total_doc_lines, stats.total_code_lines
            ));
        }
        
        // Comment insights
        let comment_ratio = if stats.total_code_lines > 0 {
            (stats.total_comment_lines as f64 / stats.total_code_lines as f64) * 100.0
        } else {
            0.0
        };
        
        if comment_ratio > 50.0 {
            insights.push(format!(
                r#"<div class="insight">
                    <div class="insight-title">💬 Comment Enthusiast</div>
                    {:.1}% of your code is comments. Your future self will either love you or wonder why you explained that `i++` does.
                </div>"#,
                comment_ratio
            ));
        } else if comment_ratio < 10.0 {
            insights.push(format!(
                r#"<div class="insight">
                    <div class="insight-title">🤫 The Silent Coder</div>
                    Only {:.1}% comments? Living dangerously! Future you is going to have some questions.
                </div>"#,
                comment_ratio
            ));
        }
        
        // File type insights
        if let Some((most_files_ext, (file_count, _))) = stats.stats_by_extension.iter().max_by_key(|(_, (count, _))| *count) {
            let file_percentage = (*file_count as f64 / stats.total_files as f64) * 100.0;
            if file_percentage > 60.0 {
                insights.push(format!(
                    r#"<div class="insight">
                        <div class="insight-title">🎯 Language Loyalist</div>
                        {:.1}% of your files are .{} files. Commitment or just haven't discovered other languages yet?
                    </div>"#,
                    file_percentage, most_files_ext
                ));
            }
        }
        
        // TypeScript detection
        if stats.stats_by_extension.contains_key("ts") || stats.stats_by_extension.contains_key("tsx") {
            let ts_stats = stats.stats_by_extension.get("ts").map(|(_, stats)| stats.total_lines).unwrap_or(0);
            let tsx_stats = stats.stats_by_extension.get("tsx").map(|(_, stats)| stats.total_lines).unwrap_or(0);
            let total_ts = ts_stats + tsx_stats;
            
            if total_ts > 1000 {
                insights.push(format!(
                    r#"<div class="insight">
                        <div class="insight-title">⚡ TypeScript Warrior</div>
                        {} lines of TypeScript! You've embraced the type safety life. Your JavaScript days are behind you.
                    </div>"#,
                    total_ts
                ));
            }
        }
        
        // Blank line insights
        let blank_ratio = if stats.total_lines > 0 {
            (stats.total_blank_lines as f64 / stats.total_lines as f64) * 100.0
        } else {
            0.0
        };
        
        if blank_ratio > 30.0 {
            insights.push(format!(
                r#"<div class="insight">
                    <div class="insight-title">🌬️ The Breathing Room Expert</div>
                    {:.1}% of your files are blank lines. You really believe in giving your code room to breathe!
                </div>"#,
                blank_ratio
            ));
        }
        
        // File size insights
        if stats.total_files > 100 {
            insights.push(format!(
                r#"<div class="insight">
                    <div class="insight-title">🗂️ File Collector</div>
                    {} files! You either love organization or can't decide where to put things.
                </div>"#,
                stats.total_files
            ));
        }
        
        // Python docstring detection
        if stats.stats_by_extension.contains_key("py") {
            let py_stats = &stats.stats_by_extension.get("py").unwrap().1;
            if py_stats.doc_lines > py_stats.comment_lines * 2 {
                insights.push(format!(
                    r#"<div class="insight">
                        <div class="insight-title">🐍 Python Docstring Devotee</div>
                        Your Python code has more docstrings than comments. Following PEP 257 like a true Pythonista!
                    </div>"#
                ));
            }
        }
        
        // Rust documentation
        if stats.stats_by_extension.contains_key("rs") {
            let rs_stats = &stats.stats_by_extension.get("rs").unwrap().1;
            if rs_stats.doc_lines > 0 {
                insights.push(format!(
                    r#"<div class="insight">
                        <div class="insight-title">🦀 Rust Documentation Hero</div>
                        {} lines of Rust docs! You're making the borrow checker AND future developers happy.
                    </div>"#,
                    rs_stats.doc_lines
                ));
            }
        }
        
        if insights.is_empty() {
            insights.push(format!(
                r#"<div class="insight">
                    <div class="insight-title">🤖 The Efficient Coder</div>
                    Your code is so clean and well-balanced, our humor algorithms can't find anything to roast you about. Impressive!
                </div>"#
            ));
        }
        
        format!(
            r#"<div class="section">
                <h2><span class="emoji">🎭</span> Brutally Honest Insights</h2>
                {}
            </div>"#,
            insights.join("\n")
        )
    }
} 