# Sphinx configuration for torvox documentation
# Extensions: Sphinx-Needs (requirements mgmt) + CodeLinks (source tracing)

import os
import sys

extensions = [
    "sphinx_needs",
    "sphinx_codelinks",
]

# Project info
project = "torvox"
copyright = "2026, torvox contributors"
version = "0.1.0"
release = "0.1.0"

# HTML output
html_theme = "sphinx_rtd_theme"
html_static_path = []

# Sphinx-Needs: load need types, link types, and ID config from ubproject.toml
needs_from_toml = "../ubproject.toml"

# CodeLinks: load source trace config from ubproject.toml
src_trace_config_from_toml = "../ubproject.toml"

# Traceability matrix
needs_flow_matrix = True
needs_flow_show_links = True

# Suppress sphinx-needs deprecation warning for needs_extra_links (TOML config)
suppress_warnings = ["needs.deprecated"]
