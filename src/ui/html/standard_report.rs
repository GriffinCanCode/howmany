use crate::core::types::{CodeStats, FileStats};
use crate::core::stats::aggregation::AggregatedStats;

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
        // Calculate real aggregated stats for better accuracy
        let aggregated_stats = self.stats_calculator.calculate_project_stats(stats, individual_files)?;
        
        let html = format!(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>HowMany Code Analysis Report</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js@3.9.1/dist/chart.min.js"></script>
    <style>
        * {{ box-sizing: border-box; }}
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
            will-change: transform;
            transition: transform 0.2s ease;
        }}
        .metric-card:hover {{ transform: translateY(-3px); }}
        .metric-value {{ font-size: 2.5em; font-weight: bold; margin: 10px 0; }}
        .metric-label {{ font-size: 0.9em; opacity: 0.9; }}
        .chart-container {{ 
            width: 100%; height: 400px; margin: 20px 0; 
            background: white; border-radius: 8px; padding: 20px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
            position: relative;
        }}
        .chart-grid {{
            display: grid; grid-template-columns: 1fr 1fr; gap: 20px; margin: 20px 0;
        }}
        .chart-loading {{
            position: absolute;
            top: 50%;
            left: 50%;
            transform: translate(-50%, -50%);
            color: #666;
            font-size: 1.1em;
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
        .stats-table tr:nth-child(even) {{ background: #f8f9fa; }}
        .complexity-badge {{ 
            padding: 4px 8px; border-radius: 4px; font-size: 0.8em; font-weight: bold; 
            display: inline-block;
        }}
        .complexity-very-low {{ background: #d4edda; color: #155724; }}
        .complexity-low {{ background: #d1ecf1; color: #0c5460; }}
        .complexity-medium {{ background: #fff3cd; color: #856404; }}
        .complexity-high {{ background: #f8d7da; color: #721c24; }}
        .complexity-very-high {{ background: #f5c6cb; color: #721c24; }}
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
        .function-item:nth-child(even) {{ background: #f8f9fa; }}
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
            height: 100%; transition: width 0.3s ease; 
        }}
        .progress-excellent {{ background: #28a745; }}
        .progress-good {{ background: #ffc107; }}
        .progress-poor {{ background: #dc3545; }}
        .lazy-section {{
            opacity: 0;
            transition: opacity 0.3s ease;
        }}
        .lazy-section.loaded {{
            opacity: 1;
        }}
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
                    <div class="quality-score {}">{:.1}</div>
                    <div>Overall Quality</div>
                    <div class="progress-bar">
                        <div class="progress-fill {}" style="width: {:.1}%"></div>
                    </div>
                </div>
                <div class="quality-metric">
                    <div class="quality-score {}">{:.1}</div>
                    <div>Maintainability</div>
                    <div class="progress-bar">
                        <div class="progress-fill {}" style="width: {:.1}%"></div>
                    </div>
                </div>
                <div class="quality-metric">
                    <div class="quality-score {}">{:.1}</div>
                    <div>Readability</div>
                    <div class="progress-bar">
                        <div class="progress-fill {}" style="width: {:.1}%"></div>
                    </div>
                </div>
                <div class="quality-metric">
                    <div class="quality-score {}">{:.1}</div>
                    <div>Testability</div>
                    <div class="progress-bar">
                        <div class="progress-fill {}" style="width: {:.1}%"></div>
                    </div>
                </div>
            </div>
        </div>
        
        <div class="chart-grid">
            <div class="chart-container">
                <h3 style="text-align: center; margin-bottom: 20px;">📊 Code Distribution</h3>
                <div class="chart-loading">Loading chart...</div>
                <canvas id="distributionChart" style="display: none;"></canvas>
            </div>
            <div class="chart-container">
                <h3 style="text-align: center; margin-bottom: 20px;">🏗️ Complexity Distribution</h3>
                <div class="chart-loading">Loading chart...</div>
                <canvas id="complexityChart" style="display: none;"></canvas>
            </div>
        </div>
        
        <div class="chart-container">
            <h3 style="text-align: center; margin-bottom: 20px;">🌐 Language Distribution</h3>
            <div class="chart-loading">Loading chart...</div>
            <canvas id="languageChart" style="display: none;"></canvas>
        </div>
        
        <div class="lazy-section" id="fileAnalysis">
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
        </div>
        
        <div class="lazy-section" id="insights">
            <div class="insights-section">
                <h2>💡 Code Insights</h2>
                {}
            </div>
        </div>
        
        <div class="lazy-section" id="individualFiles">
            {}
        </div>
        
        <script>
            // Chart data
            const chartData = {{
                distribution: {{
                    labels: ['Code Lines', 'Comment Lines', 'Doc Lines', 'Blank Lines'],
                    data: [{}, {}, {}, {}]
                }},
                complexity: {{
                    labels: ['Very Low (1-5)', 'Low (6-10)', 'Medium (11-20)', 'High (21-50)', 'Very High (51+)'],
                    data: [{}, {}, {}, {}, {}]
                }},
                language: {{
                    labels: [{}],
                    data: [{}]
                }}
            }};
            
            // Optimized chart rendering with lazy loading
            function createChart(canvasId, type, data, options) {{
                const canvas = document.getElementById(canvasId);
                const loading = canvas.parentElement.querySelector('.chart-loading');
                
                return new Promise((resolve) => {{
                    requestAnimationFrame(() => {{
                        const ctx = canvas.getContext('2d');
                        const chart = new Chart(ctx, {{
                            type: type,
                            data: data,
                            options: {{
                                responsive: true,
                                maintainAspectRatio: false,
                                animation: {{
                                    duration: 800,
                                    easing: 'easeOutQuart'
                                }},
                                ...options
                            }}
                        }});
                        
                        loading.style.display = 'none';
                        canvas.style.display = 'block';
                        resolve(chart);
                    }});
                }});
            }}
            
            // Lazy loading for sections
            const observerOptions = {{
                threshold: 0.1,
                rootMargin: '50px'
            }};
            
            const observer = new IntersectionObserver((entries) => {{
                entries.forEach(entry => {{
                    if (entry.isIntersecting) {{
                        entry.target.classList.add('loaded');
                        observer.unobserve(entry.target);
                    }}
                }});
            }}, observerOptions);
            
            // Initialize lazy sections
            document.querySelectorAll('.lazy-section').forEach(section => {{
                observer.observe(section);
            }});
            
            // Initialize charts with staggered loading
            document.addEventListener('DOMContentLoaded', function() {{
                // Load distribution chart first
                setTimeout(() => {{
                    createChart('distributionChart', 'doughnut', {{
                        labels: chartData.distribution.labels,
                        datasets: [{{
                            data: chartData.distribution.data,
                            backgroundColor: ['#28a745', '#6c757d', '#17a2b8', '#f8f9fa'],
                            borderWidth: 2,
                            borderColor: '#fff'
                        }}]
                    }}, {{
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
                    }});
                }}, 100);
                
                // Load complexity chart second
                setTimeout(() => {{
                    createChart('complexityChart', 'bar', {{
                        labels: chartData.complexity.labels,
                        datasets: [{{
                            label: 'Functions',
                            data: chartData.complexity.data,
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
                            borderWidth: 1
                        }}]
                    }}, {{
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
                    }});
                }}, 300);
                
                // Load language chart last
                setTimeout(() => {{
                    createChart('languageChart', 'bar', {{
                        labels: chartData.language.labels,
                        datasets: [{{
                            label: 'Lines of Code',
                            data: chartData.language.data,
                            backgroundColor: [
                                '#e74c3c', '#3498db', '#f39c12', '#2ecc71', '#9b59b6', 
                                '#1abc9c', '#e67e22', '#34495e', '#f1c40f', '#e91e63'
                            ],
                            borderColor: [
                                '#c0392b', '#2980b9', '#d68910', '#27ae60', '#8e44ad',
                                '#16a085', '#d35400', '#2c3e50', '#f39c12', '#c2185b'
                            ],
                            borderWidth: 1
                        }}]
                    }}, {{
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
                    }});
                }}, 500);
            }});
        </script>
    </div>
</body>
</html>
"#,
            // Metrics
            aggregated_stats.basic.total_files,
            aggregated_stats.basic.total_lines,
            aggregated_stats.basic.code_lines,
            aggregated_stats.complexity.function_count,
            aggregated_stats.complexity.cyclomatic_complexity,
            aggregated_stats.complexity.cognitive_complexity,
            aggregated_stats.complexity.maintainability_index,
            aggregated_stats.complexity.average_parameters_per_function,
            
            // Quality metrics with real values and dynamic classes
            self.get_quality_class(aggregated_stats.complexity.quality_metrics.code_health_score),
            aggregated_stats.complexity.quality_metrics.code_health_score,
            self.get_progress_class(aggregated_stats.complexity.quality_metrics.code_health_score),
            aggregated_stats.complexity.quality_metrics.code_health_score,
            
            self.get_quality_class(aggregated_stats.complexity.quality_metrics.maintainability_index),
            aggregated_stats.complexity.quality_metrics.maintainability_index,
            self.get_progress_class(aggregated_stats.complexity.quality_metrics.maintainability_index),
            aggregated_stats.complexity.quality_metrics.maintainability_index,
            
            self.get_quality_class(aggregated_stats.complexity.quality_metrics.function_size_health),
            aggregated_stats.complexity.quality_metrics.function_size_health,
            self.get_progress_class(aggregated_stats.complexity.quality_metrics.function_size_health),
            aggregated_stats.complexity.quality_metrics.function_size_health,
            
            self.get_quality_class(aggregated_stats.complexity.quality_metrics.nesting_depth_health),
            aggregated_stats.complexity.quality_metrics.nesting_depth_health,
            self.get_progress_class(aggregated_stats.complexity.quality_metrics.nesting_depth_health),
            aggregated_stats.complexity.quality_metrics.nesting_depth_health,
            
            // Tables and insights
            self.template_generator.generate_extension_rows_with_real_analysis(&aggregated_stats),
            self.template_generator.generate_real_complexity_insights(&aggregated_stats)
                .replace("\n", "</div><div class=\"insight-item\">"),
            self.template_generator.generate_optimized_individual_files_section(individual_files),
            
            // Chart data
            aggregated_stats.basic.code_lines,
            aggregated_stats.basic.comment_lines,
            aggregated_stats.basic.doc_lines,
            aggregated_stats.basic.blank_lines,
            
            // Complexity distribution data
            aggregated_stats.complexity.complexity_distribution.very_low_complexity,
            aggregated_stats.complexity.complexity_distribution.low_complexity,
            aggregated_stats.complexity.complexity_distribution.medium_complexity,
            aggregated_stats.complexity.complexity_distribution.high_complexity,
            aggregated_stats.complexity.complexity_distribution.very_high_complexity,
            
            self.template_generator.generate_complexity_labels(stats),
            self.template_generator.generate_complexity_data_with_real_analysis(&aggregated_stats)
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
    <title>HowMany - Code Analysis Report</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js@3.9.1/dist/chart.min.js"></script>
    <style>
        * {{ box-sizing: border-box; }}
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            margin: 0;
            padding: 20px;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
        }}
        .container {{
            max-width: 1400px;
            margin: 0 auto;
            background: white;
            border-radius: 15px;
            box-shadow: 0 20px 40px rgba(0,0,0,0.15);
            overflow: hidden;
        }}
        .header {{
            background: linear-gradient(135deg, #4facfe 0%, #00f2fe 100%);
            color: white;
            padding: 40px 30px;
            text-align: center;
        }}
        .header h1 {{
            font-size: 3em;
            font-weight: 300;
            margin: 0 0 10px 0;
        }}
        .header .subtitle {{
            font-size: 1.2em;
            opacity: 0.9;
        }}
        .content {{
            padding: 40px;
        }}
        .section {{
            margin-bottom: 40px;
        }}
        .section-title {{
            font-size: 2em;
            font-weight: 600;
            margin-bottom: 20px;
            color: #2c3e50;
            border-bottom: 3px solid #3498db;
            padding-bottom: 10px;
        }}
        .summary-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 20px;
            margin: 20px 0;
        }}
        .summary-card {{
            background: #f8f9fa;
            padding: 20px;
            border-radius: 10px;
            text-align: center;
            border-left: 4px solid #3498db;
            will-change: transform;
            transition: transform 0.2s ease;
        }}
        .summary-card:hover {{
            transform: translateY(-2px);
        }}
        .summary-number {{
            font-size: 2.5em;
            font-weight: bold;
            color: #2c3e50;
            margin-bottom: 5px;
        }}
        .summary-label {{
            color: #6c757d;
            font-size: 0.9em;
            text-transform: uppercase;
        }}
        .quality-metrics {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 20px;
            margin: 20px 0;
        }}
        .quality-metric {{
            background: #f8f9fa;
            padding: 20px;
            border-radius: 10px;
            text-align: center;
            border-left: 4px solid #27ae60;
        }}
        .quality-score {{
            font-size: 2em;
            font-weight: bold;
            margin-bottom: 10px;
        }}
        .quality-excellent {{ color: #27ae60; }}
        .quality-good {{ color: #f39c12; }}
        .quality-poor {{ color: #e74c3c; }}
        .charts-section {{
            display: grid;
            grid-template-columns: 1fr 1fr;
            gap: 30px;
            margin: 30px 0;
        }}
        .chart-container {{
            background: white;
            border-radius: 15px;
            padding: 25px;
            box-shadow: 0 4px 6px rgba(0,0,0,0.1);
            position: relative;
        }}
        .chart-title {{
            font-size: 1.3em;
            font-weight: 600;
            margin-bottom: 20px;
            text-align: center;
            color: #2c3e50;
        }}
        .chart-loading {{
            position: absolute;
            top: 50%;
            left: 50%;
            transform: translate(-50%, -50%);
            color: #666;
            font-size: 1.1em;
        }}
        .data-table {{
            width: 100%;
            border-collapse: collapse;
            margin-top: 20px;
            background: white;
            border-radius: 10px;
            overflow: hidden;
            box-shadow: 0 4px 6px rgba(0,0,0,0.1);
        }}
        .data-table th {{
            background: #2c3e50;
            color: white;
            padding: 15px;
            text-align: left;
            font-weight: 600;
        }}
        .data-table td {{
            padding: 12px 15px;
            border-bottom: 1px solid #dee2e6;
        }}
        .data-table tr:nth-child(even) {{
            background: #f8f9fa;
        }}
        .complexity-badge {{
            padding: 4px 8px;
            border-radius: 4px;
            font-size: 0.8em;
            font-weight: bold;
        }}
        .complexity-very-low {{ background: #d4edda; color: #155724; }}
        .complexity-low {{ background: #d1ecf1; color: #0c5460; }}
        .complexity-medium {{ background: #fff3cd; color: #856404; }}
        .complexity-high {{ background: #f8d7da; color: #721c24; }}
        .complexity-very-high {{ background: #f5c6cb; color: #721c24; }}
        .insights-section {{
            background: #f8f9fa;
            padding: 25px;
            border-radius: 10px;
            margin: 20px 0;
        }}
        .footer {{
            background: #2c3e50;
            color: white;
            padding: 20px;
            text-align: center;
        }}
        .time-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 20px;
            margin: 20px 0;
        }}
        .time-card {{
            background: #f8f9fa;
            padding: 20px;
            border-radius: 10px;
            text-align: center;
            border-left: 4px solid #9b59b6;
        }}
        .time-value {{
            font-size: 1.5em;
            font-weight: bold;
            color: #8e44ad;
            margin-bottom: 5px;
        }}
        .lazy-section {{
            opacity: 0;
            transition: opacity 0.3s ease;
        }}
        .lazy-section.loaded {{
            opacity: 1;
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>📊 Code Analysis Report</h1>
            <div class="subtitle">Comprehensive codebase insights and recommendations</div>
        </div>
        
        <div class="content">
            <div class="section">
                <h2 class="section-title">📋 Executive Summary</h2>
                <div class="summary-grid">
                    <div class="summary-card">
                        <div class="summary-number">{}</div>
                        <div class="summary-label">Total Files</div>
                    </div>
                    <div class="summary-card">
                        <div class="summary-number">{}</div>
                        <div class="summary-label">Lines of Code</div>
                    </div>
                    <div class="summary-card">
                        <div class="summary-number">{}</div>
                        <div class="summary-label">Functions</div>
                    </div>
                    <div class="summary-card">
                        <div class="summary-number">{:.1}</div>
                        <div class="summary-label">Avg Complexity</div>
                    </div>
                    <div class="summary-card">
                        <div class="summary-number">{:.1}%</div>
                        <div class="summary-label">Code Quality</div>
                    </div>
                    <div class="summary-card">
                        <div class="summary-number">{}</div>
                        <div class="summary-label">Est. Dev Time</div>
                    </div>
                </div>
            </div>

            <div class="section">
                <h2 class="section-title">🎯 Quality Metrics</h2>
                <div class="quality-metrics">
                    <div class="quality-metric">
                        <div class="quality-score {}">{:.1}%</div>
                        <div>Overall Health</div>
                    </div>
                    <div class="quality-metric">
                        <div class="quality-score {}">{:.1}%</div>
                        <div>Maintainability</div>
                    </div>
                    <div class="quality-metric">
                        <div class="quality-score {}">{:.1}%</div>
                        <div>Readability</div>
                    </div>
                    <div class="quality-metric">
                        <div class="quality-score {}">{:.1}%</div>
                        <div>Testability</div>
                    </div>
                </div>
            </div>

            <div class="section">
                <h2 class="section-title">⏱️ Time Analysis</h2>
                <div class="time-grid">
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
                        <div class="time-value">{:.1}</div>
                        <div>Lines/Hour Rate</div>
                    </div>
                </div>
            </div>

            <div class="charts-section">
                <div class="chart-container">
                    <div class="chart-title">📊 Code Distribution</div>
                    <div class="chart-loading">Loading chart...</div>
                    <canvas id="distributionChart" style="display: none;"></canvas>
                </div>
                <div class="chart-container">
                    <div class="chart-title">🏗️ Complexity Analysis</div>
                    <div class="chart-loading">Loading chart...</div>
                    <canvas id="complexityChart" style="display: none;"></canvas>
                </div>
            </div>

            <div class="lazy-section" id="insights">
                <div class="section">
                    <h2 class="section-title">💡 Insights & Recommendations</h2>
                    <div class="insights-section">
                        <h3>Code Analysis</h3>
                        <p>{}</p>
                    </div>
                    <div class="insights-section">
                        <h3>Improvement Opportunities</h3>
                        <p>{}</p>
                    </div>
                </div>
            </div>

            <div class="lazy-section" id="fileAnalysis">
                <div class="section">
                    <h2 class="section-title">📁 File Analysis</h2>
                    <table class="data-table">
                        <thead>
                            <tr>
                                <th>Language</th>
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
                </div>
            </div>

            <div class="lazy-section" id="individualFiles">
                {}
            </div>
        </div>
        
        <div class="footer">
            Generated by HowMany v{} • Analysis completed in {}ms
        </div>
    </div>
    
    <script>
        // Chart data
        const chartData = {{
            distribution: {{
                labels: ['Code Lines', 'Comments', 'Documentation', 'Blank Lines'],
                data: [{}, {}, {}, {}]
            }},
            complexity: {{
                labels: ['Very Low', 'Low', 'Medium', 'High', 'Very High'],
                data: [{}, {}, {}, {}, {}]
            }}
        }};
        
        // Optimized chart rendering
        function createChart(canvasId, type, data, options) {{
            const canvas = document.getElementById(canvasId);
            const loading = canvas.parentElement.querySelector('.chart-loading');
            
            return new Promise((resolve) => {{
                requestAnimationFrame(() => {{
                    const ctx = canvas.getContext('2d');
                    const chart = new Chart(ctx, {{
                        type: type,
                        data: data,
                        options: {{
                            responsive: true,
                            maintainAspectRatio: false,
                            animation: {{
                                duration: 600,
                                easing: 'easeOutQuart'
                            }},
                            ...options
                        }}
                    }});
                    
                    loading.style.display = 'none';
                    canvas.style.display = 'block';
                    resolve(chart);
                }});
            }});
        }}
        
        // Lazy loading observer
        const observer = new IntersectionObserver((entries) => {{
            entries.forEach(entry => {{
                if (entry.isIntersecting) {{
                    entry.target.classList.add('loaded');
                    observer.unobserve(entry.target);
                }}
            }});
        }}, {{ threshold: 0.1, rootMargin: '50px' }});
        
        // Initialize lazy sections
        document.querySelectorAll('.lazy-section').forEach(section => {{
            observer.observe(section);
        }});
        
        // Initialize charts
        document.addEventListener('DOMContentLoaded', function() {{
            setTimeout(() => {{
                createChart('distributionChart', 'doughnut', {{
                    labels: chartData.distribution.labels,
                    datasets: [{{
                        data: chartData.distribution.data,
                        backgroundColor: ['#3498db', '#95a5a6', '#2ecc71', '#ecf0f1'],
                        borderColor: '#ffffff',
                        borderWidth: 2
                    }}]
                }}, {{
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
                }});
            }}, 100);
            
            setTimeout(() => {{
                createChart('complexityChart', 'bar', {{
                    labels: chartData.complexity.labels,
                    datasets: [{{
                        label: 'Number of Functions',
                        data: chartData.complexity.data,
                        backgroundColor: ['#27ae60', '#2ecc71', '#f39c12', '#e67e22', '#e74c3c'],
                        borderColor: '#2c3e50',
                        borderWidth: 1
                    }}]
                }}, {{
                    plugins: {{
                        legend: {{ display: false }},
                        tooltip: {{
                            callbacks: {{
                                label: function(context) {{
                                    return context.parsed.y + ' functions';
                                }}
                            }}
                        }}
                    }},
                    scales: {{
                        y: {{
                            beginAtZero: true,
                            title: {{ display: true, text: 'Number of Functions' }}
                        }},
                        x: {{
                            title: {{ display: true, text: 'Complexity Level' }}
                        }}
                    }}
                }});
            }}, 300);
        }});
    </script>
</body>
</html>"#,
            // Summary data
            aggregated_stats.basic.total_files,
            aggregated_stats.basic.code_lines,
            aggregated_stats.complexity.function_count,
            aggregated_stats.complexity.cyclomatic_complexity,
            aggregated_stats.complexity.quality_metrics.code_health_score,
            aggregated_stats.time.total_time_formatted,
            
            // Quality metrics
            self.get_quality_class(aggregated_stats.complexity.quality_metrics.code_health_score),
            aggregated_stats.complexity.quality_metrics.code_health_score,
            self.get_quality_class(aggregated_stats.complexity.quality_metrics.maintainability_index),
            aggregated_stats.complexity.quality_metrics.maintainability_index,
            self.get_quality_class(aggregated_stats.ratios.quality_metrics.readability_score),
            aggregated_stats.ratios.quality_metrics.readability_score,
            self.get_quality_class(aggregated_stats.complexity.quality_metrics.nesting_depth_health),
            aggregated_stats.complexity.quality_metrics.nesting_depth_health,
            
            // Time analysis
            aggregated_stats.time.total_time_formatted,
            aggregated_stats.time.code_time_formatted,
            aggregated_stats.time.doc_time_formatted,
            aggregated_stats.time.productivity_metrics.lines_per_hour,
            
            // Insights and recommendations
            self.template_generator.generate_enhanced_insights(aggregated_stats),
            self.template_generator.generate_enhanced_recommendations(aggregated_stats),
            
            // File analysis table
            self.template_generator.generate_extension_rows_with_real_analysis(aggregated_stats),
            
            // Individual files section
            self.template_generator.generate_optimized_individual_files_section(individual_files),
            
            // Footer
            aggregated_stats.metadata.version,
            aggregated_stats.metadata.calculation_time_ms,
            
            // Chart data
            aggregated_stats.basic.code_lines,
            aggregated_stats.basic.comment_lines,
            aggregated_stats.basic.doc_lines,
            aggregated_stats.basic.blank_lines,
            
            // Complexity distribution
            aggregated_stats.complexity.complexity_distribution.very_low_complexity,
            aggregated_stats.complexity.complexity_distribution.low_complexity,
            aggregated_stats.complexity.complexity_distribution.medium_complexity,
            aggregated_stats.complexity.complexity_distribution.high_complexity,
            aggregated_stats.complexity.complexity_distribution.very_high_complexity,
        );
        
        Ok(html)
    }
    
    /// Get CSS class for quality score
    fn get_quality_class(&self, score: f64) -> &'static str {
        if score >= 80.0 {
            "quality-excellent"
        } else if score >= 60.0 {
            "quality-good"
        } else {
            "quality-poor"
        }
    }
    
    /// Get CSS class for progress bar
    fn get_progress_class(&self, score: f64) -> &'static str {
        if score >= 80.0 {
            "progress-excellent"
        } else if score >= 60.0 {
            "progress-good"
        } else {
            "progress-poor"
        }
    }
} 