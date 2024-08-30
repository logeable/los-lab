qemu: user
	make -C los qemu

.PHONY: user
user:
	make -C tools user

clean:
	cd los; cargo clean
	cd user; cargo clean
	cd tools; cargo clean