#!/bin/sh

CODE=/usr/share/OVMF/x64/OVMF_CODE.fd
VARS=/usr/share/OVMF/x64/OVMF_VARS.fd

cargo build --target x86_64-unknown-uefi
mkdir -p esp/efi/boot
cp target/x86_64-unknown-uefi/debug/boid.efi esp/efi/boot/bootx64.efi

qemu-system-x86_64 -nodefaults						\
		   -device virtio-rng-pci				\
		   -machine q35						\
		   -smp 4						\
		   -m 1024M						\
		   -vga std						\
		   --enable-kvm						\
		   -device isa-debug-exit,iobase=0xf4,iosize=0x04	\
		   -drive if=pflash,format=raw,readonly=on,file=$CODE	\
		   -drive if=pflash,format=raw,readonly=on,file=$VARS	\
		   -drive format=raw,file=fat:rw:esp			\
		   -serial stdio
