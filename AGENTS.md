# AGENTS.md — Stable Diffusion WebUI

Guidance for agentic coding agents working in this repository (kojeomstudio fork of AUTOMATIC1111/stable-diffusion-webui).

## Project Overview

A browser-based frontend for Stable Diffusion image generation. Python backend (Gradio + FastAPI), JavaScript frontend. Primary language: Python 3.10.

- **Original upstream**: https://github.com/AUTOMATIC1111/stable-diffusion-webui
- **This fork**: https://github.com/kojeomstudio/stable-diffusion-webui

This fork has modified dependency URLs pointing to `kojeomstudio/` GitHub repos (see `modules/launch_utils.py`) because some original upstream dependency repos have been deleted or archived.

## Security — DO NOT

- **NEVER commit** files in `.gitignore`: `config.json`, `ui-config.json`, `models/`, `outputs/`, `embeddings/`, `styles.csv`, `*.ckpt`, `*.safetensors`, `*.pth`, `venv/`.
- **NEVER expose** secrets, API keys, tokens, or credentials in code or comments.
- **NEVER read or log** contents of `config.json`, `ui-config.json`, or user data in `outputs/`.
- **NEVER modify** `.gitignore` to allow tracking of model weights or user config files.
- Do not install packages directly from PyPI for packages that are managed by `launch_utils.py` (torch, clip, open_clip, xformers).

## Build / Lint / Test Commands

### Lint (Python)

```bash
pip install ruff==0.3.3
ruff .
```

### Lint (JavaScript)

```bash
npm i --ci
npm run lint          # check
npm run fix           # auto-fix
```

Both linters run in CI on every push/PR. **Your code must pass `ruff .` and `npm run lint` with zero errors before committing.**

### Tests

Test framework: **pytest** with `pytest-base-url` plugin.

```bash
# Install test dependencies
pip install -r requirements-test.txt

# Run all tests (requires a running server at http://127.0.0.1:7860)
python -m pytest -vv test

# Run a single test file
python -m pytest test/test_torch_utils.py -v

# Run a single test function
python -m pytest test/test_torch_utils.py::test_get_param -v

# Run a single parametrized case
python -m pytest "test/test_torch_utils.py::test_get_param[True]" -v

# Run with coverage
python -m pytest -vv --cov . --cov-report=term test
```

**Important:** Most API tests (`test_txt2img.py`, `test_img2img.py`, `test_extras.py`, `test_utils.py`) require the webui server running on `http://127.0.0.1:7860`. Only `test_torch_utils.py` and `test_face_restorers.py` can run standalone (they import modules directly without needing a server).

To start the test server:
```bash
python launch.py --skip-prepare-environment --skip-torch-cuda-test \
  --test-server --do-not-download-clip --no-half \
  --disable-opt-split-attention --use-cpu all --api-server-stop
```

## Repository Structure

```
launch.py              # Entry point: environment setup, dependency installation
webui.py               # Main application: creates Gradio UI and/or API server
modules/               # Core Python backend
  api/                 # FastAPI REST API (api.py, models.py)
  processing_scripts/  # Seed, sampler, refiner scripts
  hypernetworks/       # Hypernetwork training
  textual_inversion/   # Textual inversion training
  shared.py            # Global state container (opts, sd_model, device, etc.)
  errors.py            # Error reporting utilities
  processing.py        # Core image processing pipeline
  sd_models.py         # Model loading and checkpoint management
  images.py            # Image saving, grids, metadata
  devices.py           # PyTorch device management (CUDA, MPS, XPU, NPU)
  ui.py                # Gradio UI construction
scripts/               # User-facing automation scripts (xyz_grid, loopback, etc.)
javascript/            # Frontend JS modules (loaded by script.js)
test/                  # pytest test suite
extensions-builtin/    # 11 built-in extensions (Lora, LDSR, SwinIR, etc.)
extensions/            # User-installed extensions (excluded from linting)
html/                  # HTML templates
```

## Code Style — Python

### Imports

- No strict isort enforcement (I001 is ignored), but follow a logical order: stdlib, then third-party, then `modules.*`.
- Use `# noqa: F401` for intentionally unused imports.
- Use `from __future__ import annotations` where helpful for forward references.
- Guard circular imports with `TYPE_CHECKING`:

```python
from __future__ import annotations
import os
import sys
import torch
from modules import shared, errors
from modules.paths_internal import models_path
```

