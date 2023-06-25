# FileName
BIN=sd_importer

# Carpeta de destino
DEST=/usr/local/bin

# By default will build and install
all: build install

# Compile the sofware using the release flag
build: 
	@cargo build --release

# Install the package by coping to the release
install:
	@sudo cp target/release/$(BIN) $(DEST)
