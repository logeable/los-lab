qemu:
	cd los-user; cargo build --release
	cargo xtask app build
	cargo xtask app asm
	make -C los qemu

gdb:
	make -C los gdb