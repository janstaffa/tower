
; Full test for the Tower architecture.
; Tests every instruction in every instruction mode.
; Error code will be stored at 0xFEFF.

; ========== MACROS ==========
#macro MW
	LDA $1
	STA $2
#end

; ========== TESTS ==========
	
	

	MW #0xFF, &0xFE32

	; test 1 - ADDING, INC, DEC, indirect access
	LDA #50
	STA &0xFE01
	LDA *0xFE01
	DEC %A
	DEC %A
	ADD #5
	INC %A
	ADD #200
	STA &0xFE00
	LDA @0xFE00
	SUB #50
	CMP #205
	;LDA #01 
	JNZ _failed
	

	; test 2 - NAND and SR
	LDA #0
	NOTA
	SRA
	NAND #0b00000001
	CMP #254
	LDA #02
	JNZ _failed


	; test 3 - flags
	LDA #0
	TAF
	LDA #03
	JC _failed
	LDA #255
	ADD #10
	JC _continue1
	LDA #04
	JMP _failed
	_continue1:
	ADC #10
	SUB #20
	JZ _continue2
	LDA #05
	JMP _failed
	
	_continue2:

	; test 4 - stack
	; test pushing/poping
	LDA #20
	PSA
	LDA #200
	POA
	CMP #20
	JNZ _failed

	LDA #0b1111
	TAF
	LDA #0
	PSF
	LDA #0b0011
	TAF
	POF
	TFA
	CMP #0b1111
	JNZ _failed


	; test recursion
	LDA #10
	JSR recursive_function
	JMP _continue3

	recursive_function:
		ADD #5
		JC _break
		JSR recursive_function
		_break:
			RTS

	_continue3:
	
	LDA #123
	HLT

_failed:
	STA &0xFEFF
	hlt
