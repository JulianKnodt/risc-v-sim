FILE ?= add.asm

bin:
	@riscv64-unknown-elf-as -march=rv32i -o $(FILE).elf $(FILE)
	@riscv64-unknown-elf-objcopy $(FILE).elf -j .text -O binary $(FILE).bin
	@rm $(FILE).elf

clean:
	rm *.out
