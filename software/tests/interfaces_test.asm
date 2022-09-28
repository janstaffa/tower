; display alternating pattern on LEDs
LDA #0b10101010
STA &0xFF00

; print all printable characters on TTY
LDA #32
_char_loop:
	STA &0xFF01
	INC %A

	CMP #127
	JNZ _char_loop

HLT