#include "test2.asm"

#macro CLC
TFA
NAND #0b1110
NOTA
TAF 
#end

#macro TEST
ADD #$1
JMP &0xF0F0
#end

#macro ADD_16
TEST $1
#end

#macro ADD_17
 ADD_16 $1
 SUB @$2
#end


; VARIABLE = #200

#macro ADD_INLINE
 LDA $1
 ADD $2
 STA &$3
#end

label1:
 ADD_INLINE *0x10, #10, &0x01


 ADD_17 @0xff, %A

 ADD 0x000f
 
 LDA #10
 ADD #15
 STA &0x0001
 


; LDA *$1
; ADD *$2
; STA $1

; LDA *$3
; ADD *$4
; STA $2


label2:


 ADD 0x255