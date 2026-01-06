# =========================================================================
# ANSI Color and Style Definitions
# =========================================================================
BOLD := \033[1m
GREEN := \033[32m
YELLOW := \033[33m
RED := \033[31m
RESET := \033[0m

# =========================================================================
# Application Variables
# =========================================================================
APP_NAME=clink
VERSION := $(shell git describe --tags --always --dirty)

# Paths
BIN_DEST=$(HOME)/bin
CONFIG_DEST=$(HOME)/.config/$(APP_NAME)
CACHE_FILE=$(HOME)/.cache/$(APP_NAME)

# Source files
CONFIG_SRC=config/default.toml
SCRIPT_SRC=scripts/update_cache.py

# Target files
CONFIG_FILE=$(CONFIG_DEST)/config.toml
SCRIPT_FILE=$(CONFIG_DEST)/update_cache.py

# Tools
PYTHON=/usr/bin/python3
MUSL_TARGET = x86_64-unknown-linux-musl
DIST_DIR = dist/$(APP_NAME)-$(VERSION)

.PHONY: all install uninstall build release clean

all: build

# =========================================================================
# build Target: Compiles and places the binary locally (GLIBC)
# =========================================================================
build:
	@echo "$(BOLD)$(GREEN)Compiling binary (Debug/Local) ...$(RESET)"
	cargo build --release
	@mkdir -p $(BIN_DEST)
	@echo "$(BOLD)$(GREEN)Deploying binary to $(BIN_DEST) ...$(RESET)"
	cp -u target/release/$(APP_NAME) $(BIN_DEST)/$(APP_NAME)

# =========================================================================
# install Target: Sets up config, script, and cronjob
# =========================================================================
install: build
	@echo "$(BOLD)$(GREEN)Setting up configuration in $(CONFIG_DEST) ...$(RESET)"
	@mkdir -p $(CONFIG_DEST)
	cp -u $(CONFIG_SRC) $(CONFIG_FILE)
	cp -u $(SCRIPT_SRC) $(SCRIPT_FILE)
	chmod +x $(SCRIPT_FILE)
	
	@echo "$(BOLD)$(GREEN)Initializing cache file ...$(RESET)"
	@mkdir -p $(shell dirname $(CACHE_FILE))
	touch $(CACHE_FILE)

	@echo "$(BOLD)$(GREEN)Configuring cronjob (every 10 min) ...$(RESET)"
	# Remove any existing clink cronjob and add the new one using variables
	@(crontab -l 2>/dev/null | grep -v "$(APP_NAME)/update_cache.py"; \
	  echo "*/10 * * * * $(PYTHON) $(SCRIPT_FILE)") | crontab -
	
	@echo "$(BOLD)$(GREEN)Installation complete!$(RESET)"

# =========================================================================
# release Target: Builds portable glibc and MUSL binaries
# =========================================================================
# *  Local: Uses Podman + Cross (Requires: 
#    - cargo install cross --git https://github.com/cross-rs/cross
#    - dnf install podman openssl-devel
# * CI/CD: Uses Docker + Cross (GitHub Actions)
release:
	@echo "$(BOLD)$(YELLOW)Cleaning old release artifacts ...$(RESET)"
	rm -rf $(DIST_DIR)
	rm -rf target/cross
	
	@echo "$(BOLD)$(GREEN)Building GLIBC binary (Host) ...$(RESET)"
	cargo build --release
	
	@echo "$(BOLD)$(GREEN)Building MUSL binary (Container via Cross) ...$(RESET)"
	cross build --release --target $(MUSL_TARGET) --target-dir target/cross
	
	@echo "$(BOLD)$(GREEN)Preparing distribution folders ...$(RESET)"
	mkdir -p $(DIST_DIR)/$(APP_NAME)-glibc
	mkdir -p $(DIST_DIR)/$(APP_NAME)-musl
	
	@echo "$(BOLD)$(GREEN)Packaging binaries ...$(RESET)"
	cp target/release/$(APP_NAME) $(DIST_DIR)/$(APP_NAME)-glibc/
	cp target/cross/$(MUSL_TARGET)/release/$(APP_NAME) $(DIST_DIR)/$(APP_NAME)-musl/
	
	# Copy support files to both dist folders
	cp $(CONFIG_SRC) $(DIST_DIR)/$(APP_NAME)-glibc/config.toml
	cp $(SCRIPT_SRC) $(DIST_DIR)/$(APP_NAME)-glibc/update_cache.py
	cp $(CONFIG_SRC) $(DIST_DIR)/$(APP_NAME)-musl/config.toml
	cp $(SCRIPT_SRC) $(DIST_DIR)/$(APP_NAME)-musl/update_cache.py
	
	@echo "$(BOLD)$(GREEN)Creating tarballs ...$(RESET)"
	tar -czvf $(APP_NAME)-$(VERSION)-linux-x86_64-glibc.tar.gz -C $(DIST_DIR) $(APP_NAME)-glibc
	tar -czvf $(APP_NAME)-$(VERSION)-linux-x86_64-musl.tar.gz -C $(DIST_DIR) $(APP_NAME)-musl
	
	@echo -e "\n$(BOLD)$(YELLOW)Final Binary Sizes:$(RESET)"
	@ls -lh target/release/$(APP_NAME) | awk '{print "GLIBC: " $$5}'
	@ls -lh target/cross/$(MUSL_TARGET)/release/$(APP_NAME) | awk '{print "MUSL:  " $$5}'

# =========================================================================
# uninstall Target: Complete cleanup
# =========================================================================
uninstall:
	@echo "$(BOLD)$(RED)Removing cronjob ...$(RESET)"
	@crontab -l 2>/dev/null | grep -v "$(APP_NAME)/update_cache.py" | crontab - || crontab -r
	
	@echo "$(BOLD)$(RED)Deleting configuration and binary ...$(RESET)"
	rm -rf $(CONFIG_DEST)
	rm -f $(BIN_DEST)/$(APP_NAME)
	rm -f $(CACHE_FILE)
	@echo "$(BOLD)$(GREEN)Uninstallation complete.$(RESET)"

clean:
	cargo clean
	rm -rf dist/
	rm -f *.tar.gz
