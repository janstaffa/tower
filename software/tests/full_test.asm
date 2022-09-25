
; Full test for the Tower architecture.
; Tests every instruction in every instruction mode.
; Error code will be stored at 0xFFFF.

; ========== MACROS ==========
#macro MW
	LDA $1
	STA $2
#end

; ========== TESTS ==========


	MW #0xFF, &0xFF32

	; test 1
	LDA #50
	STA &0xFF01
	LDA *0xFF01
	DEC %A
	ADD #5
	INC %A
	ADD #200
	STA &0xFF00
	LDA @0xFF00
	SUB #50
	CMP #205
	LDA #01 
	JNZ _failed
	

	; test 2
	LDA #0
	NOTA
	SRA
	NAND #0b00000001
	CMP #254
	LDA #02
	JNZ _failed




	_loop:
		JMP _loop
	
_failed:
	STA &0xFFFF
	hlt
