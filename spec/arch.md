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
| --- | --- |
| byte | 8 |
| word | 8 |
| dword | 16 |


## 2. Registers

The Tower uses multiple types of registers for different purposes, however the programmer has direct access to only a few of them. Those being the A and B registers used as inputs to the ALU as well as general purpose registers, these registers can be accessed directly from code. The rest of the registers are used for internal function of the computer and thus are managed by the computer itself. The programmer has some level of indirect control of these registers.

##
### Accessible registers

| **name** | **description** | **size(b)** |
| --- | --- | --- |
| **A** | Accumulator - stores the result of the ALU, one of the inputs to the ALU, general purpose register. | **8** |
| **B** | Input to the ALU, general purpose register. | **8** |

##
### Internal registers

| **name** | **description** | **size(b)** |
| --- | --- | --- |
| Program Counter | Program counter | 16 |
| Memory Address Register | Stores currently read/written memory address. | 16 |
| Instruction Register | Stores the instruction that is being executed. | 8 |
| L Argument register| Stores the low byte of an argument to be used as an argument to any instruction. | 8 |
| H Argument register| Stores the high byte of an argument to be used as an argument to any instruction. | 8 |
| Flags register | Stores flags(see below). | 8 |

##
### Flags

Flags are used to store boolean information between instructions. They are set automatically depending on the executed instruction. The Carry and Zero flags directly change the behavior of the control logic allowing for a feedback loop which is necessary to make the computer Turing complete.

| **name** | **description** |
| --- | --- |
| Carry | Set when the carry flag is set. (if instruction was CMP, RAX \< RBX) |
| Zero | Set when the result of an operation is zero -\> RAX == RBX |
| Sign | Set if the number could be negative. |
| Overflow | Set when a signed number overflows. |


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
[AAAAA][BB][CCC][DDDD]

