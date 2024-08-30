qemu: user
	make -C los qemu

.PHONY: user
user:
	make -C tools user
	make -C los qemu

clean:
	cd los; cargo clean
	cd user; cargo clean
	cd tools; cargo clean