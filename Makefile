# Attempt to load a config.make file.
# If none is found, project defaults in config.project.make will be used.
ifneq ($(wildcard config.make),)
	include config.make
endif

# make sure the the OF_ROOT location is defined
ifndef OF_ROOT
	OF_ROOT=$(realpath ../../..)
endif

# call the project makefile!
include $(OF_ROOT)/libs/openFrameworksCompiled/project/makefileCommon/compile.project.mk

# ==============================================================================
# CUSTOM ICON INSTALLATION (macOS only)
# ==============================================================================
# To install the custom icon automatically after building, use:
#   make && make icon
#
# This replaces the default of.icns with your custom icon from bin/data/icon.icns
# ==============================================================================

# Target to copy icon after building (macOS only)
.PHONY: icon

icon:
ifeq ($(shell uname),Darwin)
	@echo "Installing custom icon..."
	@if [ -f "bin/data/icon.icns" ]; then \
		mkdir -p "bin/$(APPNAME).app/Contents/Resources"; \
		cp "bin/data/icon.icns" "bin/$(APPNAME).app/Contents/Resources/of.icns"; \
		touch "bin/$(APPNAME).app"; \
		rm -f "bin/$(APPNAME).app/Contents/Resources/icon.icns"; \
		echo "✓ Custom icon installed: bin/$(APPNAME).app/Contents/Resources/of.icns"; \
		echo "  (Replaced default of.icns)"; \
		echo ""; \
		echo "Note: If icon doesn't appear immediately, try:"; \
		echo "  1. Right-click app -> Get Info"; \
		echo "  2. Or run: rm -rf ~/Library/Caches/com.apple.finder/"; \
		echo "  3. Or restart Finder: killall Finder"; \
	else \
		echo "! No custom icon found at bin/data/icon.icns"; \
		echo "  Place your icon.icns file there and run 'make icon'"; \
	fi
else
	@echo "Icon installation is only supported on macOS"
endif
