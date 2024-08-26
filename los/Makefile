BINTOOLS_PREFIX = riscv64-unknown-elf-
RUSTSBI_QEMU = ../target/rustsbi-qemu.bin
KERNEL = los
KERNEL_BIN = ${KERNEL}.bin
TARGET_DIR = ../target/riscv64gc-unknown-none-elf/release/
FULL_KERNEL = ${TARGET_DIR}${KERNEL}
FULL_KERNEL_BIN = ${TARGET_DIR}${KERNEL_BIN}

QEMU = qemu-system-riscv64
KERNEL_BASE_ADDRESS = 0x80200000
SMP = 4

qemu_opts = \
	-machine virt \
	-smp cores=${SMP} \
	-nographic \
	-bios ${RUSTSBI_QEMU} \
	-device loader,file=${FULL_KERNEL},addr=${KERNEL_BASE_ADDRESS}

qemu: ${KERNEL_BIN} 
	${QEMU} ${qemu_opts}

gdb: ${KERNEL_BIN}
	${QEMU} ${qemu_opts} -s -S &
	@sleep 1
	riscv64-elf-gdb \
		-ex 'file ${FULL_KERNEL}' \
		-ex 'set arch riscv:rv64' \
		-ex 'target remote 127.0.0.1:1234' \
		-ex 'b *${KERNEL_BASE_ADDRESS}'
	pkill -l -n ${QEMU} 


${KERNEL}:
	cargo build --release

${KERNEL_BIN}: ${KERNEL}
	${BINTOOLS_PREFIX}objcopy --strip-all ${FULL_KERNEL} -O binary  ${FULL_KERNEL_BIN}

${FULL_KERNEL}: ${KERNEL}

${FULL_KERNEL_BIN}: ${KERNEL_BIN}

readelf: ${FULL_KERNEL}
	${BINTOOLS_PREFIX}readelf -h $<
	${BINTOOLS_PREFIX}readelf -S $<

objdump: ${FULL_KERNEL}
	${BINTOOLS_PREFIX}objdump -D $<

objdump-bin: ${FULL_KERNEL_BIN}
	${BINTOOLS_PREFIX}objdump -m riscv -b binary -D $<

hexdump-bin: ${FULL_KERNEL_BIN}
	hexdump -C $<
