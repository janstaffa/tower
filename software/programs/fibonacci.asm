; recursively compute fibonacci numbers from 1 to 255
; 1 1 2 3 5 8 13 21...


#macro MW
    LDA $1
    STA $2
#end


; setup memory
MW #0xFE, &0xFD00 ; absolute memory page
MW #0x02, &0xFD01 ; current page offset

MW #1, &0xFD02 ; argument X
MW #1, &0xFD03 ; argument Y

MW #1, &0xFE00 ; start of sequence
MW #1, &0xFE01

JSR _compute
HLT

_compute:
    ; compute next number
    LDA *0xFD02
    ADD *0xFD03

    JW _return
    JMP _continue

    _return:
        ; final number reached
        RTS
    _continue:
        ; update arguments
        PSA
        LDA *0xFD03
        STA &0xFD02
        POA
        STA &0xFD03

        ; store result
        STA @0xFD00
        
        ; increment page offset
        LDA *0xFD01
        ADD #1
        STA &0xFD01

        ; recurse
        JSR _compute
    
    ; return up the stack
    RTS