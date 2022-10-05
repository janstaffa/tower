INC &0xFE00
JZ _end
; DEC &0xFE00
; JNZ _end 

LDA #123
HLT

_end:
	LDA #111