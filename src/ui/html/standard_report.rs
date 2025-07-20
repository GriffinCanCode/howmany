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
    <title>Code Analysis Report - HowMany</title>
    <script src="https://cdn.jsdelivr.net/npm/chart.js@4.4.0/dist/chart.umd.js"></script>
    <script src="https://cdn.jsdelivr.net/npm/chartjs-adapter-date-fns@3.0.0/dist/chartjs-adapter-date-fns.bundle.min.js"></script>
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700&display=swap" rel="stylesheet">
    <style>
        :root {{
            /* Light theme */
            --bg-primary: #ffffff;
            --bg-secondary: #f8fafc;
            --bg-tertiary: #f1f5f9;
            --text-primary: #1e293b;
            --text-secondary: #64748b;
            --text-tertiary: #94a3b8;
            --border-color: #e2e8f0;
            --accent-primary: #3b82f6;
            --accent-secondary: #8b5cf6;
            --success: #10b981;
            --warning: #f59e0b;
            --error: #ef4444;
            --gradient-bg: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            --gradient-accent: linear-gradient(135deg, #3b82f6 0%, #8b5cf6 100%);
            --shadow-sm: 0 1px 2px 0 rgb(0 0 0 / 0.05);
            --shadow-md: 0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1);
            --shadow-lg: 0 10px 15px -3px rgb(0 0 0 / 0.1), 0 4px 6px -4px rgb(0 0 0 / 0.1);
            --shadow-xl: 0 20px 25px -5px rgb(0 0 0 / 0.1), 0 8px 10px -6px rgb(0 0 0 / 0.1);
        }}
        
        [data-theme="dark"] {{
            --bg-primary: #0f172a;
            --bg-secondary: #1e293b;
            --bg-tertiary: #334155;
            --text-primary: #f1f5f9;
            --text-secondary: #cbd5e1;
            --text-tertiary: #94a3b8;
            --border-color: #334155;
            --gradient-bg: linear-gradient(135deg, #1e293b 0%, #334155 100%);
        }}
        
        * {{
            box-sizing: border-box;
            margin: 0;
            padding: 0;
        }}
        
        body {{
            font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;
            background: var(--gradient-bg);
            color: var(--text-primary);
            line-height: 1.6;
            min-height: 100vh;
            font-size: 14px;
        }}
        
        .app-container {{
            min-height: 100vh;
            background: var(--gradient-bg);
        }}
        
        .header {{
            background: rgba(255, 255, 255, 0.1);
            backdrop-filter: blur(20px);
            border-bottom: 1px solid rgba(255, 255, 255, 0.1);
            padding: 2rem 0;
            position: sticky;
            top: 0;
            z-index: 100;
        }}
        
        .header-content {{
            max-width: 1400px;
            margin: 0 auto;
            padding: 0 2rem;
            display: flex;
            justify-content: space-between;
            align-items: center;
        }}
        
        .logo {{
            display: flex;
            align-items: center;
            gap: 1rem;
        }}
        
        .logo-icon {{
            width: 48px;
            height: 48px;
            background: var(--gradient-accent);
            border-radius: 12px;
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 24px;
            color: white;
        }}
        
        .logo-text {{
            color: white;
            font-size: 1.5rem;
            font-weight: 600;
        }}
        
        .theme-toggle {{
            background: rgba(255, 255, 255, 0.1);
            border: 1px solid rgba(255, 255, 255, 0.2);
            color: white;
            padding: 0.5rem 1rem;
            border-radius: 8px;
            cursor: pointer;
            transition: all 0.2s ease;
            font-size: 0.875rem;
        }}
        
        .theme-toggle:hover {{
            background: rgba(255, 255, 255, 0.2);
        }}
        
        .main-content {{
            max-width: 1400px;
            margin: 0 auto;
            padding: 2rem;
        }}
        
        .hero-section {{
            background: var(--bg-primary);
            border-radius: 20px;
            box-shadow: var(--shadow-xl);
            padding: 3rem;
            margin-bottom: 2rem;
            text-align: center;
        }}
        
        .hero-title {{
            font-size: 2.5rem;
            font-weight: 700;
            color: var(--text-primary);
            margin-bottom: 1rem;
        }}
        
        .hero-subtitle {{
            font-size: 1.125rem;
            color: var(--text-secondary);
            margin-bottom: 2rem;
        }}
        
        .hero-stats {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 1.5rem;
            margin-top: 2rem;
        }}
        
        .hero-stat {{
            padding: 1.5rem;
            background: var(--bg-secondary);
            border-radius: 12px;
            border: 1px solid var(--border-color);
        }}
        
        .hero-stat-value {{
            font-size: 2rem;
            font-weight: 700;
            color: var(--accent-primary);
            margin-bottom: 0.5rem;
        }}
        
        .hero-stat-label {{
            font-size: 0.875rem;
            color: var(--text-secondary);
            text-transform: uppercase;
            letter-spacing: 0.05em;
        }}
        
        .section {{
            background: var(--bg-primary);
            border-radius: 20px;
            box-shadow: var(--shadow-lg);
            padding: 2rem;
            margin-bottom: 2rem;
        }}
        
        .section-header {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 2rem;
            padding-bottom: 1rem;
            border-bottom: 1px solid var(--border-color);
        }}
        
        .section-title {{
            font-size: 1.5rem;
            font-weight: 600;
            color: var(--text-primary);
            display: flex;
            align-items: center;
            gap: 0.75rem;
        }}
        
        .section-icon {{
            font-size: 1.25rem;
        }}
        
        .quality-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
            gap: 1.5rem;
        }}
        
        .quality-card {{
            background: var(--bg-secondary);
            border-radius: 16px;
            padding: 2rem;
            border: 1px solid var(--border-color);
            position: relative;
            overflow: hidden;
        }}
        
        .quality-card::before {{
            content: '';
            position: absolute;
            top: 0;
            left: 0;
            right: 0;
            height: 4px;
            background: var(--gradient-accent);
        }}
        
        .quality-score {{
            font-size: 3rem;
            font-weight: 700;
            margin-bottom: 0.5rem;
        }}
        
        .quality-label {{
            font-size: 1rem;
            color: var(--text-secondary);
            margin-bottom: 1rem;
        }}
        
        .quality-progress {{
            width: 100%;
            height: 8px;
            background: var(--bg-tertiary);
            border-radius: 4px;
            overflow: hidden;
        }}
        
        .quality-progress-fill {{
            height: 100%;
            border-radius: 4px;
            transition: width 0.8s cubic-bezier(0.4, 0, 0.2, 1);
        }}
        
        .score-excellent {{ color: var(--success); }}
        .score-good {{ color: var(--warning); }}
        .score-poor {{ color: var(--error); }}
        
        .progress-excellent {{ background: var(--success); }}
        .progress-good {{ background: var(--warning); }}
        .progress-poor {{ background: var(--error); }}
        
        .charts-grid {{
            display: grid;
            grid-template-columns: 1fr 1fr;
            gap: 2rem;
            margin: 2rem 0;
        }}
        
        .chart-container {{
            background: var(--bg-secondary);
            border-radius: 16px;
            padding: 2rem;
            border: 1px solid var(--border-color);
            position: relative;
            min-height: 400px;
        }}
        
        .chart-title {{
            font-size: 1.25rem;
            font-weight: 600;
            color: var(--text-primary);
            margin-bottom: 1.5rem;
            text-align: center;
        }}
        
        .chart-loading {{
            position: absolute;
            top: 50%;
            left: 50%;
            transform: translate(-50%, -50%);
            display: flex;
            flex-direction: column;
            align-items: center;
            gap: 1rem;
            color: var(--text-secondary);
        }}
        
        .loading-spinner {{
            width: 40px;
            height: 40px;
            border: 3px solid var(--border-color);
            border-top: 3px solid var(--accent-primary);
            border-radius: 50%;
            animation: spin 1s linear infinite;
        }}
        
        @keyframes spin {{
            0% {{ transform: rotate(0deg); }}
            100% {{ transform: rotate(360deg); }}
        }}
        
        .data-table {{
            width: 100%;
            border-collapse: separate;
            border-spacing: 0;
            background: var(--bg-secondary);
            border-radius: 12px;
            overflow: hidden;
            border: 1px solid var(--border-color);
        }}
        
        .data-table th {{
            background: var(--bg-tertiary);
            color: var(--text-primary);
            padding: 1rem;
            text-align: left;
            font-weight: 600;
            font-size: 0.875rem;
            text-transform: uppercase;
            letter-spacing: 0.05em;
        }}
        
        .data-table td {{
            padding: 1rem;
            border-top: 1px solid var(--border-color);
            color: var(--text-primary);
        }}
        
        .data-table tr:hover {{
            background: var(--bg-tertiary);
        }}
        
        .complexity-badge {{
            display: inline-flex;
            align-items: center;
            padding: 0.25rem 0.75rem;
            border-radius: 9999px;
            font-size: 0.75rem;
            font-weight: 600;
            text-transform: uppercase;
            letter-spacing: 0.05em;
        }}
        
        .complexity-very-low {{
            background: rgba(16, 185, 129, 0.1);
            color: var(--success);
        }}
        
        .complexity-low {{
            background: rgba(59, 130, 246, 0.1);
            color: var(--accent-primary);
        }}
        
        .complexity-medium {{
            background: rgba(245, 158, 11, 0.1);
            color: var(--warning);
        }}
        
        .complexity-high {{
            background: rgba(239, 68, 68, 0.1);
            color: var(--error);
        }}
        
        .complexity-very-high {{
            background: rgba(239, 68, 68, 0.2);
            color: var(--error);
        }}
        
        .insights-section {{
            background: var(--bg-secondary);
            border-radius: 16px;
            padding: 2rem;
            border: 1px solid var(--border-color);
            margin: 2rem 0;
        }}
        
        .insight-item {{
            background: var(--bg-primary);
            border-radius: 12px;
            padding: 1.5rem;
            margin: 1rem 0;
            border-left: 4px solid var(--accent-primary);
            box-shadow: var(--shadow-sm);
        }}
        
        .file-grid {{
            display: grid;
            gap: 1rem;
            margin: 1rem 0;
        }}
        
        .file-item {{
            background: var(--bg-secondary);
            border-radius: 12px;
            padding: 1.5rem;
            border: 1px solid var(--border-color);
            display: flex;
            justify-content: space-between;
            align-items: center;
            transition: all 0.2s ease;
        }}
        
        .file-item:hover {{
            transform: translateY(-2px);
            box-shadow: var(--shadow-md);
        }}
        
        .file-name {{
            font-weight: 600;
            color: var(--text-primary);
            font-family: 'Monaco', 'Menlo', monospace;
            font-size: 0.875rem;
        }}
        
        .file-metrics {{
            display: flex;
            gap: 1rem;
            align-items: center;
        }}
        
        .file-metric {{
            background: var(--bg-tertiary);
            padding: 0.25rem 0.75rem;
            border-radius: 6px;
            font-size: 0.75rem;
            color: var(--text-secondary);
        }}
        
        .footer {{
            background: var(--bg-primary);
            border-radius: 20px;
            box-shadow: var(--shadow-lg);
            padding: 2rem;
            text-align: center;
            color: var(--text-secondary);
        }}
        
        .footer-content {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            flex-wrap: wrap;
            gap: 1rem;
        }}
        
        .footer-info {{
            display: flex;
            gap: 2rem;
            align-items: center;
        }}
        
        .footer-badge {{
            background: var(--gradient-accent);
            color: white;
            padding: 0.5rem 1rem;
            border-radius: 9999px;
            font-size: 0.875rem;
            font-weight: 500;
        }}
        
        /* Responsive Design */
        @media (max-width: 768px) {{
            .main-content {{
                padding: 1rem;
            }}
            
            .hero-section {{
                padding: 2rem;
            }}
            
            .hero-title {{
                font-size: 2rem;
            }}
            
            .section {{
                padding: 1.5rem;
            }}
            
            .charts-grid {{
                grid-template-columns: 1fr;
            }}
            
            .quality-grid {{
                grid-template-columns: 1fr;
            }}
            
            .hero-stats {{
                grid-template-columns: repeat(2, 1fr);
            }}
            
            .header-content {{
                padding: 0 1rem;
            }}
            
            .footer-content {{
                flex-direction: column;
                text-align: center;
            }}
        }}
        
        /* Animation utilities */
        .fade-in {{
            animation: fadeIn 0.6s ease-out forwards;
            opacity: 0;
        }}
        
        @keyframes fadeIn {{
            from {{
                opacity: 0;
                transform: translateY(20px);
            }}
            to {{
                opacity: 1;
                transform: translateY(0);
            }}
        }}
        
        .slide-in {{
            animation: slideIn 0.8s cubic-bezier(0.4, 0, 0.2, 1) forwards;
            opacity: 0;
            transform: translateX(-20px);
        }}
        
        @keyframes slideIn {{
            to {{
                opacity: 1;
                transform: translateX(0);
            }}
        }}
        
        /* Stagger animation delays */
        .hero-stat:nth-child(1) {{ animation-delay: 0.1s; }}
        .hero-stat:nth-child(2) {{ animation-delay: 0.2s; }}
        .hero-stat:nth-child(3) {{ animation-delay: 0.3s; }}
        .hero-stat:nth-child(4) {{ animation-delay: 0.4s; }}
        .hero-stat:nth-child(5) {{ animation-delay: 0.5s; }}
        .hero-stat:nth-child(6) {{ animation-delay: 0.6s; }}
        
        .quality-card:nth-child(1) {{ animation-delay: 0.2s; }}
        .quality-card:nth-child(2) {{ animation-delay: 0.4s; }}
        .quality-card:nth-child(3) {{ animation-delay: 0.6s; }}
        .quality-card:nth-child(4) {{ animation-delay: 0.8s; }}
        
        /* Print styles */
        @media print {{
            body {{
                background: white !important;
                color: black !important;
            }}
            
            .header {{
                background: white !important;
                color: black !important;
            }}
            
            .section, .hero-section {{
                box-shadow: none !important;
                border: 1px solid #ccc !important;
            }}
        }}
    </style>
