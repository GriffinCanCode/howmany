use regex::Regex;

pub struct PythonPatterns {
    external_patterns: Vec<Regex>,
    cache_patterns: Vec<Regex>,
    extensions: Vec<String>,
}

impl PythonPatterns {
    pub fn new() -> Self {
        let external_patterns = vec![
            // Python bytecode
            Regex::new(r"__pycache__").unwrap(),
            Regex::new(r"\.pyc$").unwrap(),
            Regex::new(r"\.pyo$").unwrap(),
            Regex::new(r"\.pyd$").unwrap(),
            
            // Package installations
            Regex::new(r"site-packages").unwrap(),
            Regex::new(r"dist-packages").unwrap(),
            Regex::new(r"\.egg-info").unwrap(),
            Regex::new(r"\.dist-info").unwrap(),
            Regex::new(r"\.whl$").unwrap(),
            Regex::new(r"\.egg$").unwrap(),
            
            // Virtual environments
            Regex::new(r"\.venv/").unwrap(),
            Regex::new(r"venv/").unwrap(),
            Regex::new(r"env/").unwrap(),
            Regex::new(r"\.env/").unwrap(),
            Regex::new(r"virtualenv/").unwrap(),
            
            // Conda environments
            Regex::new(r"\.conda/").unwrap(),
            Regex::new(r"conda-meta/").unwrap(),
            Regex::new(r"envs/").unwrap(),
            Regex::new(r"pkgs/").unwrap(),
            
            // Build directories
            Regex::new(r"build/").unwrap(),
            Regex::new(r"dist/").unwrap(),
            Regex::new(r"\.build/").unwrap(),
            Regex::new(r"\.dist/").unwrap(),
        ];

        let cache_patterns = vec![
            // Testing caches
            Regex::new(r"\.pytest_cache").unwrap(),
            Regex::new(r"\.tox/").unwrap(),
            Regex::new(r"\.nox/").unwrap(),
            Regex::new(r"\.coverage").unwrap(),
            Regex::new(r"htmlcov/").unwrap(),
            Regex::new(r"\.coverage\..*").unwrap(),
            
            // Type checking caches
            Regex::new(r"\.mypy_cache").unwrap(),
            Regex::new(r"\.pytype/").unwrap(),
            Regex::new(r"\.pyre/").unwrap(),
            Regex::new(r"\.dmypy\.json").unwrap(),
            
            // Linting and formatting caches
            Regex::new(r"\.ruff_cache").unwrap(),
            Regex::new(r"\.black").unwrap(),
            Regex::new(r"\.pylint\.d/").unwrap(),
            Regex::new(r"\.flake8").unwrap(),
            
            // Jupyter notebook caches
            Regex::new(r"\.ipynb_checkpoints").unwrap(),
            Regex::new(r"\.jupyter/").unwrap(),
            
            // Documentation build caches
            Regex::new(r"\.sphinx/").unwrap(),
            Regex::new(r"docs/_build/").unwrap(),
            Regex::new(r"docs/build/").unwrap(),
            Regex::new(r"\.doctrees/").unwrap(),
            
            // Profiling and debugging
            Regex::new(r"\.prof").unwrap(),
            Regex::new(r"\.profile").unwrap(),
            Regex::new(r"\.pstats").unwrap(),
            
            // Packaging caches
            Regex::new(r"\.eggs/").unwrap(),
            Regex::new(r"\.cache/").unwrap(),
            Regex::new(r"\.pip/").unwrap(),
            
            // Development server caches
            Regex::new(r"\.django_cache/").unwrap(),
            Regex::new(r"\.flask_session/").unwrap(),
            
            // Machine learning caches
            Regex::new(r"\.wandb/").unwrap(),
            Regex::new(r"\.mlflow/").unwrap(),
            Regex::new(r"\.tensorboard/").unwrap(),
        ];

        let extensions = vec![
            // Python source files
            "py".to_string(), "pyw".to_string(), "pyi".to_string(),
            "pyx".to_string(), "pxd".to_string(), "pxi".to_string(),
            
            // Jupyter notebooks
            "ipynb".to_string(),
            
            // Python templates
            "pyt".to_string(), "pth".to_string(),
            
            // Web frameworks
            "wsgi".to_string(), "asgi".to_string(),
            
            // Configuration files commonly used in Python projects
            "cfg".to_string(), "ini".to_string(),
        ];

        Self {
            external_patterns,
            cache_patterns,
            extensions,
        }
    }

    pub fn get_external_patterns(&self) -> &[Regex] {
        &self.external_patterns
    }

    pub fn get_cache_patterns(&self) -> &[Regex] {
        &self.cache_patterns
    }

    pub fn get_extensions(&self) -> &[String] {
        &self.extensions
    }

    pub fn get_script_names() -> Vec<&'static str> {
        vec![
            // Package management
            "requirements.txt", "requirements-dev.txt", "requirements-test.txt",
            "dev-requirements.txt", "test-requirements.txt",
            "setup.py", "setup.cfg", "pyproject.toml", "poetry.lock",
            "Pipfile", "Pipfile.lock", "pipenv.lock",
            "environment.yml", "environment.yaml", "conda.yaml",
            "conda-requirements.txt", "environment-dev.yml",
            
            // Modern Python packaging
            "pyproject.toml", "setup.cfg", "MANIFEST.in",
            "flit.ini", "pdm.lock", "poetry.toml",
            
            // Testing configuration
            "tox.ini", "pytest.ini", "pytest.cfg",
            "nose.cfg", "nose2.cfg", "unittest.cfg",
            "conftest.py", "test_*.py", "*_test.py",
            ".coveragerc", "coverage.ini",
            
            // Code quality and linting
            "mypy.ini", ".mypy.ini", "mypy.cfg",
            ".flake8", ".pylintrc", "pylint.cfg",
            ".bandit", "bandit.yaml", "bandit.yml",
            ".pre-commit-config.yaml", ".pre-commit-hooks.yaml",
            "pycodestyle.cfg", "pep8.cfg",
            "ruff.toml", ".ruff.toml",
            
            // Documentation
            "conf.py", "make.bat", "Makefile",
            "docs/conf.py", "doc/conf.py",
            "sphinx.cfg", "readthedocs.yml", ".readthedocs.yml",
            
            // Web frameworks
            "manage.py", "wsgi.py", "asgi.py",
            "app.py", "main.py", "run.py",
            "flask_app.py", "django_app.py",
            "fastapi_app.py", "starlette_app.py",
            "gunicorn.conf.py", "uwsgi.ini",
            
            // Data science and ML
            "jupyter_notebook_config.py", "ipython_config.py",
            "notebook.json", "lab.json",
            "requirements-gpu.txt", "requirements-cpu.txt",
            
            // Deployment and containerization
            "Dockerfile", "docker-compose.yml", "docker-compose.yaml",
            "Procfile", "runtime.txt", "app.yaml",
            "serverless.yml", "serverless.yaml",
            
            // CI/CD
            ".github/workflows/python.yml", ".gitlab-ci.yml",
            "azure-pipelines.yml", "Jenkinsfile",
            "noxfile.py", "tasks.py",
            
            // Database
            "alembic.ini", "migrations/", "models.py",
            "database.py", "db.py", "schema.py",
            
            // Configuration
            "config.py", "settings.py", "local_settings.py",
            ".env", ".env.local", ".env.example",
            "logging.conf", "logging.ini",
        ]
    }
} 