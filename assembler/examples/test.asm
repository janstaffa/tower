#include "test2.asm"

#macro 
CLC:
    TFA
    NAND 0b1110
    NOTA
    TAF 

#macro
ADD_16 $1, $2, $3, $4:
    LDA *$1
    ADD *$2
    STA $1

    LDA *$3
    ADD *$4
    STA $2



.label1:



.label2
