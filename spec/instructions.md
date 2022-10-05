# Tower - Instruction set

| opcode | mnemonic |  available modes  |           operation           |                               description                                |
| :----: | :------: | :---------------: | :---------------------------: | :----------------------------------------------------------------------: |
|  0x01  |   NOP    |        Imp        |               -               |                              Does nothing.                               |
|  0x02  |   LDA    | Imm,Abs,ZPage,Ind |            A = Arg            |                       Load a value in register A.                        |
|  0x03  |   STA    |  Const,Ind,ZPage  |            Arg = A            |                  Stores value in register A to memory.                   |
|  0x04  |   ADW    | Imm,Abs,ZPage,Ind |        A = A + B + FC         |      Load a value into B, than add it to A and Wrap and store in A.      |
|  0x05  |   ADD    | Imm,Abs,ZPage,Ind |           A = A + B           |         Load a value into B, than add it with A and store in A.          |
|  0x06  |   SBW    | Imm,Abs,ZPage,Ind |        A = A - B - FB         | Load a value into B, than subtract it and Borrow from A and store in A.  |
|  0x07  |   SUB    | Imm,Abs,ZPage,Ind |           A = A - B           |       Load a value into B, than subtract it from A and store in A.       |
|  0x08  |   INC    |      Abs,Acc      |             Arg++             |                         Increments the operand.                          |
|  0x09  |   DEC    |      Abs,Acc      |             Arg--             |                         Decrements the operand.                          |
|  0x0a  |   CMP    | Imm,Abs,ZPage,Ind | A == Arg -> FZ, A < Arg -> FO |      Load a value into B, than subtract it from A and store Flags.       |
|  0x0b  |   JMP    |     Const,Ind     |           PC = Arg            |                     Sets the Program Counter to Arg.                     |
|  0x0c  |    JW    |     Const,Ind     |      FC == 1 -> PC = Arg      |         Sets the Program Counter to Arg if the Wrap flag is set.         |
|  0x0d  |    JZ    |     Const,Ind     |      Z == 1 -> PC = Arg       |         Set the Program Counter to Arg if the Zero flag is set.          |
|  0x0e  |   JNZ    |     Const,Ind     |      Z == 0 -> PC = Arg       |       Set the Program Counter to Arg if the Zero flag is not set.        |
|  0x0f  |   NOT    |      Abs,Acc      |            A = !A             |                     Inverts the value in register A.                     |
|  0x10  |   NAND   | Imm,Abs,ZPage,Ind |        A = A NAND Arg         |         Load a value into B, than NAND it with A and store in A.         |
|  0x11  |    SR    |      Abs,Acc      |          A = A >> 1           |           Performs the Shift-Right operation on the value in A           |
|  0x12  |    SL    |      Abs,Acc      |          A = A << 1           |           Performs the Shift-Left operation on the value in A            |
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
|  0x1d  |   HLT    |        Imp        |               -               |                           Halts the computer.                            |
