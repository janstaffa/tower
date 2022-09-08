## Tower Docs - assembler 

The Tower assembler is used to assemble programs for the computer. It uses the syntax defined in this document. **The assembler is case in-sensitive.**

**Table of contents:**
1. Syntax
2. Labels
3. Macros
4. Include
5. Allocation

### 1. Syntax

One instruction or macro can be written per line. Comments are prefixed with the `;` symbol. Available instructions are defined in the ISA spec.

**Registers**
Registers are prefixed with `%`.

**Literals**
Literal values are prefixed with `$` followed by the value itself (in any of the available radixes).
example: `$100 ; literal value 100`

**Radixes**
There are multiple radixes available in the assembler.
0x -\> hex
0b -\> binary
\_ -\> decimal

**Data access**

| Mode      | prefix | example     |
|-----------|--------|-------------|
| Implied   | -      | RTS         |
| Immediate | #      | ADD #15     |
| Absolute  | \*     | ADD \*0xFF  | 
| Constant  | -      | JMP 0x400   |
| Indirect  | @      | ADD @0x00FF |
| RegA/B    | %      | INC %A/B    |

