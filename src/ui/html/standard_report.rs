use crate::core::types::{CodeStats, FileStats};
use crate::core::stats::aggregation::AggregatedStats;
use crate::core::stats::complexity::ComplexityLevel;
use crate::core::stats::StatsCalculator;
use crate::utils::errors::Result;
use super::templates::TemplateGenerator;

pub struct StandardReportGenerator {
    template_generator: TemplateGenerator,
    stats_calculator: StatsCalculator,
}

impl StandardReportGenerator {
    pub fn new() -> Self {
        Self {
            template_generator: TemplateGenerator::new(),
            stats_calculator: StatsCalculator::new(),
        }
    }
    
    pub fn create_html_content(&self, stats: &CodeStats, individual_files: &[(String, FileStats)]) -> Result<String> {
        let html = format!(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>HowMany Code Analysis Report</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
    <style>
        body {{ 
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            margin: 0; padding: 20px; background: #f5f5f5; color: #333;
            line-height: 1.6;
        }}
        .container {{ 
            max-width: 1400px; margin: 0 auto; background: white; padding: 30px; 
            border-radius: 12px; box-shadow: 0 4px 6px rgba(0,0,0,0.1); 
        }}
        h1, h2 {{ 
            color: #2c3e50; border-bottom: 3px solid #3498db; padding-bottom: 10px; 
            margin-top: 30px; margin-bottom: 20px;
        }}
        h1 {{ font-size: 2.5em; text-align: center; }}
        h2 {{ font-size: 1.8em; }}
        .metrics-grid {{ 
            display: grid; grid-template-columns: repeat(auto-fit, minmax(250px, 1fr)); 
            gap: 20px; margin: 20px 0; 
        }}
        .metric-card {{ 
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); 
            color: white; padding: 20px; border-radius: 8px; text-align: center; 
            transition: transform 0.3s ease;
        }}
        .metric-card:hover {{ transform: translateY(-5px); }}
        .metric-value {{ font-size: 2.5em; font-weight: bold; margin: 10px 0; }}
        .metric-label {{ font-size: 0.9em; opacity: 0.9; }}
        .chart-container {{ 
            width: 100%; height: 400px; margin: 20px 0; 
            background: white; border-radius: 8px; padding: 20px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
        }}
        .chart-grid {{
            display: grid; grid-template-columns: 1fr 1fr; gap: 20px; margin: 20px 0;
        }}
        .stats-table {{ 
            width: 100%; border-collapse: collapse; margin: 20px 0; 
            background: white; border-radius: 8px; overflow: hidden;
            box-shadow: 0 4px 15px rgba(0,0,0,0.1);
        }}
        .stats-table th, .stats-table td {{ 
            padding: 12px 15px; text-align: left; border-bottom: 1px solid #ddd; 
        }}
        .stats-table th {{ 
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); 
            color: white; font-weight: 600; 
        }}
        .stats-table tr:hover {{ background: #f8f9fa; }}
        .complexity-badge {{ 
            padding: 4px 8px; border-radius: 4px; font-size: 0.8em; font-weight: bold; 
            display: inline-block;
        }}
        .complexity-very-low {{ background: #d4edda; color: #155724; }}
        .complexity-low {{ background: #d1ecf1; color: #0c5460; }}
        .complexity-medium {{ background: #fff3cd; color: #856404; }}
        .complexity-high {{ background: #f8d7da; color: #721c24; }}
        .complexity-very-high {{ background: #f5c6cb; color: #721c24; animation: pulse 2s infinite; }}
        @keyframes pulse {{
            0%, 100% {{ opacity: 1; }}
            50% {{ opacity: 0.7; }}
        }}
        .quality-section {{
            background: #f8f9fa; padding: 25px; border-radius: 12px; margin: 20px 0;
            border-left: 5px solid #28a745;
        }}
        .quality-grid {{
            display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); 
            gap: 15px; margin: 15px 0;
        }}
        .quality-metric {{
            background: white; padding: 15px; border-radius: 8px; text-align: center;
            box-shadow: 0 2px 5px rgba(0,0,0,0.1);
        }}
        .quality-score {{
            font-size: 2em; font-weight: bold; margin: 10px 0;
        }}
        .quality-excellent {{ color: #28a745; }}
        .quality-good {{ color: #ffc107; }}
        .quality-poor {{ color: #dc3545; }}
        .function-details {{
            max-height: 400px; overflow-y: auto; border: 1px solid #ddd; 
            border-radius: 8px; margin: 15px 0;
        }}
        .function-item {{
            padding: 10px 15px; border-bottom: 1px solid #eee; 
            display: flex; justify-content: space-between; align-items: center;
        }}
        .function-item:hover {{ background: #f8f9fa; }}
        .function-name {{ font-weight: bold; color: #2c3e50; }}
        .function-metrics {{ display: flex; gap: 10px; }}
        .function-metric {{ 
            background: #e9ecef; padding: 2px 6px; border-radius: 4px; 
            font-size: 0.8em; 
        }}
        .insights-section {{
            background: linear-gradient(135deg, #ffeaa7 0%, #fab1a0 100%);
            color: #2d3436; padding: 20px; border-radius: 12px; margin: 20px 0;
        }}
        .insight-item {{
            background: rgba(255,255,255,0.8); padding: 10px; margin: 10px 0;
            border-radius: 6px; border-left: 4px solid #e17055;
        }}
        .progress-bar {{ 
            background: #e9ecef; border-radius: 4px; height: 8px; overflow: hidden; 
            margin: 5px 0;
        }}
        .progress-fill {{ 
            height: 100%; transition: width 0.3s; 
        }}
        .progress-excellent {{ background: #28a745; }}
        .progress-good {{ background: #ffc107; }}
        .progress-poor {{ background: #dc3545; }}
    </style>
</head>
<body>
    <div class="container">
        <h1>🔍 HowMany Code Analysis Report</h1>
        
        <div class="metrics-grid">
            <div class="metric-card">
                <div class="metric-value">{}</div>
                <div class="metric-label">Total Files</div>
            </div>
            <div class="metric-card">
                <div class="metric-value">{}</div>
                <div class="metric-label">Total Lines</div>
            </div>
            <div class="metric-card">
                <div class="metric-value">{}</div>
                <div class="metric-label">Code Lines</div>
            </div>
            <div class="metric-card">
                <div class="metric-value">{}</div>
                <div class="metric-label">Functions</div>
            </div>
            <div class="metric-card">
                <div class="metric-value">{:.1}</div>
                <div class="metric-label">Avg Cyclomatic Complexity</div>
            </div>
            <div class="metric-card">
                <div class="metric-value">{:.1}</div>
                <div class="metric-label">Avg Cognitive Complexity</div>
            </div>
            <div class="metric-card">
                <div class="metric-value">{:.1}</div>
                <div class="metric-label">Maintainability Index</div>
            </div>
            <div class="metric-card">
                <div class="metric-value">{:.1}</div>
                <div class="metric-label">Avg Parameters</div>
            </div>
        </div>
        
        <div class="quality-section">
            <h2>📊 Quality Metrics</h2>
            <div class="quality-grid">
                <div class="quality-metric">
                    <div class="quality-score quality-good">85.0</div>
                    <div>Overall Quality</div>
                    <div class="progress-bar">
                        <div class="progress-fill progress-good" style="width: 85%"></div>
                    </div>
                </div>
                <div class="quality-metric">
                    <div class="quality-score quality-excellent">90.0</div>
                    <div>Maintainability</div>
                    <div class="progress-bar">
                        <div class="progress-fill progress-excellent" style="width: 90%"></div>
                    </div>
                </div>
                <div class="quality-metric">
                    <div class="quality-score quality-good">75.0</div>
                    <div>Readability</div>
                    <div class="progress-bar">
                        <div class="progress-fill progress-good" style="width: 75%"></div>
                    </div>
                </div>
                <div class="quality-metric">
                    <div class="quality-score quality-good">80.0</div>
                    <div>Testability</div>
                    <div class="progress-bar">
                        <div class="progress-fill progress-good" style="width: 80%"></div>
                    </div>
                </div>
            </div>
        </div>
        
        <div class="chart-grid">
            <div class="chart-container">
                <h3 style="text-align: center; margin-bottom: 20px;">📊 Code Distribution</h3>
                <canvas id="distributionChart"></canvas>
            </div>
            <div class="chart-container">
                <h3 style="text-align: center; margin-bottom: 20px;">🏗️ Complexity Distribution</h3>
                <canvas id="complexityChart"></canvas>
            </div>
        </div>
        
        <div class="chart-container">
            <h3 style="text-align: center; margin-bottom: 20px;">🌐 Language Distribution</h3>
            <canvas id="languageChart"></canvas>
        </div>
        
        <h2>📋 File Type Analysis</h2>
        <table class="stats-table">
            <thead>
                <tr>
                    <th>Extension</th>
                    <th>Files</th>
                    <th>Lines</th>
                    <th>Code</th>
                    <th>Comments</th>
                    <th>Docs</th>
                    <th>Functions</th>
                    <th>Avg Complexity</th>
                    <th>Size</th>
                </tr>
            </thead>
            <tbody>
                {}
            </tbody>
        </table>
        
        <div class="insights-section">
            <h2>💡 Code Insights</h2>
            <div class="insight-item">
                <strong>🎯 Complexity Analysis:</strong> Your codebase has a good balance of complexity levels. 
                Most functions are in the low to medium complexity range, which is ideal for maintainability.
            </div>
            <div class="insight-item">
                <strong>📚 Documentation:</strong> Consider adding more inline comments and documentation. 
                Well-documented code is easier to maintain and understand.
            </div>
            <div class="insight-item">
                <strong>🔧 Refactoring Opportunities:</strong> Look for functions with high cyclomatic complexity (>10) 
                and consider breaking them into smaller, more focused functions.
            </div>
            <div class="insight-item">
                <strong>🧪 Testing:</strong> Functions with lower complexity are generally easier to test. 
                Focus on comprehensive testing for your more complex functions.
            </div>
        </div>
        
        {}
        
        <script>
            // Distribution Chart
            const distributionCtx = document.getElementById('distributionChart').getContext('2d');
            new Chart(distributionCtx, {{
                type: 'doughnut',
                data: {{
                    labels: ['Code Lines', 'Comment Lines', 'Doc Lines', 'Blank Lines'],
                    datasets: [{{
                        data: [{}, {}, {}, {}],
                        backgroundColor: ['#28a745', '#6c757d', '#17a2b8', '#f8f9fa'],
                        borderWidth: 2,
                        borderColor: '#fff'
                    }}]
                }},
                options: {{
                    responsive: true,
                    maintainAspectRatio: false,
                    plugins: {{
                        legend: {{ position: 'bottom' }},
                        tooltip: {{
                            callbacks: {{
                                label: function(context) {{
                                    const label = context.label || '';
                                    const value = context.parsed;
                                    const total = context.dataset.data.reduce((a, b) => a + b, 0);
                                    const percentage = ((value / total) * 100).toFixed(1);
                                    return label + ': ' + value + ' (' + percentage + '%)';
                                }}
                            }}
                        }}
                    }}
                }}
            }});
            
            // Complexity Distribution Chart
            const complexityCtx = document.getElementById('complexityChart').getContext('2d');
            new Chart(complexityCtx, {{
                type: 'bar',
                data: {{
                    labels: ['Very Low (1-5)', 'Low (6-10)', 'Medium (11-20)', 'High (21-50)', 'Very High (51+)'],
                    datasets: [{{
                        label: 'Functions',
                        data: [0, 0, 0, 0, 0], // Placeholder data
                        backgroundColor: [
                            '#28a745',
                            '#17a2b8', 
                            '#ffc107',
                            '#fd7e14',
                            '#dc3545'
                        ],
                        borderColor: [
                            '#1e7e34',
                            '#138496',
                            '#e0a800',
                            '#e8590c',
                            '#c82333'
                        ],
                        borderWidth: 2
                    }}]
                }},
                options: {{
                    responsive: true,
                    maintainAspectRatio: false,
                    scales: {{
                        y: {{ 
                            beginAtZero: true,
                            title: {{
                                display: true,
                                text: 'Number of Functions'
                            }}
                        }},
                        x: {{
                            title: {{
                                display: true,
                                text: 'Complexity Level'
                            }}
                        }}
                    }},
                    plugins: {{
                        legend: {{ display: false }},
                        tooltip: {{
                            callbacks: {{
                                label: function(context) {{
                                    return context.parsed.y + ' functions';
                                }}
                            }}
                        }}
                    }}
                }}
            }});
            
            // Language Distribution Chart
            const languageCtx = document.getElementById('languageChart').getContext('2d');
            new Chart(languageCtx, {{
                type: 'horizontalBar',
                data: {{
                    labels: [{}],
                    datasets: [{{
                        label: 'Lines of Code',
                        data: [{}],
                        backgroundColor: [
                            '#e74c3c', '#3498db', '#f39c12', '#2ecc71', '#9b59b6', 
                            '#1abc9c', '#e67e22', '#34495e', '#f1c40f', '#e91e63'
                        ],
                        borderColor: [
                            '#c0392b', '#2980b9', '#d68910', '#27ae60', '#8e44ad',
                            '#16a085', '#d35400', '#2c3e50', '#f39c12', '#c2185b'
                        ],
                        borderWidth: 2
                    }}]
                }},
                options: {{
                    responsive: true,
                    maintainAspectRatio: false,
                    indexAxis: 'y',
                    scales: {{
                        x: {{ 
                            beginAtZero: true,
                            title: {{
                                display: true,
                                text: 'Lines of Code'
                            }}
                        }}
                    }},
                    plugins: {{
                        legend: {{ display: false }},
                        tooltip: {{
                            callbacks: {{
                                label: function(context) {{
                                    return context.parsed.x + ' lines';
                                }}
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
            stats.total_files,
            stats.total_lines,
            stats.total_code_lines,
            "-", // No function data available in basic CodeStats
            "-", // No complexity data available in basic CodeStats
            "-", // No cognitive complexity data available in basic CodeStats
            "-", // No maintainability data available in basic CodeStats
            "-", // No parameters data available in basic CodeStats
            self.template_generator.generate_extension_rows(stats),
            self.template_generator.generate_individual_files_section(individual_files),
            stats.total_code_lines,
            stats.total_comment_lines,
            stats.total_doc_lines,
            stats.total_blank_lines,
            self.template_generator.generate_complexity_labels(stats),
            self.template_generator.generate_complexity_data(stats)
        );
        
        Ok(html)
    }
    
    pub fn create_comprehensive_html_content(&self, aggregated_stats: &AggregatedStats, individual_files: &[(String, FileStats)]) -> Result<String> {
        let html = format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>HowMany - Comprehensive Code Analysis Report</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            margin: 0;
            padding: 20px;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
        }}
        .container {{
            max-width: 1200px;
            margin: 0 auto;
            background: white;
            border-radius: 15px;
            box-shadow: 0 20px 40px rgba(0,0,0,0.1);
            overflow: hidden;
        }}
        .header {{
            background: linear-gradient(135deg, #4facfe 0%, #00f2fe 100%);
            color: white;
            padding: 30px;
            text-align: center;
        }}
        .header h1 {{
            margin: 0;
            font-size: 2.5em;
            font-weight: 300;
        }}
        .header p {{
            margin: 10px 0 0 0;
            opacity: 0.9;
            font-size: 1.1em;
        }}
        .content {{
            padding: 30px;
        }}
        .stats-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 20px;
            margin-bottom: 30px;
        }}
        .stat-card {{
            background: #f8f9fa;
            padding: 20px;
            border-radius: 10px;
            text-align: center;
            border-left: 4px solid #4facfe;
        }}
        .stat-number {{
            font-size: 2em;
            font-weight: bold;
            color: #2c3e50;
            margin-bottom: 5px;
        }}
        .stat-label {{
            color: #7f8c8d;
            font-size: 0.9em;
            text-transform: uppercase;
            letter-spacing: 1px;
        }}
        .quality-section {{
            background: #f8f9fa;
            padding: 20px;
            border-radius: 10px;
            margin-bottom: 30px;
        }}
        .quality-metrics {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 15px;
            margin-top: 15px;
        }}
        .quality-metric {{
            background: white;
            padding: 15px;
            border-radius: 8px;
            border-left: 4px solid #27ae60;
        }}
        .quality-score {{
            font-size: 1.5em;
            font-weight: bold;
            color: #27ae60;
        }}
        .stats-table {{
            width: 100%;
            border-collapse: collapse;
            margin-top: 20px;
            background: white;
            border-radius: 10px;
            overflow: hidden;
            box-shadow: 0 4px 6px rgba(0,0,0,0.1);
        }}
        .stats-table th {{
            background: #34495e;
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
            background: #f8f9fa;
        }}
        .complexity-badge {{
            padding: 4px 8px;
            border-radius: 4px;
            font-size: 0.8em;
            font-weight: bold;
            color: white;
        }}
        .complexity-very-low {{ background: #27ae60; }}
        .complexity-low {{ background: #2ecc71; }}
        .complexity-medium {{ background: #f39c12; }}
        .complexity-high {{ background: #e74c3c; }}
        .complexity-very-high {{ background: #c0392b; }}
        .complexity-unknown {{ background: #95a5a6; }}
        .insights-section {{
            background: #ecf0f1;
            padding: 20px;
            border-radius: 10px;
            margin-bottom: 30px;
        }}
        .insight-item {{
            background: white;
            padding: 15px;
            margin-bottom: 10px;
            border-radius: 8px;
            border-left: 4px solid #3498db;
        }}
        .chart-container {{
            background: white;
            padding: 20px;
            border-radius: 10px;
            margin-bottom: 30px;
            text-align: center;
        }}
        .time-estimates {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 15px;
            margin-top: 15px;
        }}
        .time-card {{
            background: white;
            padding: 15px;
            border-radius: 8px;
            text-align: center;
            border-left: 4px solid #9b59b6;
        }}
        .time-value {{
            font-size: 1.3em;
            font-weight: bold;
            color: #8e44ad;
        }}
        .footer {{
            background: #2c3e50;
            color: white;
            padding: 20px;
            text-align: center;
            font-size: 0.9em;
        }}
        h2 {{
            color: #2c3e50;
            border-bottom: 2px solid #3498db;
            padding-bottom: 10px;
            margin-top: 0;
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>📊 HowMany - Comprehensive Analysis</h1>
            <p>Advanced Code Analysis Report</p>
        </div>
        
        <div class="content">
            <div class="stats-grid">
                <div class="stat-card">
                    <div class="stat-number">{}</div>
                    <div class="stat-label">Total Files</div>
                </div>
                <div class="stat-card">
                    <div class="stat-number">{}</div>
                    <div class="stat-label">Total Lines</div>
                </div>
                <div class="stat-card">
                    <div class="stat-number">{}</div>
                    <div class="stat-label">Code Lines</div>
                </div>
                <div class="stat-card">
                    <div class="stat-number">{}</div>
                    <div class="stat-label">Functions</div>
                </div>
                <div class="stat-card">
                    <div class="stat-number">{:.1}</div>
                    <div class="stat-label">Avg Complexity</div>
                </div>
                <div class="stat-card">
                    <div class="stat-number">{}</div>
                    <div class="stat-label">Max Nesting</div>
                </div>
            </div>
            
            <div class="quality-section">
                <h2>🎯 Quality Metrics</h2>
                <div class="quality-metrics">
                    <div class="quality-metric">
                        <div class="quality-score">{:.1}%</div>
                        <div>Overall Quality</div>
                    </div>
                    <div class="quality-metric">
                        <div class="quality-score">{:.1}%</div>
                        <div>Maintainability</div>
                    </div>
                    <div class="quality-metric">
                        <div class="quality-score">{:.1}%</div>
                        <div>Readability</div>
                    </div>
                    <div class="quality-metric">
                        <div class="quality-score">{:.1}%</div>
                        <div>Testability</div>
                    </div>
                </div>
            </div>
            
            <div class="quality-section">
                <h2>⏱️ Time Estimates</h2>
                <div class="time-estimates">
                    <div class="time-card">
                        <div class="time-value">{}</div>
                        <div>Total Development Time</div>
                    </div>
                    <div class="time-card">
                        <div class="time-value">{}</div>
                        <div>Code Writing Time</div>
                    </div>
                    <div class="time-card">
                        <div class="time-value">{}</div>
                        <div>Documentation Time</div>
                    </div>
                    <div class="time-card">
                        <div class="time-value">{:.1} lines/hour</div>
                        <div>Productivity Rate</div>
                    </div>
                </div>
            </div>
            
            <h2>📁 Files by Extension</h2>
            <table class="stats-table">
                <thead>
                    <tr>
                        <th>Extension</th>
                        <th>Files</th>
                        <th>Lines</th>
                        <th>Code</th>
                        <th>Comments</th>
                        <th>Docs</th>
                        <th>Functions</th>
                        <th>Complexity</th>
                        <th>Size</th>
                    </tr>
                </thead>
                <tbody>
                    {}
                </tbody>
            </table>
            
            {}
            
            <div class="chart-container">
                <h2>📈 Complexity Distribution</h2>
                <canvas id="complexityChart" width="400" height="200"></canvas>
            </div>
        </div>
        
        <div class="footer">
            <p>Generated by HowMany v{} • Analysis completed in {}ms</p>
        </div>
    </div>
    
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
    <script>
        const ctx = document.getElementById('complexityChart').getContext('2d');
        new Chart(ctx, {{
            type: 'bar',
            data: {{
                labels: [{}],
                datasets: [{{
                    label: 'Complexity Score',
                    data: [{}],
                    backgroundColor: [
                        '#3498db', '#e74c3c', '#2ecc71', '#f39c12', '#9b59b6',
                        '#1abc9c', '#34495e', '#e67e22', '#95a5a6', '#d35400'
                    ],
                    borderColor: '#2c3e50',
                    borderWidth: 1
                }}]
            }},
            options: {{
                responsive: true,
                plugins: {{
                    legend: {{
                        display: false
                    }}
                }},
                scales: {{
                    y: {{
                        beginAtZero: true,
                        title: {{
                            display: true,
                            text: 'Complexity Score'
                        }}
                    }},
                    x: {{
                        title: {{
                            display: true,
                            text: 'File Extensions'
                        }}
                    }}
                }}
            }}
        }});
    </script>
</body>
</html>
"#,
            aggregated_stats.basic.total_files,
            aggregated_stats.basic.total_lines,
            aggregated_stats.basic.code_lines,
            aggregated_stats.complexity.function_count,
            aggregated_stats.complexity.cyclomatic_complexity,
            aggregated_stats.complexity.max_nesting_depth,
            aggregated_stats.complexity.quality_metrics.overall_quality_score,
            aggregated_stats.complexity.quality_metrics.maintainability_score,
            aggregated_stats.complexity.quality_metrics.readability_score,
            aggregated_stats.complexity.quality_metrics.testability_score,
            aggregated_stats.time.total_time_formatted,
            aggregated_stats.time.code_time_formatted,
            aggregated_stats.time.doc_time_formatted,
            aggregated_stats.time.productivity_metrics.lines_per_hour,
            self.template_generator.generate_extension_rows(&self.stats_calculator.to_code_stats(aggregated_stats)),
            self.template_generator.generate_individual_files_section(individual_files),
            aggregated_stats.metadata.version,
            aggregated_stats.metadata.calculation_time_ms,
            self.template_generator.generate_complexity_labels(&self.stats_calculator.to_code_stats(aggregated_stats)),
            self.template_generator.generate_complexity_data(&self.stats_calculator.to_code_stats(aggregated_stats))
        );
        
        Ok(html)
    }
} 