A - opcode(5b)
B - flags(2b)
C - instruction mode(3b)
D - micro step(4b)
```

The individual micro-steps fetched from the ROM are executed in order using a 4bit counter allowing a maximum of 16 steps. The micro code consists of 32-bit values specifying which control signals have to be enabled.

## 5. Interfaces

The Tower can communicate with other hardware devices using a set of ports. These ports are connected directly to the data BUS and can be set to read or write data to or from the bus. Multiple devices can be connected at once and switched using the 3 least significant bits of the address bus. The maximum number of connected devices is 8. The interface is done over an 8 bit parallel signal and is not meant to be used directly by other devices rather by adapters which stand between the device and the computer. These adapters are responsible for converting parallel to serial data streams, connecting power to the device and other challenges which may occur.

Additional hardware:

- 4x20 LCD text display
- RTC
- Sound generator

## 6. Memory

The memory is split between multiple chips which are mapped to explicit memory address ranges. The memory layout can be seen below:

| **address range** | **purpose** | **size** |
| --- | --- | --- |
| 0x00-0xFF | ROM - program memory | 16KB |
| 0x100-0x1FF | Zero page | 256B |
| 0x200-0x2FF | Stack | 256B |
| 0x300-0x7FFF | General purpose RAM | 16KB |
| 0x800-0xFFFF | unused (vRAM) | ~32KB |

This allows for a very flexible architecture, since some of these chips can be accessed by other hardware rather than by the CPU.

### Zero page

Since the computer is designed to run at a rather slow clock speed, memory access is not much slower than register access, therefore the Tower does not have any souly general purpose registers. To compensate this, it treats the first "page" of memory(first 256B) differently by only requiring the low byte to be specified when addressed. When used correctly, this scheme can save many clock cycles. This area of memory is used to store additional data that would normally be stored in physical registers. These registers are referred to as "virtual registers".

Here is a list of all the additional virtual registers:

| **location** | **name** | **size(w)** | **description** |
| --- | --- | --- | --- |
| 0x00 | X | 1 | Index register. |
| 0x01 | Y | 1 | Index register. |
| 0x02 | C | 1 | General purpose register. |
| 0x03 | D | 1 | General purpose register. |
| 0x04 | E | 1 | General purpose register. |
| 0x05 | F | 1 | General purpose register. |
| 0x6..0xFF | - | 250 | Empty space available for user data. |

## 7. Instruction Set Architecture
### General format
NAME \<destination\>  \<source\>
NAME \<destination\>  \<source\>

##

### Control signals

**General**
- HLT = halt the computer
- IEND = end of instruction, reset the step counter

**Program Counter**
- PCE = enable PC
- PCO = output data in PC to the address BUS
- PCJ = set the PC to the address on the address BUS

**Registers**
- AI = set A to the data on the data BUS
- AO = output data in A to the data BUS
- BI = set B to the data on the data BUS
- BO = output data in B to the data BUS

**ALU**
- RSO = output result of the operation to the data BUS
- OPSUB = set ALU subtract
- OPNOT = set ALU to NOT
- OPNOR = set ALU to NOR
- OPAND = set ALU to and

**Flags**
- FI = set the flags register to the 4lsb of the data BUS
- FO = output the flags register to the 4lsb of the data BUS

**Memory**
- MI = set the MAR to the address on the address BUS
- RI = store the data on the data BUS to memory
- RO = output data in memory to the data BUS
- INI = store byte on the data BUS to the instruction register
- HI = store the byte on the data BUS to the H register
- HO = output the byte in the H register to the data BUS
- LI = store the byte on the data BUS to the L register
- LO = output the byte in the L register to the data BUS
- DVE = enable the device logic to access the data BUS
- DVW = if high the data from the device will be put on the data BUS, if low data from the data BUS will be sent to the device

##
### Instruction set

| **opcode** | **mnemonic** | **arg 1** | **arg 2** | **operation** | **description** |
| --- | --- | --- | --- | --- | --- |
| 0x01 | NOP | - | - | - | Doesn't do anything |
| 0x02 | ADC | addr/imm8/imm16 | - | RAX = RAX + RBX + C | Load A1 into RBX. Add RBX and the carry bit to RAX. |
|      | ADD | addr/imm8/imm16 | - | RAX = RAX + RBX | Load A1 into RBX. Add RBX to RAX.. |
|      | SUB | addr/imm8/imm16 | - | RAX = RAX - RBX | Load A1 into RBX. Subtract RBC from RAX. |
|      | SBC | addr/imm8/imm16 | - | RAX = RAX - RBX - C | Load A1 into RBX. Subtract RBC and the carry bit from RAX. |
|      | INC | %r/addr/imm16 | - | A1++ | Increments A1. |
|      | DEC | %r/addr/imm16 | - | A1– | Decrements A1. |
|      | CMP | [addr]/imm8/imm16 | - | A1 == RAX -\> Eq, A1 \< RAX -\> Ls | Sets a flag: Eq = equals, Ls |= less based on the result of a comparison. |
|      | JMP | addr | - | PC = A1 | Sets the Program Counter to A1. |
|      | JC | addr | - | C == 1 -\> PC = A1 | Sets the Program Counter to A1 if the carry |flag is set. |
|      | JZ | addr | - | S == 0 -\> PC = A1 | Set the Program Counter to A1 if the zero |flag is set 0. |
|      | MW | addr | [addr]/imm8 | A1 = A2 | Copies one byte from A1 to A2. |
|      | STA | addr | - | A1 = A2 | Copies value from a register A2 to a memory location A1. |
|      | LDA | [addr]/imm8/imm16 | - | | Stores a byte in the register A1. |
|      | NOT |
|      | AND |
|      | HLT |

## 8. Programming

The computer can be programmed in the Tower assembly language compiled using the Tower assembler.

The assembly language uses syntax described below.

**Assembly syntax**

Registers: registers are prefixed with%

Literals: literals prefixed with $

- radixes: 0x -\> hex

0b -\> binary

\_ -\> decimal

- pointers:
  - %rax - pointer register (rax)
  - 100 - pointer to address (100)

- [%rax] - get value at address/register
- literal values:
  - $100 - literal value 100






