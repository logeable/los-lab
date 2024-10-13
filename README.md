# los is a OS demo inspired by rCore

```shell
❯ make qemu
cargo build
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.08s
riscv64-unknown-elf-objcopy --strip-all target/riscv64gc-unknown-none-elf/debug/los -O binary  target/riscv64gc-unknown-none-elf/debug/los.bin
qemu-system-riscv64 -machine virt -smp cores=4 -nographic -bios target/rustsbi-qemu.bin -device loader,file=target/riscv64gc-unknown-none-elf/debug/los,addr=0x80200000
[rustsbi] RustSBI version 0.4.0-alpha.1, adapting to RISC-V SBI v2.0.0
.______       __    __      _______.___________.  _______..______   __
|   _  \     |  |  |  |    /       |           | /       ||   _  \ |  |
|  |_)  |    |  |  |  |   |   (----`---|  |----`|   (----`|  |_)  ||  |
|      /     |  |  |  |    \   \       |  |      \   \    |   _  < |  |
|  |\  \----.|  `--'  |.----)   |      |  |  .----)   |   |  |_)  ||  |
| _| `._____| \______/ |_______/       |__|  |_______/    |______/ |__|
[rustsbi] Implementation     : RustSBI-QEMU Version 0.2.0-alpha.3
[rustsbi] Platform Name      : riscv-virtio,qemu
[rustsbi] Platform SMP       : 4
[rustsbi] Platform Memory    : 0x80000000..0x88000000
[rustsbi] Boot HART          : 0
[rustsbi] Device Tree Region : 0x87e00000..0x87e01cb0
[rustsbi] Firmware Address   : 0x80000000
[rustsbi] Supervisor Address : 0x80200000
[rustsbi] pmp01: 0x00000000..0x80000000 (-wr)
[rustsbi] pmp02: 0x80000000..0x80200000 (---)
[rustsbi] pmp03: 0x80200000..0x88000000 (xwr)
[rustsbi] pmp04: 0x88000000..0x00000000 (-wr)
memory    : [0x80000000..0x88000000]
kernel    : [0x80200000..0x80da6000]
.text     : [0x80200000..0x8023a000]
.rodata   : [0x8023a000..0x80243000]
.data     : [0x80243000..0x80ba5000]
.btstack  : [0x80ba5000..0x80ca5000]
.bss      : [0x80ca5000..0x80da6000]
0: add
1: gettimeofday
2: hello_world
3: init
4: loop_print_a
5: loop_print_b
6: lshell
7: priviledge_inst
8: stack
welcome to lshell
[2] >> add
1 + 2 = 3
[2] >> lshell
welcome to lshell
[3] >> stack
pc:0x10006c
ra:0x100052 fp:0x127fb0
ra:0x100038 fp:0x127fc0
ra:0x10045a fp:0x127fd0
ra:0x100020 fp:0x127ff0
ra:0x104fe6 fp:0x128000
[TRAP] load page fault: 0x131898 0x1001e2 TrapContext { regs: [0, 1049548, 1211280, 0, 0, 32, 1, 0, 1212336, 0, 1251480, 1251480, 1, 1, 2, 1052840, 1210816, 64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1107938, 0, 0], sstatus: Sstatus { bits: 8589934592 }, sepc: 1049058, kernel_satp: 9223372036855303590, kernel_sp: 18446744073709268992, trap_handler: 2149755848 }
subprocess stack(4) exited with -1
```
