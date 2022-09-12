#include "test2.asm"

#macro CLC
TFA
NAND #0b1110
NOTA
TAF 
#end

#macro TEST
ADD $1

#macro ADD_16
TEST *$1
ADD *$2
STA $1

LDA *$3
ADD *$4
STA $2
#end


; VARIABLE = #200

label1:
; ADD_16 0x00f, 0xabc, #128

LDA *$1
ADD *$2
STA $1

LDA *$3
ADD *$4
STA $2


label2:
