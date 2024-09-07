qemu: user
	make -C los qemu

gdb:
	make -C los gdb BUILD_MODE=debug

.PHONY: user
user:
	make -C tools user

clean:
	cd los; cargo clean
	cd user; cargo clean
	cd tools; cargo clean
