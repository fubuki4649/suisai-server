BINARY_NAME  := suisai
PREFIX       ?= /usr/local
INSTALL_DIR  := $(PREFIX)/bin
CONFIG_DIR   := /etc/$(BINARY_NAME)
ENV_FILE     := $(CONFIG_DIR)/suisai.env
SYSTEMD_DIR  := /etc/systemd/system
SERVICE_FILE := $(SYSTEMD_DIR)/$(BINARY_NAME).service

SUDO         ?= sudo

all: build

build:
	cargo build --release

install: build
	@if [ ! -f .env ]; then \
		echo "Error: .env not found. Please copy example.env to .env and configure it before installing." >&2; \
		exit 1; \
	fi
	@if [ ! -f $(BINARY_NAME).service ]; then \
		echo "Error: $(BINARY_NAME).service not found." >&2; \
		exit 1; \
	fi
	$(SUDO) install -d $(INSTALL_DIR)
	$(SUDO) install -m 755 target/release/$(BINARY_NAME) $(INSTALL_DIR)/$(BINARY_NAME)
	$(SUDO) install -d $(CONFIG_DIR)
	$(SUDO) install -m 640 .env $(ENV_FILE)
	$(SUDO) install -m 644 $(BINARY_NAME).service $(SERVICE_FILE)
	$(SUDO) systemctl daemon-reload
	@echo "Installed $(BINARY_NAME) successfully."
	@echo "To enable and start: $(SUDO) systemctl enable --now $(BINARY_NAME)"

uninstall:
	-$(SUDO) systemctl disable --now $(BINARY_NAME) 2>/dev/null
	$(SUDO) rm -f $(SERVICE_FILE)
	$(SUDO) systemctl daemon-reload
	$(SUDO) rm -f $(INSTALL_DIR)/$(BINARY_NAME)
	$(SUDO) rm -rf $(CONFIG_DIR)
	@echo "Uninstalled $(BINARY_NAME)."

clean:
	cargo clean

.PHONY: all build install uninstall clean
