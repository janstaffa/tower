; compute 17 * 6

#macro MW
    LDA $1
    STA $2
#end

; load number 1
MW #17, &0xFE00
; load number 2
MW #6, &0xFE01

_add_loop:
    LDA *0xFE00
    CMP #0
    JZ _stop

    LDA *0xFE02
    ADD *0xFE01
    STA &0xFE02
    

    LDA *0xFE00
    SUB #1
    STA &0xFE00
    JMP _add_loop

_stop:
    HLT


