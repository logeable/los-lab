qemu:
	make -C tools user
	make -C los qemu

clean:
	cd los; cargo clean
	cd user; cargo clean
	cd tools; cargo clean