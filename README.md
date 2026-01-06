# ⚡ Clink

### A fast, modular shell prompt for Linux written in Rust — with asynchronous Git updates.

> **Note:** This is a personal project used to explore Rust programming concepts, modular design, and CLI optimization. I welcome any Pull Requests, suggestions, or improvements. Feel free to open issues or submit PRs!

---

## 🚀 Features

- **Customizable Modular Layout:** Define exactly what you want to see and in what order via a simple array.
- **Built-in & Custom Modules:** Use system-aware modules (Git, PWD, Permissions) or define your own static decorators.
- **Asynchronous Git Updates:** A background worker ensures "Ahead/Behind" stats are always current without network lag in your terminal.
- **Smart Logic:** Context-aware modules like `permission` only appear when relevant (e.g., in Read-Only directories).
- **Zero Ghosting:** Implements internal ANSI wrapping to ensure shell width math and cursor position are always perfect.

---

## 🔧 Module Reference & Configuration

The prompt is built by concatenating modules in the order they appear in the `layout` array within `~/.config/clink/config.toml`.

### Color Palette

Clink supports a full range of 16-color ANSI names. Use these names in your configuration (case-insensitive):

| Standard     | Bright Variant                  |
| :----------- | :------------------------------ |
| `black`      | `grey` or `gray` (Bright Black) |
| `red`        | `redbright`                     |
| `green`      | `greenbright`                   |
| `yellow`     | `yellowbright`                  |
| `blue`       | `bluebright`                    |
| `magenta`    | `magentabright`                 |
| `cyan`       | `cyanbright`                    |
| `white`      | `whitebright`                   |
| `foreground` | (Terminal Default)              |

---

## 🛰 Advanced: Git Caching & Background Fetch

Clink uses a **Split-Logic** approach to keep your prompt snappy, even in massive repositories.

### 1. The Rust Prompt (The Writer)

Every time a Git repository is rendered, Clink records the repository path and a timestamp to `~/.cache/clink`. It uses a **File Lock (flock)** to ensure that multiple open terminal windows do not conflict while writing to the cache file.

### 2. The Python Updater (The Heavy Lifter)

The provided `update_cache.py` script runs in the background to handle network-heavy tasks:

- **Deduplication:** Ensures each repository is only fetched once per cycle.
- **Pruning:** Cleans outdated or redundant entries from the cache file.
- **Background Fetch:** Executes `git fetch` for every unique repository found in the cache.

### 3. Automated Setup

To get real-time "Ahead/Behind" data, the updater script should be scheduled via a cron job (this is handled automatically by `make install`).

---

## 📥 Installation

### Option 1: Automated (Recommended)

1. **Install Rust:**
   `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
2. **Clone the repository.**
3. **Use the provided Makefile:**
   `make install`

**The Makefile performs the following:**

- **Binary:** Compiles and installs the `clink` binary to your path.
- **Config:** Installs `~/.config/clink/config.toml` (defines layout and styles).
- **Cache:** Initializes `~/.cache/clink/cache` (stores repo timestamps).
- **Automation:** Installs `update_cache.py` and sets a **Cronjob** to run every 10 minutes.

### Option 2: Manual Setup

1. Download the latest release.
2. Copy `update_cache.py` and `config.toml` to `~/.config/clink/`.
3. Move the compiled `clink` binary into a directory in your **PATH** (e.g., `~/bin` or `/usr/local/bin`).
4. Create the cache file: `mkdir -p ~/.cache/clink && touch ~/.cache/clink/cache`.
5. Manually set the cron job:
   `*/10 * * * * /usr/bin/python3 ~/.config/clink/update_cache.py`

---

## 🛠 Makefile Commands

- `make build` – Compiles and installs **only the binary**. Use this for development.
- `make install` – Full installation (Binary, Config, Cache, and Cronjob).
- `make uninstall` – Removes everything: the cron job, binary, and all configurations.
- `make release` – Optimized build intended for CI/CD packaging.

---

## 🐚 Activating Clink

Add the following line to the end of your `~/.bashrc`:

```bash
PS1='$(clink)'
```
