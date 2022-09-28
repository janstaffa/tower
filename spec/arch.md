# Tower - architecture

**Features**

- 8-bit data BUS
- 16-bit address BUS
- 16K ROM and ~48K RAM
- 256B zero page
- Turing complete machine
- Custom instruction set

**Description**

The Tower is a simple computer architecture by design, made for academic purposes of gaining deeper knowledge of how computers really work. It's simple design allows people with very little knowledge of computer science to understand the basics of how modern computers work.

---

**Table of contents**

1. Internal structure

2. Registers

3. Arithmetic Logic Unit

4. Control logic

5. Interfaces

6. Memory

7. Instruction set architecture

8. Programming

## 1. Internal structure

Tower has two buses, the data bus and the address bus. The data bus is 8 bits in size and the address bus is 16 bits. Addresses are naturally 16 bits long allowing a maximum of 64Ki memory locations each one byte in size. The CPU does not support memory banking. The full address range is split between the ROM and RAM chips. All data is stored and transferred in big endian order.

### Unit size definitions

| **unit** | **size(in bits)** |
| -------- | ----------------- |
| byte     | 8                 |
| word     | 8                 |
| dword    | 16                |

## 2. Registers

The Tower uses multiple types of registers for different purposes, however the programmer has direct access to only a few of them. Those being the A and B registers used as inputs to the ALU as well as general purpose registers, these registers can be accessed directly from code. The rest of the registers are used for internal function of the computer and thus are managed by the computer itself. The programmer has some level of indirect control of these registers.

##

### Accessible registers

| **name** | **description**                                                                                     | **size(b)** |
| -------- | --------------------------------------------------------------------------------------------------- | ----------- |
| **A**    | Accumulator - stores the result of the ALU, one of the inputs to the ALU, general purpose register. | **8**       |
| **B**    | Input to the ALU, general purpose register.                                                         | **8**       |

##

### Internal registers

| **name**             | **description**                                                                   | **size(b)** |
| -------------------- | --------------------------------------------------------------------------------- | ----------- |
| Program Counter      | Program counter                                                                   | 16          |
| Instruction Register | Stores the instruction that is being executed.                                    | 8           |
| Stack Pointer        | Stores the current position in the stack.                                         | 8          |
| L Argument register  | Stores the low byte of an argument to be used as an argument to any instruction.  | 8           |
| H Argument register  | Stores the high byte of an argument to be used as an argument to any instruction. | 8           |
| Flags register       | Stores flags(see below).                                                          | 8           |

##

### Flags

Flags are used to store boolean information between instructions. They are set automatically depending on the executed instruction. The Carry and Zero flags directly change the behavior of the control logic allowing for a feedback loop which is necessary to make the computer Turing complete.

| **name** | **description**                                                      |
| -------- | -------------------------------------------------------------------- |
| Carry    | Set when the carry flag is set. (if instruction was CMP, RAX \< RBX) |
| Zero     | Set when the result of an operation is zero -\> RAX == RBX           |
| Sign     | Set if the number could be negative.                                 |
| Overflow | Set when a signed number overflows.                                  |
| InCarry  | Set when the incrementer overflows.                                  |

## 3. Arithmetic Logic Unit

The Arithmetic Logic Unit(ALU) is responsible for performing all the calculations and logical operations of the computer. As the computer is 8-bit, the ALU works with binary numbers of that length. Allowing for a numeral range of 0 - 255 for unsigned and -128 - 127 for signed. Signed numbers are represented using two's complement. However the CPU at this level does not differentiate between signed and unsigned values it treats them the same way. It is up to the programmer to decide which value is signed and which not. The ALU uses 8-bit registers A and B as input. The result of the operation is automatically inserted into the A register.

Available operations of the ALU are:

- ADD
- SUBTRACT
- NOT, NAND
- SHIFT RIGHT

By design the ALU can only perform a small number of operations. More complex operations can be implemented in software. This allows for much simpler hardware at the cost of more instructions required to perform some operations. Since the computer is Turing complete, any calculation can be implemented using these few operations.

If the 8 bit numeral range is not sufficient and the number overflows the carry bit is set. the carry out bit from the previous operation can be used as the carry in bit of the next operation. This is done using the ADC and SBC instructions. This scheme allows for chaining of addition/subtraction operations and therefore operating on larger values than one byte.

## 4. Control logic

The control logic has multiple components, the instruction and argument registers and the microcode. Every instruction starts with a fetch cycle which fetches the instruction at the address pointed to by the Instruction Pointer from memory and stores it in the Instruction Register.

If the instruction requires an argument, it will be fetched from the next one or two memory locations following the instruction and stored in the argument registers. The number of arguments an instruction requires and how these arguments should be processed is specified by the instruction mode which is stored in the 3 highest significant bits of the instruction itself. This limits the number of hardware instructions to 32.

### Instruction modes

**Implied** addressing is the simplest instruction mode, since it doesnt require any arguments. Instead the operand is obvious from the opcode. Instructions using this mode include: PUA, SLA, SRA.

**Immediate** mode uses a one byte argument to specify a constant one byte value.

