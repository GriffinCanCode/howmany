use crate::core::types::{CodeStats, FileStats};
use crate::core::stats::aggregation::AggregatedStats;
use crate::utils::errors::Result;
use super::time_utils::TimeCalculator;
use super::insights::InsightsGenerator;
use super::templates::TemplateGenerator;

pub struct TimeWastedReportGenerator {
    time_calculator: TimeCalculator,
    insights_generator: InsightsGenerator,
    template_generator: TemplateGenerator,
}

impl TimeWastedReportGenerator {
    pub fn new() -> Self {
        Self {
            time_calculator: TimeCalculator::new(),
            insights_generator: InsightsGenerator::new(),
            template_generator: TemplateGenerator::new(),
        }
    }
    
    pub fn create_time_wasted_content(&self, stats: &CodeStats, individual_files: &[(String, FileStats)]) -> Result<String> {
        let time_estimates = self.time_calculator.calculate_time_wasted(stats);
        let funny_insights = self.insights_generator.generate_funny_insights(stats, individual_files);
        
        let html = format!(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>⏰ TIME WASTED - A Brutally Honest Code Analysis</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
    <style>
        body {{ 
            font-family: 'SF Pro Display', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            margin: 0; 
            padding: 0; 
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: #333;
            line-height: 1.6;
        }}
        .container {{ 
            max-width: 1200px; 
            margin: 0 auto; 
            background: white; 
            min-height: 100vh;
            box-shadow: 0 0 50px rgba(0,0,0,0.3);
        }}
        .header {{
            background: linear-gradient(135deg, #ff6b6b 0%, #feca57 100%);
            color: white;
            padding: 40px 30px;
            text-align: center;
            position: relative;
            overflow: hidden;
        }}
        .header::before {{
            content: '';
            position: absolute;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background: url('data:image/svg+xml,<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100"><text y="50" font-size="20" fill="rgba(255,255,255,0.1)">⏰💻😴</text></svg>') repeat;
            opacity: 0.3;
        }}
        .header h1 {{ 
            font-size: 3.5em; 
            margin: 0; 
            text-shadow: 2px 2px 4px rgba(0,0,0,0.3);
            position: relative;
            z-index: 1;
        }}
        .header .subtitle {{ 
            font-size: 1.2em; 
            margin-top: 10px; 
            opacity: 0.9;
            position: relative;
            z-index: 1;
        }}
        .content {{ padding: 30px; }}
        .time-grid {{ 
            display: grid; 
            grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); 
            gap: 25px; 
            margin: 30px 0; 
        }}
        .time-card {{ 
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); 
            color: white; 
            padding: 25px; 
            border-radius: 15px; 
            text-align: center; 
            box-shadow: 0 8px 25px rgba(0,0,0,0.15);
            transition: transform 0.3s ease;
        }}
        .time-card:hover {{ transform: translateY(-5px); }}
        .time-value {{ 
            font-size: 2.8em; 
            font-weight: bold; 
            margin: 15px 0; 
            text-shadow: 1px 1px 2px rgba(0,0,0,0.3);
        }}
        .time-label {{ 
            font-size: 1em; 
            opacity: 0.9; 
            margin-bottom: 10px;
        }}
        .time-sublabel {{ 
            font-size: 0.85em; 
            opacity: 0.7; 
            font-style: italic;
        }}
        .section {{ 
            background: #f8f9fa; 
            margin: 30px 0; 
            padding: 25px; 
            border-radius: 12px; 
            border-left: 5px solid #667eea;
        }}
        .section h2 {{ 
            color: #2c3e50; 
            margin-top: 0; 
            font-size: 1.8em;
            display: flex;
            align-items: center;
            gap: 10px;
        }}
        .insight {{ 
            background: white; 
            padding: 20px; 
            margin: 15px 0; 
            border-radius: 8px; 
            border-left: 4px solid #feca57;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
        }}
        .insight-title {{ 
            font-weight: bold; 
            color: #2c3e50; 
            margin-bottom: 8px;
            font-size: 1.1em;
        }}
        .chart-container {{ 
            background: white; 
            padding: 20px; 
            border-radius: 12px; 
            margin: 20px 0; 
            box-shadow: 0 4px 15px rgba(0,0,0,0.1);
        }}
        .chart-title {{ 
            text-align: center; 
            margin-bottom: 20px; 
            color: #2c3e50; 
            font-size: 1.4em;
            font-weight: bold;
        }}
        .stats-table {{ 
            width: 100%; 
            border-collapse: collapse; 
            margin: 20px 0; 
            background: white;
            border-radius: 8px;
            overflow: hidden;
            box-shadow: 0 4px 15px rgba(0,0,0,0.1);
        }}
        .stats-table th {{ 
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); 
            color: white; 
            padding: 15px; 
            text-align: left; 
            font-weight: 600;
        }}
        .stats-table td {{ 
            padding: 12px 15px; 
            border-bottom: 1px solid #eee; 
        }}
        .stats-table tr:hover {{ 
            background: #f8f9fa; 
        }}
        .waste-badge {{ 
            padding: 6px 12px; 
            border-radius: 20px; 
            font-size: 0.85em; 
            font-weight: bold; 
            display: inline-block;
        }}
        .waste-low {{ background: #d4edda; color: #155724; }}
        .waste-medium {{ background: #fff3cd; color: #856404; }}
        .waste-high {{ background: #f8d7da; color: #721c24; }}
        .waste-extreme {{ background: #f5c6cb; color: #721c24; animation: pulse 2s infinite; }}
        @keyframes pulse {{
            0%, 100% {{ opacity: 1; }}
            50% {{ opacity: 0.7; }}
        }}
        .footer {{
            background: #2c3e50;
            color: white;
            padding: 30px;
            text-align: center;
            margin-top: 50px;
        }}
        .footer p {{
            margin: 5px 0;
            opacity: 0.8;
        }}
        .emoji {{ font-size: 1.2em; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>⏰ TIME WASTED</h1>
            <div class="subtitle">A brutally honest analysis of your coding adventures</div>
        </div>
        
        <div class="content">
            <div class="time-grid">
                <div class="time-card">
                    <div class="time-label">Total Time Wasted</div>
                    <div class="time-value">{}</div>
                    <div class="time-sublabel">Could've learned a new language</div>
                </div>
                <div class="time-card">
                    <div class="time-label">Writing Documentation</div>
                    <div class="time-value">{}</div>
                    <div class="time-sublabel">Nobody reads it anyway</div>
                </div>
                <div class="time-card">
                    <div class="time-label">Actual Coding</div>
                    <div class="time-value">{}</div>
                    <div class="time-sublabel">The fun part!</div>
                </div>
                <div class="time-card">
                    <div class="time-label">Writing Comments</div>
                    <div class="time-value">{}</div>
                    <div class="time-sublabel">Future you will thank you</div>
                </div>
            </div>
            
            {}
            
            <div class="section">
                <h2><span class="emoji">📊</span> The Damage by File Type</h2>
                <div class="chart-container">
                    <div class="chart-title">Time Wasted Distribution</div>
                    <canvas id="wasteChart" width="400" height="200"></canvas>
                </div>
                <table class="stats-table">
                    <thead>
                        <tr>
                            <th>File Type</th>
                            <th>Files</th>
                            <th>Time Wasted</th>
                            <th>Regret Level</th>
                            <th>Could've Been</th>
                        </tr>
                    </thead>
                    <tbody>
                        {}
                    </tbody>
                </table>
            </div>
            
            {}
        </div>
        
        <div class="footer">
            <p><strong>Remember:</strong> Time you enjoyed wasting is not wasted time!</p>
            <p>Generated by HowMany - Making developers feel productive since 2024</p>
        </div>
        
        <script>
            const ctx = document.getElementById('wasteChart').getContext('2d');
            new Chart(ctx, {{
                type: 'doughnut',
                data: {{
                    labels: [{}],
                    datasets: [{{
                        data: [{}],
                        backgroundColor: [
                            '#ff6b6b', '#feca57', '#48dbfb', '#ff9ff3', 
                            '#54a0ff', '#5f27cd', '#00d2d3', '#ff9f43',
                            '#10ac84', '#ee5a24', '#0984e3', '#a29bfe'
                        ],
                        borderWidth: 3,
                        borderColor: '#fff'
                    }}]
                }},
                options: {{
                    responsive: true,
                    maintainAspectRatio: false,
                    plugins: {{
                        legend: {{ 
                            position: 'bottom',
                            labels: {{
                                padding: 20,
                                usePointStyle: true
                            }}
                        }}
                    }}
                }}
            }});
        </script>
    </div>
</body>
</html>
"#,
            time_estimates.total_time,
            time_estimates.doc_time,
            time_estimates.code_time,
            time_estimates.comment_time,
            funny_insights,
            self.template_generator.generate_waste_table_rows(stats),
            self.template_generator.generate_individual_files_section(individual_files),
            self.template_generator.generate_complexity_labels(stats),
            self.template_generator.generate_complexity_data(stats)
        );
        
        Ok(html)
    }
    
    pub fn create_comprehensive_time_wasted_content(&self, aggregated_stats: &AggregatedStats, individual_files: &[(String, FileStats)]) -> Result<String> {
        // Convert AggregatedStats to CodeStats for compatibility with existing methods
        let code_stats = self.convert_to_code_stats(aggregated_stats);
        let time_estimates = self.time_calculator.calculate_time_wasted(&code_stats);
        let funny_insights = self.insights_generator.generate_funny_insights(&code_stats, individual_files);
        
        let html = format!(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>⏰ TIME WASTED - A Brutally Honest Code Analysis</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
    <style>
        body {{ 
            font-family: 'SF Pro Display', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            margin: 0; 
            padding: 0; 
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: #333;
            line-height: 1.6;
        }}
        .container {{ 
            max-width: 1200px; 
            margin: 0 auto; 
            background: white; 
            min-height: 100vh;
            box-shadow: 0 0 50px rgba(0,0,0,0.3);
        }}
        .header {{
            background: linear-gradient(135deg, #ff6b6b 0%, #ee5a24 100%);
            color: white;
            padding: 40px 20px;
            text-align: center;
            position: relative;
            overflow: hidden;
        }}
        .header::before {{
            content: '';
            position: absolute;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background: url('data:image/svg+xml,<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100"><text y="50" font-size="20" opacity="0.1">⏰</text></svg>') repeat;
            animation: float 20s infinite linear;
        }}
        @keyframes float {{
            0% {{ transform: translateX(0) translateY(0); }}
            100% {{ transform: translateX(-100px) translateY(-100px); }}
        }}
        .header h1 {{
            margin: 0;
            font-size: 3em;
            font-weight: 700;
            text-shadow: 2px 2px 4px rgba(0,0,0,0.3);
            position: relative;
            z-index: 1;
        }}
        .header p {{
            margin: 10px 0 0 0;
            font-size: 1.2em;
            opacity: 0.9;
            position: relative;
            z-index: 1;
        }}
        .content {{
            padding: 40px 20px;
        }}
        .time-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 20px;
            margin-bottom: 40px;
        }}
        .time-card {{
            background: linear-gradient(135deg, #ff9a9e 0%, #fecfef 100%);
            padding: 30px;
            border-radius: 15px;
            text-align: center;
            box-shadow: 0 8px 32px rgba(255, 107, 107, 0.2);
            transition: transform 0.3s ease;
        }}
        .time-card:hover {{
            transform: translateY(-5px);
        }}
        .time-value {{
            font-size: 2.5em;
            font-weight: bold;
            color: #2c3e50;
            margin-bottom: 10px;
            text-shadow: 1px 1px 2px rgba(0,0,0,0.1);
        }}
        .time-label {{
            color: #34495e;
            font-size: 1.1em;
            font-weight: 600;
            text-transform: uppercase;
            letter-spacing: 1px;
        }}
        .insights {{
            background: #f8f9fa;
            padding: 30px;
            border-radius: 15px;
            margin-bottom: 40px;
        }}
        .insights h2 {{
            color: #e74c3c;
            margin-top: 0;
            font-size: 2em;
            text-align: center;
        }}
        .insight-item {{
            background: white;
            padding: 20px;
            margin-bottom: 15px;
            border-radius: 10px;
            border-left: 5px solid #e74c3c;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
        }}
        .stats-table {{
            width: 100%;
            border-collapse: collapse;
            margin-top: 20px;
            background: white;
            border-radius: 10px;
            overflow: hidden;
            box-shadow: 0 4px 20px rgba(0,0,0,0.1);
        }}
        .stats-table th {{
            background: #e74c3c;
            color: white;
            padding: 15px;
            text-align: left;
            font-weight: 600;
        }}
        .stats-table td {{
            padding: 12px 15px;
            border-bottom: 1px solid #ecf0f1;
        }}
        .stats-table tr:hover {{
            background: #fff5f5;
        }}
        .chart-container {{
            background: white;
            padding: 30px;
            border-radius: 15px;
            margin-bottom: 40px;
            text-align: center;
            box-shadow: 0 4px 20px rgba(0,0,0,0.1);
        }}
        .footer {{
            background: #2c3e50;
            color: white;
            padding: 20px;
            text-align: center;
            font-size: 0.9em;
        }}
        h2 {{
            color: #e74c3c;
            border-bottom: 2px solid #e74c3c;
            padding-bottom: 10px;
            margin-top: 0;
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>⏰ TIME WASTED REPORT</h1>
            <p>A brutally honest analysis of your coding journey</p>
        </div>
        
        <div class="content">
            <div class="time-grid">
                <div class="time-card">
                    <div class="time-value">{}</div>
                    <div class="time-label">Total Time Invested</div>
                </div>
                <div class="time-card">
                    <div class="time-value">{}</div>
                    <div class="time-label">Documentation Time</div>
                </div>
                <div class="time-card">
                    <div class="time-value">{}</div>
                    <div class="time-label">Actual Coding Time</div>
                </div>
                <div class="time-card">
                    <div class="time-value">{}</div>
                    <div class="time-label">Comment Writing Time</div>
                </div>
            </div>
            
            <div class="insights">
                <h2>🎭 Brutal Insights</h2>
                {}
            </div>
            
            <h2>📊 Time Breakdown by Extension</h2>
            <table class="stats-table">
                <thead>
                    <tr>
                        <th>Extension</th>
                        <th>Files</th>
                        <th>Lines</th>
                        <th>Estimated Time</th>
                        <th>Productivity Score</th>
                    </tr>
                </thead>
                <tbody>
                    {}
                </tbody>
            </table>
            
            {}
            
            <div class="chart-container">
                <h2>📈 Time Distribution Chart</h2>
                <canvas id="timeChart" width="400" height="200"></canvas>
            </div>
        </div>
        
        <div class="footer">
            <p>Generated by HowMany v{} • Remember: Time you enjoyed wasting was not wasted time! 🎯</p>
        </div>
    </div>
    
    <script>
        const ctx = document.getElementById('timeChart').getContext('2d');
        new Chart(ctx, {{
            type: 'doughnut',
            data: {{
                labels: [{}],
                datasets: [{{
                    data: [{}],
                    backgroundColor: [
                        '#ff6b6b', '#4ecdc4', '#45b7d1', '#96ceb4', '#ffeaa7',
                        '#dda0dd', '#98d8c8', '#f7dc6f', '#bb8fce', '#85c1e9'
                    ],
                    borderColor: '#fff',
                    borderWidth: 2
                }}]
            }},
            options: {{
                responsive: true,
                plugins: {{
                    legend: {{
                        position: 'bottom'
                    }}
                }}
            }}
        }});
    </script>
</body>
</html>
"#,
            aggregated_stats.time.total_time_formatted,
            aggregated_stats.time.doc_time_formatted,
            aggregated_stats.time.code_time_formatted,
            aggregated_stats.time.comment_time_formatted,
            funny_insights,
            self.template_generator.generate_waste_table_rows(&code_stats),
            self.template_generator.generate_individual_files_section(individual_files),
            aggregated_stats.metadata.version,
            self.template_generator.generate_complexity_labels(&code_stats),
            self.template_generator.generate_complexity_data(&code_stats)
        );
        
        Ok(html)
    }
    
    fn convert_to_code_stats(&self, aggregated_stats: &AggregatedStats) -> CodeStats {
        use std::collections::HashMap;
        
        let mut stats_by_extension = HashMap::new();
        
        for (ext, ext_stats) in &aggregated_stats.basic.stats_by_extension {
            stats_by_extension.insert(ext.clone(), (ext_stats.file_count, crate::core::types::FileStats {
                total_lines: ext_stats.total_lines,
                code_lines: ext_stats.code_lines,
                comment_lines: ext_stats.comment_lines,
                blank_lines: ext_stats.blank_lines,
                file_size: ext_stats.total_size,
                doc_lines: ext_stats.doc_lines,
            }));
        }
        
        CodeStats {
            total_files: aggregated_stats.basic.total_files,
            total_lines: aggregated_stats.basic.total_lines,
            total_code_lines: aggregated_stats.basic.code_lines,
            total_comment_lines: aggregated_stats.basic.comment_lines,
            total_blank_lines: aggregated_stats.basic.blank_lines,
            total_size: aggregated_stats.basic.total_size,
            total_doc_lines: aggregated_stats.basic.doc_lines,
            stats_by_extension,
        }
    }
} 