# Tower - Instruction set

| opcode | mnemonic |  available modes  |           operation           |                               description                                |
| :----: | :------: | :---------------: | :---------------------------: | :----------------------------------------------------------------------: |
|  0x01  |   NOP    |        Imp        |               -               |                              Does nothing.                               |
|  0x02  |   LDA    | Imm,Abs,ZPage,Ind |            A = Arg            |                       Load a value in register A.                        |
|  0x03  |   STA    |       Const       |            Arg = A            |                  Stores value in register A to memory.                   |
|  0x04  |   ADC    | Imm,Abs,ZPage,Ind |        A = A + B + FC         |    Load a value into B, than add it with A and Carry and store in A.     |
|  0x05  |   ADD    | Imm,Abs,ZPage,Ind |           A = A + B           |         Load a value into B, than add it with A and store in A.          |
|  0x06  |   SBB    | Imm,Abs,ZPage,Ind |        A = A - B - FB         | Load a value into B, than subtract it and Borrow from A and store in A.  |
|  0x07  |   SUB    | Imm,Abs,ZPage,Ind |           A = A - B           |       Load a value into B, than subtract it from A and store in A.       |
|  0x08  |   INC    |   Const,RegA/B    |             Arg++             |                          Increments the value.                           |
|  0x09  |   DEC    |   Const,RegA/B    |             Arg--             |                          Decrements the value.                           |
|  0x0a  |   CMP    | Imm,Abs,ZPage,Ind | A == Arg -> FZ, A < Arg -> FO |      Load a value into B, than subtract it from A and store Flags.       |
|  0x0b  |   JMP    |       Const       |           PC = Arg            |                     Sets the Program Counter to Arg.                     |
|  0x0c  |    JC    |       Const       |      FC == 1 -> PC = Arg      |        Sets the Program Counter to Arg if the Carry flag is set.         |
|  0x0d  |    JZ    |       Const       |      Z == 1 -> PC = Arg       |         Set the Program Counter to Arg if the Zero flag is set.          |
|  0x0e  |   JNZ    |       Const       |      Z == 0 -> PC = Arg       |       Set the Program Counter to Arg if the Zero flag is not set.        |
|  0x0f  |   NOTA   |        Imp        |            A = !A             |                     Inverts the value in register A.                     |
|  0x10  |   NAND   | Imm,Abs,ZPage,Ind |        A = A NAND Arg         |         Load a value into B, than NAND it with A and store in A.         |
|  0x11  |   SRA    |        Imp        |          A = A >> 1           |           Performs the Shift-Right operation on the value in A           |
|  0x12  |   SLA    |        Imp        |          A = A << 1           |           Performs the Shift-Left operation on the value in A            |
|  0x13  |   JSR    |       Const       |   \*SP = PC; SP++; PC = Arg   | Sets the Program Counter to Arg and saves current PC value on the stack. |
|  0x14  |   RTS    |        Imp        |           PC = \*SP           |        Sets the Program Counter to the value on top of the stack.        |
|  0x15  |   PSA    |        Imp        |           \*SP = A            |             Pushes value in register A on top of the stack.              |
|  0x16  |   PSF    |        Imp        |           \*SP = F            |                    Pushes Flags on top of the stack.                     |
|  0x17  |   POA    |        Imp        |           A = \*SP            |      Pops the top value from the stack and saves it in A register.       |
|  0x18  |   POF    |        Imp        |           F = \*SP            |         Pops the top value from the stack and saves it in Flags.         |
|  0x19  |   TBA    |        Imp        |             A = B             |                        Transfers value in B to A.                        |
|  0x1a  |   TAB    |        Imp        |             B = A             |                        Transfers value in A to B.                        |
|  0x1b  |   TFA    |        Imp        |             A = F             |                        Transfers value in F to A.                        |
|  0x1c  |   TAF    |        Imp        |             F = A             |                        Transfers value in A to F.                        |
|  0x1f  |   HLT    |        Imp        |               -               |                           Halts the computer.                            |