**Constant** mode uses the following two words to specify a constant two byte value.

**Absolute** mode uses the following two words as the effective address of the operand.

**Indirect** mode uses the following two words as the address containing the high byte of the effective address. The low byte is fetched by incrementing this address. This mode is useful for stepping through arrays as well as using pointers.

**Zero page** mode allows for faster memory access by only requiring the low byte of the address to be specified. The high byte is implied to be the zero page. This mode is useful for storing temporary data that would otherwise be stored in registers.

**Register A/B** mode specifies a register to be used as the operand.

A ROM containing the microcode is then used to find the set of micro instructions required to perform this instruction. This is done by combining values from the flags register, the current micro step, the number of arguments and the opcode of the instruction and using this value as a memory address in the ROM. Below is a visual representation of how these values are combined to form an address.

```
[AAAAA][BBB][CCC][DDDD]

A - opcode(5b)
B - instruction mode(3b)
C - flags(3b)
D - micro step(4b)
```

The individual micro-steps fetched from the ROM are executed in order using a 4bit counter allowing a maximum of 16 steps. The micro code consists of 40-bit values specifying which control signals have to be enabled.

## 5. Interfaces

The Tower can communicate with other hardware devices using a special part of memory that data can be read from and written to using standard memory access instructions (LDA, STA). The maximum number of connected devices is limited by the number of mapped addresses which is 256. Devices can connect using an 8bit parallel port that is not meant to be used directly by other devices rather by adapters which stand between the device and the computer. These adapters are responsible for converting parallel to serial data streams, connecting power to the device and other challenges which may appear.

Example additional hardware:

- LCD text display
- Keyboard
- Sound generator

## 6. Memory

The memory is split between multiple chips which are mapped to explicit memory address ranges. The memory layout can be seen below:

| **address range** | **purpose**          | **size** |
| ----------------- | -------------------- | -------- |
| 0x00-0xFF         | ROM - program memory | 16KB     |
| 0x100-0x1FF       | Zero page            | 256B     |
| 0x200-0x2FF       | Stack                | 256B     |
| 0x300-0xFEFF      | General purpose RAM  | ~48KB    |
| 0xFF00-0xFFFF     | I/O mapped memory    | 256B     |

### Zero page

Since the computer is designed to run at a rather slow clock speed, memory access is not much slower than register access, therefore the Tower does not have any souly general purpose registers. To compensate this, it treats the first "page" of memory(first 256B) differently by only requiring the low byte to be specified when addressed. When used correctly, this scheme can save many clock cycles. This area of memory is used to store additional data that would normally be stored in physical registers.

## 7. Instruction Set Architecture

The instruction set is defined <a href="instructions.md">here</a>.

##

### Control signals

**General**

- IEND ... end of instruction, reset the step counter
- HLT ... halt the computer

**Program Counter**

- PCI ... increment Program Counter
- PCO ... output data in the Program Counter to the address BUS
- PCJ ... set the Program Counter to the address on the address BUS

**Stack Pointer**

- SPI ... set Stack Pointer to the value on the data BUS
- SPO ... output Stack Pointer to the data BUS
- SPOA ... output Stack Pointer to the 8lsb of the address BUS

**Registers**

- AI ... set A to the value on the data BUS
- BI ... set B to the value on the data BUS
- BO ... output B to the data BUS
- HI ... set H to the value on the data BUS
- HO ... output H to the data BUS
- LI ... set L to the value on the data BUS
- LO ... output L to the data BUS
- HLO ... output H and L to the address BUS
- HLI ... set H and L to the value on the address BUS
- ARHI ... set H to the value on the data BUS
- ARHO ... output H to the data BUS
- ARLI ... set L to the value on the data BUS
- ARLO ... output L to the data BUS
- ARHLO ... output H and L to the address BUS

**ALU**

- ALUO ... output the value in the ALU to the data BUS
- OPADD ... set ALU to ADD
- OPSUB ... set ALU to SUBTRACT
- OPNOT ... set ALU to NOT
- OPNAND ... set ALU to NAND
- OPSR ... set ALU to SHIFT RIGHT
- ALUFI ... enables the Carry flag to be used by the ALu
- INCE ... enables the Incrementer
- DEC ... sets the Incrementer to decrement
- INCI ... set the Incrementer to the value on the data BUS
- INCO ... output the value in the Incrementer to the data BUS

**Flags**

- FI ... set the flags register to the 4lsb of the data BUS
- FO ... output the flags register to the 4lsb of the data BUS

**Memory**

- MI ... store value on the data BUS to memory
- MO ... output value in memory to the data BUS
- INI ... store value on the data BUS to the instruction register

**Address injection**

- \_RAMSTART ... sets the address BUS to the first address in RAM
- \_SPSTART ... sets the address BUS to the first address of the Stack (Note: this is relative to \_RAMSTART, so both signals need to be active for \_SPSTART to work)

##

## 8. Programming

The computer can be programmed in the Tower assembly language compiled using the Tower assembler.

The assembler uses syntax described in the [assembler docs](/assembler/docs/asm.md)
