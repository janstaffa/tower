### List and ammount of ICs required for the physical build

General:
- 10x 74xx04 (hex inverter)
Board 1(Clock, Interfaces):
- 74xx00 (quad 2 input nand gate)
- oscilator

Board 2(Memory):
- Memory:
	- 74xx30 (8 input NAND gate)
	- 2x 74xx245 (octal bus transceiver)
	- 74xx32 (quad 2 input or gate)
	- 74xx08 (quad 2 input and gate)
	- 2x 74xx157 (quad 2 line to 1 line multiplexer)
	- 64K static RAM
	- 16K EEPROM
- Program Counter:
	- 2x 74xx8161 (8 bit binary counter)
	- 2x 74xx245 (octal bus transceiver)
- Stack Pointer:
	- 74xx383 (8 bit register)
	- 74xx32 (quad 2 input or gate)
	- 2x 74xx245 (octal bus transceiver)
- Memory Registers:
	- 4x 74xx157 (quad 2 line to 1 line multiplexer)
	- 74xx32 (quad 2 input or gate)
	- 2x 74xx383 (8 bit register)
	- 4x 74xx245 (octal bus transceiver)


Board 3(Control logic):
- Argument Registers:
	- 2x 74xx383 (8 bit register)
	- 4x 74xx245 (octal bus transceiver)
- Instruction Register:
	- 74xx383 (8 bit register)
- Step counter:
	- 74xx161 (4 bit binary counter)
	- 74xx32 (quad 2 input or gate)
- Microcode:
	- 5x 32K EEPROM

	
Board 4(ALU):
- A and B registers:
	- 2x 74xx383 (8 bit register)
	- 74xx245 (octal bus transceiver)
- ALU:
	- 5x 74xx245 (octal bus transceiver)
	- 2x 74xx4078 (8 input nor gate)
	- 74xx240 (octal buffer, inverting outputs)
	- 2x 74xx00 (quad 2 input nand gate)
	- 2x 74xx283 (4-bit binary full adder)
	- 74xx86 (quad 2 input xor gate)
	- 2x 74xx157 (quad 2 line to 1 line multiplexer)
	- 2x 74xx04 (hex inverter)
	- 74xx08 (quad 2 input and gate)
	- 74xx32 (quad 2 input or gate)