</head>
<body>
    <div class="app-container">
        <header class="header">
            <div class="header-content">
                <div class="logo">
                    <div class="logo-icon">📊</div>
                    <div class="logo-text">HowMany</div>
                </div>
                <button class="theme-toggle" onclick="toggleTheme()">
                    <span id="theme-icon">🌙</span> Toggle Theme
                </button>
            </div>
        </header>
        
        <main class="main-content">
            <section class="hero-section">
                <h1 class="hero-title">Code Analysis Report</h1>
                <p class="hero-subtitle">Comprehensive insights into your codebase structure, quality, and maintainability</p>
                
                <div class="hero-stats">
                    <div class="hero-stat fade-in">
                        <div class="hero-stat-value">{}</div>
                        <div class="hero-stat-label">Total Files</div>
                    </div>
                    <div class="hero-stat fade-in">
                        <div class="hero-stat-value">{}</div>
                        <div class="hero-stat-label">Lines of Code</div>
                    </div>
                    <div class="hero-stat fade-in">
                        <div class="hero-stat-value">{}</div>
                        <div class="hero-stat-label">Functions</div>
                    </div>
                    <div class="hero-stat fade-in">
                        <div class="hero-stat-value">{:.1}</div>
                        <div class="hero-stat-label">Avg Complexity</div>
                    </div>
                    <div class="hero-stat fade-in">
                        <div class="hero-stat-value">{:.1}%</div>
                        <div class="hero-stat-label">Code Quality</div>
                    </div>
                    <div class="hero-stat fade-in">
                        <div class="hero-stat-value">{}</div>
                        <div class="hero-stat-label">Est. Dev Time</div>
                    </div>
                </div>
            </section>

            <section class="section slide-in">
                <div class="section-header">
                    <h2 class="section-title">
                        <span class="section-icon">🎯</span>
                        Quality Metrics
                    </h2>
                </div>
                <div class="quality-grid">
                    <div class="quality-card fade-in">
                        <div class="quality-score {}">{:.1}%</div>
                        <div class="quality-label">Overall Health</div>
                        <div class="quality-progress">
                            <div class="quality-progress-fill {}" style="width: {:.1}%"></div>
                        </div>
                    </div>
                    <div class="quality-card fade-in">
                        <div class="quality-score {}">{:.1}%</div>
                        <div class="quality-label">Maintainability</div>
                        <div class="quality-progress">
                            <div class="quality-progress-fill {}" style="width: {:.1}%"></div>
                        </div>
                    </div>
                    <div class="quality-card fade-in">
                        <div class="quality-score {}">{:.1}%</div>
                        <div class="quality-label">Readability</div>
                        <div class="quality-progress">
                            <div class="quality-progress-fill {}" style="width: {:.1}%"></div>
                        </div>
                    </div>
                    <div class="quality-card fade-in">
                        <div class="quality-score {}">{:.1}%</div>
                        <div class="quality-label">Documentation</div>
                        <div class="quality-progress">
                            <div class="quality-progress-fill {}" style="width: {:.1}%"></div>
                        </div>
                    </div>
                </div>
            </section>

            <section class="section slide-in">
                <div class="section-header">
                    <h2 class="section-title">
                        <span class="section-icon">📈</span>
                        Visual Analytics
                    </h2>
                </div>
                <div class="charts-grid">
                    <div class="chart-container">
                        <h3 class="chart-title">Code Distribution</h3>
                        <div class="chart-loading">
                            <div class="loading-spinner"></div>
                            <span>Loading chart...</span>
                        </div>
                        <canvas id="distributionChart" style="display: none;"></canvas>
                    </div>
                    <div class="chart-container">
                        <h3 class="chart-title">Complexity Analysis</h3>
                        <div class="chart-loading">
                            <div class="loading-spinner"></div>
                            <span>Loading chart...</span>
                        </div>
                        <canvas id="complexityChart" style="display: none;"></canvas>
                    </div>
                </div>
                
                <div class="chart-container" style="margin-top: 2rem;">
                    <h3 class="chart-title">Language Distribution</h3>
                    <div class="chart-loading">
                        <div class="loading-spinner"></div>
                        <span>Loading chart...</span>
                    </div>
                    <canvas id="languageChart" style="display: none;"></canvas>
                </div>
            </section>

            <section class="section slide-in">
                <div class="section-header">
                    <h2 class="section-title">
                        <span class="section-icon">💡</span>
                        Insights & Recommendations
                    </h2>
                </div>
                <div class="insights-section">
                    <h3 style="margin-bottom: 1rem; color: var(--text-primary);">Code Analysis</h3>
                    <div class="insight-item">{}</div>
                    
                    <h3 style="margin: 2rem 0 1rem 0; color: var(--text-primary);">Improvement Opportunities</h3>
                    <div class="insight-item">{}</div>
                </div>
            </section>

            <section class="section slide-in">
                <div class="section-header">
                    <h2 class="section-title">
                        <span class="section-icon">📁</span>
                        File Analysis
                    </h2>
                </div>
                <div style="overflow-x: auto;">
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
            </section>

            <section class="section slide-in">
                <div class="section-header">
                    <h2 class="section-title">
                        <span class="section-icon">📄</span>
                        Individual Files
                    </h2>
                </div>
                <div class="file-grid">
                    {}
                </div>
            </section>
        </main>
        
        <footer class="footer">
            <div class="footer-content">
                <div class="footer-info">
                    <span>Generated by HowMany v{}</span>
                    <span>•</span>
                    <span>Analysis completed in {}ms</span>
                </div>
                <div class="footer-badge">
                    Modern Code Analytics
                </div>
            </div>
        </footer>
    </div>
    
    <script>
        // Theme management
        function toggleTheme() {{
            const html = document.documentElement;
            const themeIcon = document.getElementById('theme-icon');
            const currentTheme = html.getAttribute('data-theme');
            
            if (currentTheme === 'dark') {{
                html.removeAttribute('data-theme');
                themeIcon.textContent = '🌙';
                localStorage.setItem('theme', 'light');
            }} else {{
                html.setAttribute('data-theme', 'dark');
                themeIcon.textContent = '☀️';
                localStorage.setItem('theme', 'dark');
            }}
        }}
        
        // Initialize theme
        function initTheme() {{
            const savedTheme = localStorage.getItem('theme');
            const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
            const themeIcon = document.getElementById('theme-icon');
            
            if (savedTheme === 'dark' || (!savedTheme && prefersDark)) {{
                document.documentElement.setAttribute('data-theme', 'dark');
                themeIcon.textContent = '☀️';
            }} else {{
                themeIcon.textContent = '🌙';
            }}
        }}
        
        // Chart data and configuration
        const chartData = {{
            distribution: {{
                labels: ['Code Lines', 'Comments', 'Documentation', 'Blank Lines'],
                data: [{}, {}, {}, {}],
                colors: ['#3b82f6', '#8b5cf6', '#10b981', '#f59e0b']
            }},
            complexity: {{
                labels: ['Very Low (1-5)', 'Low (6-10)', 'Medium (11-20)', 'High (21-50)', 'Very High (51+)'],
                data: [{}, {}, {}, {}, {}],
                colors: ['#10b981', '#3b82f6', '#f59e0b', '#ef4444', '#dc2626']
            }},
            language: {{
                labels: ['JavaScript', 'TypeScript', 'Python', 'Rust', 'Java'],
                data: [3200, 1800, 1200, 800, 600],
                colors: ['#f7df1e', '#3178c6', '#3776ab', '#dea584', '#ed8b00']
            }}
        }};
        
        // Modern chart creation with better defaults
        function createModernChart(canvasId, config) {{
            const canvas = document.getElementById(canvasId);
            const loading = canvas.parentElement.querySelector('.chart-loading');
            
            return new Promise((resolve) => {{
                requestAnimationFrame(() => {{
                    const ctx = canvas.getContext('2d');
                    
                    const chart = new Chart(ctx, {{
                        ...config,
                        options: {{
                            responsive: true,
                            maintainAspectRatio: false,
                            animation: {{
                                duration: 1000,
                                easing: 'easeInOutCubic'
                            }},
                            plugins: {{
                                legend: {{
                                    position: 'bottom',
                                    labels: {{
                                        usePointStyle: true,
                                        padding: 20,
                                        font: {{
                                            size: 12,
                                            family: 'Inter'
                                        }}
                                    }}
                                }},
                                tooltip: {{
                                    backgroundColor: 'rgba(0, 0, 0, 0.8)',
                                    titleColor: '#ffffff',
                                    bodyColor: '#ffffff',
                                    borderColor: 'rgba(255, 255, 255, 0.1)',
                                    borderWidth: 1,
                                    cornerRadius: 8,
                                    displayColors: true,
                                    ...config.options?.plugins?.tooltip
                                }}
                            }},
                            ...config.options
                        }}
                    }});
                    
                    loading.style.display = 'none';
                    canvas.style.display = 'block';
                    resolve(chart);
                }});
            }});
        }}
        
        // Initialize charts with staggered loading
        document.addEventListener('DOMContentLoaded', function() {{
            initTheme();
            
            // Animate elements on load
            const fadeElements = document.querySelectorAll('.fade-in');
            const slideElements = document.querySelectorAll('.slide-in');
            
            setTimeout(() => {{
                fadeElements.forEach(el => {{
                    el.style.opacity = '1';
                    el.style.transform = 'translateY(0)';
                }});
                
                slideElements.forEach(el => {{
                    el.style.opacity = '1';
                    el.style.transform = 'translateX(0)';
                }});
            }}, 100);
            
            // Load distribution chart
            setTimeout(() => {{
                createModernChart('distributionChart', {{
                    type: 'doughnut',
                    data: {{
                        labels: chartData.distribution.labels,
                        datasets: [{{
                            data: chartData.distribution.data,
                            backgroundColor: chartData.distribution.colors,
                            borderWidth: 0,
                            hoverBorderWidth: 2,
                            hoverBorderColor: '#ffffff'
                        }}]
                    }},
                    options: {{
                        plugins: {{
                            tooltip: {{
                                callbacks: {{
                                    label: function(context) {{
                                        const label = context.label || '';
                                        const value = context.parsed;
                                        const total = context.dataset.data.reduce((a, b) => a + b, 0);
                                        const percentage = ((value / total) * 100).toFixed(1);
                                        return `${{label}}: ${{value.toLocaleString()}} (${{percentage}}%)`;
                                    }}
                                }}
                            }}
                        }}
                    }}
                }});
            }}, 200);
            
            // Load complexity chart
            setTimeout(() => {{
                createModernChart('complexityChart', {{
                    type: 'bar',
                    data: {{
                        labels: chartData.complexity.labels,
                        datasets: [{{
                            label: 'Number of Functions',
                            data: chartData.complexity.data,
                            backgroundColor: chartData.complexity.colors,
                            borderRadius: 8,
                            borderSkipped: false
                        }}]
                    }},
                    options: {{
                        plugins: {{
                            legend: {{ display: false }},
                            tooltip: {{
                                callbacks: {{
                                    label: function(context) {{
                                        return `${{context.parsed.y}} functions`;
                                    }}
                                }}
                            }}
                        }},
                        scales: {{
                            y: {{
                                beginAtZero: true,
                                grid: {{
                                    color: 'rgba(0, 0, 0, 0.05)'
                                }},
                                ticks: {{
                                    font: {{
                                        family: 'Inter'
                                    }}
                                }}
                            }},
                            x: {{
                                grid: {{
                                    display: false
                                }},
                                ticks: {{
                                    font: {{
                                        family: 'Inter'
                                    }}
                                }}
                            }}
                        }}
                    }}
                }});
            }}, 400);
            
            // Load language chart
            setTimeout(() => {{
                createModernChart('languageChart', {{
                    type: 'bar',
                    data: {{
                        labels: chartData.language.labels,
                        datasets: [{{
                            label: 'Lines of Code',
                            data: chartData.language.data,
                            backgroundColor: chartData.language.colors,
                            borderRadius: 8,
                            borderSkipped: false
                        }}]
                    }},
                    options: {{
                        indexAxis: 'y',
                        plugins: {{
                            legend: {{ display: false }},
                            tooltip: {{
                                callbacks: {{
                                    label: function(context) {{
                                        return `${{context.parsed.x.toLocaleString()}} lines`;
                                    }}
                                }}
                            }}
                        }},
                        scales: {{
                            x: {{
                                beginAtZero: true,
                                grid: {{
                                    color: 'rgba(0, 0, 0, 0.05)'
                                }},
                                ticks: {{
                                    font: {{
                                        family: 'Inter'
                                    }}
                                }}
                            }},
                            y: {{
                                grid: {{
                                    display: false
                                }},
                                ticks: {{
                                    font: {{
                                        family: 'Inter'
                                    }}
                                }}
                            }}
                        }}
                    }}
                }});
            }}, 600);
        }});
        
        // Performance monitoring
        window.addEventListener('load', function() {{
            const loadTime = performance.now();
            console.log(`Report loaded in ${{loadTime.toFixed(2)}}ms`);
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
            self.get_progress_class(aggregated_stats.complexity.quality_metrics.code_health_score),
            aggregated_stats.complexity.quality_metrics.code_health_score,
            
            self.get_quality_class(aggregated_stats.complexity.quality_metrics.maintainability_index),
            aggregated_stats.complexity.quality_metrics.maintainability_index,
            self.get_progress_class(aggregated_stats.complexity.quality_metrics.maintainability_index),
            aggregated_stats.complexity.quality_metrics.maintainability_index,
            
            self.get_quality_class(aggregated_stats.ratios.quality_metrics.readability_score),
            aggregated_stats.ratios.quality_metrics.readability_score,
            self.get_progress_class(aggregated_stats.ratios.quality_metrics.readability_score),
            aggregated_stats.ratios.quality_metrics.readability_score,
            
            self.get_quality_class(aggregated_stats.ratios.quality_metrics.documentation_score),
            aggregated_stats.ratios.quality_metrics.documentation_score,
            self.get_progress_class(aggregated_stats.ratios.quality_metrics.documentation_score),
            aggregated_stats.ratios.quality_metrics.documentation_score,
            
            // Insights and recommendations
            self.template_generator.generate_enhanced_insights(aggregated_stats),
            self.template_generator.generate_enhanced_recommendations(aggregated_stats),
            
            // File analysis table
            self.template_generator.generate_extension_rows_with_real_analysis(aggregated_stats),
            
            // Individual files section - convert to modern grid format
            self.generate_modern_individual_files_section(individual_files),
            
            // Footer
            aggregated_stats.metadata.version,
            aggregated_stats.metadata.calculation_time_ms,
            
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
            aggregated_stats.complexity.complexity_distribution.very_high_complexity
        );
        
        Ok(html)
    }
    
    /// Get CSS class for quality score
    fn get_quality_class(&self, score: f64) -> &'static str {
        if score >= 80.0 {
            "score-excellent"
        } else if score >= 60.0 {
            "score-good"
        } else {
            "score-poor"
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
    
    fn generate_modern_individual_files_section(&self, individual_files: &[(String, FileStats)]) -> String {
        if individual_files.is_empty() {
            return r#"<div class="file-item">
                <div class="file-name">No individual files to display</div>
                <div class="file-metrics">
                    <span class="file-metric">Analysis complete</span>
                </div>
            </div>"#.to_string();
        }
        
        let mut section = String::with_capacity(individual_files.len() * 300);
        
        // Sort files by a combination of size and complexity for better insights
        let mut sorted_files: Vec<_> = individual_files.iter().collect();
        sorted_files.sort_by(|a, b| {
            let score_a = (a.1.total_lines as f64 * 0.6) + (a.1.code_lines as f64 * 0.4);
            let score_b = (b.1.total_lines as f64 * 0.6) + (b.1.code_lines as f64 * 0.4);
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        // Show top 15 files to keep the report manageable
        for (file_path, file_stats) in sorted_files.iter().take(15) {
            let complexity_estimate = self.estimate_file_complexity_score(file_stats);
            let complexity_class = if complexity_estimate > 7.0 { "complexity-high" } 
                                  else if complexity_estimate > 4.0 { "complexity-medium" } 
                                  else { "complexity-low" };
            
            let file_name = self.shorten_file_path(file_path);
            
            section.push_str(&format!(
                r#"<div class="file-item">
                    <div class="file-name">{}</div>
                    <div class="file-metrics">
                        <span class="file-metric">Lines: {}</span>
                        <span class="file-metric">Code: {}</span>
                        <span class="file-metric">Comments: {}</span>
                        <span class="file-metric complexity-badge {}">Risk: {}</span>
                    </div>
                </div>"#,
                file_name,
                file_stats.total_lines,
                file_stats.code_lines,
                file_stats.comment_lines,
                complexity_class,
                if complexity_estimate > 7.0 { "HIGH" } 
                else if complexity_estimate > 4.0 { "MEDIUM" } 
                else { "LOW" }
            ));
        }
        
        section
    }
    
    fn estimate_file_complexity_score(&self, file_stats: &FileStats) -> f64 {
        let mut complexity: f64 = 1.0;
        
        // Size-based complexity
        if file_stats.total_lines > 500 {
            complexity += 3.0;
        } else if file_stats.total_lines > 200 {
            complexity += 1.5;
        }
        
        // Code density
        let code_ratio = if file_stats.total_lines > 0 {
            file_stats.code_lines as f64 / file_stats.total_lines as f64
        } else { 0.0 };
        
        if code_ratio > 0.8 {
            complexity += 2.0;
        } else if code_ratio > 0.6 {
            complexity += 1.0;
        }
        
        // Comment ratio (lower comments = higher complexity)
        let comment_ratio = if file_stats.total_lines > 0 {
            file_stats.comment_lines as f64 / file_stats.total_lines as f64
        } else { 0.0 };
        
        if comment_ratio < 0.05 {
            complexity += 1.5;
        } else if comment_ratio < 0.1 {
            complexity += 0.5;
        }
        
        complexity.min(10.0)
    }
    
    fn shorten_file_path(&self, path: &str) -> String {
        if path.len() <= 50 {
            return path.to_string();
        }
        
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() > 2 {
            format!(".../{}/{}", parts[parts.len() - 2], parts[parts.len() - 1])
        } else {
            let truncated: String = path.chars().take(47).collect();
            format!("{}...", truncated)
        }
    }
} 