### Naming

- Functions and variables: `snake_case`
- Classes: `PascalCase`
- Module-level globals: `snake_case` (e.g., `sd_model`, `cmd_opts`, `startup_timer`)
- Constants: `UPPER_SNAKE_CASE` (e.g., `exception_records`)

### Types

- Gradual typing — some files use type hints, many older files do not.
- Prefer adding type hints to new code (return types especially: `-> str`, `-> bool`, `-> None`).
- Use `TYPE_CHECKING` guard for imports that would create circular dependencies.

### Error Handling

Use the project's custom error module (`modules/errors.py`):

```python
from modules import errors

# Report an error message to stderr
errors.report("something went wrong", exc_info=True)

# Display an exception with context
try:
    ...
except Exception as e:
    errors.display(e, "failed to load model")

# Display only once per task
errors.display_once(e, "loading lora")
```

Key error functions:
- `errors.report(message, *, exc_info=False)` — print to stderr with optional traceback
- `errors.display(e, task, *, full_traceback=False)` — formatted exception output
- `errors.display_once(e, task)` — same as display, but only once per task string
- `errors.print_error_explanation(message)` — boxed explanation message

Exceptions are recorded to `exception_records` (capped at 5). For API endpoints, use `fastapi.exceptions.HTTPException`.

### Formatting Rules (from ruff config)

- **No line length limit** (E501 ignored).
- Allow `lambda` assignment (E731 ignored).
- Allow type comparisons (E721 ignored).
- `webui.py` may have imports not at top of file (E402 ignored).
- Enabled rule sets: flake8-bugbear (B), complexity (C), isort (I), warnings (W).
- Do not compare types directly; use `isinstance()` in new code despite E721 being ignored.
- `extensions/` and `extensions-disabled/` are excluded from linting entirely.

### String Formatting

- Prefer f-strings in new code.
- `.format()` is used in older code; both are acceptable.

### Docstrings

- Triple-quote `"""` style when used.
- Not consistently applied across the codebase. Add docstrings to public APIs.

## Code Style — JavaScript

### ESLint Rules (from `.eslintrc.js`)

- **4-space indentation** (no tabs for indentation)
- **Semicolons required** always
- **Unix line endings** (LF)
- **No unused variable check** (`no-unused-vars: off`)
- Braces: `multi-line` and `consistent` style
- Object curly spacing: no spaces inside `{}`
- Arrow function spacing required around `=>`
- No trailing spaces
- Quote props: `consistent-as-needed`

### JS Patterns

- Mix of `var` (older code) and `const`/`let` (newer code). Prefer `const`/`let`.
- Functions use `function` keyword style in most files.
- Global variables are declared in `.eslintrc.js` globals section (e.g., `gradioApp`, `opts`, `localization`).
- Files in `javascript/` are loaded by `script.js` as Gradio JS components.

## Key Conventions

1. **Global state** lives in `modules/shared.py` (e.g., `shared.opts`, `shared.sd_model`, `shared.device`, `shared.state`). Access via `from modules import shared`.

2. **CLI arguments** are defined in `modules/cmd_args.py` and parsed in `modules/shared_cmd_options.py`. The `IGNORE_CMD_ARGS_ERRORS=1` env var is set during testing to prevent pytest args from colliding with webui arg parsing.

3. **Extensions** are loaded from `extensions-builtin/` and `extensions/`. They are excluded from linting. Extension code uses callback hooks from `modules/script_callbacks.py`.

4. **Device handling** goes through `modules/devices.py` — supports CUDA, MPS (Apple Silicon), XPU, and NPU. Always use `devices.get_optimal_device()` or `shared.device` rather than hardcoding `cuda`.

5. **Configuration** is stored in `config.json` and `ui-config.json`. Runtime options are in `shared.opts` (a `shared.Options` instance).

6. **Branch conventions**: PRs should target `dev`, not `master`. The CI warns about PRs targeting `master`.

7. **Dependencies**: Install via `launch.py` (handles PyTorch + all deps). Do not install directly from `requirements.txt` unless you understand the torch index URL requirements.

8. **Forked repositories**: Dependency repos point to `kojeomstudio/` forks (see `modules/launch_utils.py`). CLIP and OpenCLIP are installed from self-hosted zip archives in the `archive/` directory. Do not change these URLs back to original upstream without understanding the reason for the fork (some original repos are deleted or archived).
