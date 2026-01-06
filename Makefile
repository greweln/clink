# =========================================================================
# ANSI Color and Style Definitions
# =========================================================================
# Define standard colors/styles for readable terminal output
BOLD := \033[1m
GREEN := \033[32m
YELLOW := \033[33m
RED := \033[31m
RESET := \033[0m

# =========================================================================
# Application Variables
# =========================================================================
APP_NAME=clink
CACHE_FILE=$(HOME)/.cache/$(APP_NAME)
CONFIG_SRC=config/default.toml
SCRIPT_SRC=scripts/update_cache.py
CONFIG_DEST=$(HOME)/.config/$(APP_NAME)
CONFIG_FILE=$(CONFIG_DEST)/config.toml
SCRIPT_FILE=$(CONFIG_DEST)/update_cache.py
BIN_DEST=$(HOME)/bin
PYTHON=/usr/bin/python3  # Adjust if python3 is elsewhere
VERSION := $(shell git describe --tags --always --dirty)
DIST_DIR = dist/$(APP_NAME)-$(VERSION)
MUSL_TARGET = x86_64-unknown-linux-musl

.PHONY: all install uninstall build release

all: install

# =========================================================================
# build Target: Compiles and places the binary
# =========================================================================
build:
	
	@echo -e  "$(BOLD)$(GREEN)Compiling binary ...$(RESET)"
	cargo build --release
	
	@echo -e "$(BOLD)$(GREEN)Creating $(BIN_DEST) ...$(RESET)"
	mkdir -p $(BIN_DEST)
	
	@echo -e "$(BOLD)$(GREEN)Deploying binary ...$(RESET)"
	cp -u target/release/$(APP_NAME) $(BIN_DEST)/$(APP_NAME)
	
# =========================================================================
# install Target: Sets up config, script, and cronjob
# =========================================================================
install: build
	@echo "$(BOLD)$(GREEN)Creating $(CONFIG_DEST) ...$(RESET)"
	mkdir -p $(CONFIG_DEST)
	
	@echo "$(BOLD)$(GREEN)Copying default config ...$(RESET)"
	cp -u $(CONFIG_SRC) $(CONFIG_FILE)
	
	@echo "$(BOLD)$(GREEN)Copying background script ...$(RESET)"
	cp -u $(SCRIPT_SRC) $(SCRIPT_FILE)
	chmod +x $(SCRIPT_FILE)
	
	@echo "$(BOLD)$(GREEN)Creating cache file ...$(RESET)"
	touch $(CACHE_FILE)

	@echo "$(BOLD)$(GREEN)Setting up cronjob ...$(RESET)"
	@(crontab -l 2>/dev/null | grep -v "$(APP_NAME)/update_cache.py"; \
	  echo "*/10 * * * * $(PYTHON) $(SCRIPT_FILE)") | crontab -
	
	@echo "$(BOLD)$(GREEN)Installation complete.$(RESET)"
	
# =========================================================================
# uninstall Target: Removes all installed components
# =========================================================================
uninstall:
	@echo "$(BOLD)$(GREEN)Removing cronjob ...$(RESET)"
	@crontab -l 2>/dev/null | grep -v "$(APP_NAME)/update_cache.py" | crontab - || crontab -r
	
	@echo "$(BOLD)$(GREEN)Removing config directory ...$(RESET)"
	rm -rf $(CONFIG_DEST)
	
	@echo "$(BOLD)$(GREEN)Removing binary ...$(RESET)"
	rm -f $(BIN_DEST)/$(APP_NAME)
	
	@echo "$(BOLD)$(GREEN)Removing cache file ...$(RESET)"
	rm -f $(CACHE_FILE)	
# =========================================================================
# release Target: Builds cross-compiled static binaries and creates tarballs
# =========================================================================
release:
	@echo -e "$(BOLD)$(GREEN)Building glibc release ...$(RESET)"
	cargo build --release
	
	@echo -e "$(BOLD)$(GREEN)Building MUSL static binary ...$(RESET)"
	# rustup target add $(MUSL_TARGET) >/dev/null 2>&1 || true
	cargo build --release --target $(MUSL_TARGET) --features musl
	
	@echo -e "$(BOLD)$(GREEN)Preparing Distribution Directories ...$(RESET)"
	rm -rf $(DIST_DIR)
	mkdir -p $(DIST_DIR)/$(APP_NAME)-glibc
	mkdir -p $(DIST_DIR)/$(APP_NAME)-musl
	
	@echo -e "$(BOLD)$(GREEN)Copying glibc files ...$(RESET)"
	cp target/release/$(APP_NAME) $(DIST_DIR)/$(APP_NAME)-glibc/
	cp $(CONFIG_SRC) $(DIST_DIR)/$(APP_NAME)-glibc/config.toml
	cp $(SCRIPT_SRC) $(DIST_DIR)/$(APP_NAME)-glibc/update_cache.py
	
	@echo -e "$(BOLD)$(GREEN)Copying musl files ...$(RESET)"
	cp target/$(MUSL_TARGET)/release/$(APP_NAME) $(DIST_DIR)/$(APP_NAME)-musl/
	cp $(CONFIG_SRC) $(DIST_DIR)/$(APP_NAME)-musl/config.toml
	cp $(SCRIPT_SRC) $(DIST_DIR)/$(APP_NAME)-musl/update_cache.py
	
	@echo -e "$(BOLD)$(GREEN)Creating tarballs ...$(RESET)"
	tar -czvf $(APP_NAME)-$(VERSION)-linux-x86_64-glibc.tar.gz -C $(DIST_DIR) $(APP_NAME)-glibc
	tar -czvf $(APP_NAME)-$(VERSION)-linux-x86_64-musl.tar.gz -C $(DIST_DIR) $(APP_NAME)-musl
	
	@echo -e "$(BOLD)$(GREEN)Release tarballs created:$(RESET)"
	@echo -e "$(APP_NAME)-$(VERSION)-linux-x86_64-glibc.tar.gz"
	@echo -e "$(APP_NAME)-$(VERSION)-linux-x86_64-musl.tar.gz"
