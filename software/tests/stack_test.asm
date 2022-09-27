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
	JMP _continue

	recursive_function:
		ADD #5
		JC _break
		JSR recursive_function
		_break:
			RTS
	_continue:
		HLT


	_failed:
	 	LDA #123
		HLT