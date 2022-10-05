; Fetch the instruction.
#pref
	PCO MO INI
	PCI

; Reset step counter after every instruction ends.
#suf
	IEND


#macro FETCH_LOW
	PCO MO ARLI
	PCI

#macro FETCH_HIGH
	PCO MO ARHI
	PCI


; Fetches next two words from memory and stores them in ARH and ARL respectively.
#macro FETCH_ARGS
	FETCH_HIGH
	FETCH_LOW


; Fetches two words starting at the address specified at the next two locations in memory and stores them in H and L respectively.
#macro FETCH_INDIRECTLY
	FETCH_ARGS

	ARHLO MO HI

	ARLO INCI
	INCE
	INCO ARLI

	#if incwrap 
		ARHO INCI
		INCE
		INCO ARHI
	#end

	ARHLO MO LI


; ========== NOP ==========
#def NOP
; do nothing


; ========== LDA ==========
#def LDA
imm:
	PCO MO AI
	PCI
abs:
	FETCH_ARGS

	ARHLO MO AI
zpage:
	FETCH_LOW

	_RAMSTART ARHLO MO AI
ind:
	FETCH_INDIRECTLY

	HLO MO AI

; ========== STA ==========
#def STA
const:
	FETCH_ARGS

	ALUO ARHLO MI
zpage:
	FETCH_LOW

	_RAMSTART ARHLO ALUO MI
ind:
	FETCH_INDIRECTLY

	HLO ALUO MI

#suf
; nothing

; ========== ADC ==========
#def ADC
imm:
	; get value from memory and store in B
	PCO MO BI
	PCI

	OPADD ALUFI ALUO AI
	IEND
abs:
	FETCH_ARGS

	ARHLO MO BI

	OPADD ALUFI ALUO AI
	IEND
zpage:
	FETCH_LOW

	_RAMSTART ARHLO MO BI

	OPADD ALUFI ALUO AI
	IEND
ind:
	FETCH_INDIRECTLY

	HLO MO BI
	OPADD ALUFI ALUO AI



; ========== ADD ==========
#def ADD
imm:
	; get value from memory and store in B
	PCO MO BI
	PCI

	OPADD ALUO AI
	IEND
abs:
	FETCH_ARGS

	ARHLO MO BI

	OPADD ALUO AI
	IEND
zpage:
	FETCH_LOW

	_RAMSTART ARHLO MO BI

	OPADD ALUO AI
	IEND
ind:
	FETCH_INDIRECTLY

	HLO MO BI
	OPADD ALUO AI

#suf
	IEND


#suf
; nothing

; ========== SBB ==========
#def SBB
imm:
	; get value from memory and store in B
	PCO MO BI
	PCI

	OPSUB ALUFI ALUO AI
	IEND
abs:
	FETCH_ARGS

	ARHLO MO BI

	OPSUB ALUFI ALUO AI
	IEND
zpage:
	FETCH_LOW

	_RAMSTART ARHLO MO BI

	OPSUB ALUFI ALUO AI
	IEND
ind:
	FETCH_INDIRECTLY

	HLO MO BI
	OPSUB ALUFI ALUO AI




; ========== SUB ==========
#def SUB
imm:
	; get value from memory and store in B
	PCO MO BI
	PCI

	OPSUB ALUO AI
	IEND
abs:
	FETCH_ARGS

	ARHLO MO BI

	OPSUB ALUO AI
	IEND
zpage:
	FETCH_LOW

	_RAMSTART ARHLO MO BI

	OPSUB ALUO AI
	IEND
ind:
	FETCH_INDIRECTLY

	HLO MO BI
	OPSUB ALUO AI

#suf
	IEND

; ========== INC ==========
#def INC
abs:
	FETCH_ARGS

	ARHLO MO INCI
	INCE 
	ARHLO INCO MI
accumulator:
	ALUO INCI
	INCE 
	INCO AI

; ========== DEC ==========
#def DEC
abs:
	FETCH_ARGS

	ARHLO MO INCI
	DEC INCE 
	ARHLO INCO MI
accumulator:
	ALUO INCI
	DEC INCE
	INCO AI

#suf
; nothing

; ========== CMP ==========
#def CMP
imm:
	; get value from memory and store in B
	PCO MO BI
	PCI

	OPSUB
abs:
	FETCH_ARGS

	ARHLO MO BI

	OPSUB
zpage:
	FETCH_LOW

	_RAMSTART ARHLO MO BI

	OPSUB
ind:
	FETCH_INDIRECTLY

	HLO MO BI
	OPSUB

#suf
	IEND

; ========== JMP ==========
#def JMP
const:
	FETCH_ARGS

	ARHLO PCJ
ind:
	FETCH_INDIRECTLY

	HLO PCJ


; ========== JW ==========
#def JW
const:
	#if wrap
		FETCH_ARGS
		ARHLO PCJ
	#else
		PCI
		PCI
	#end
ind:
	#if wrap
		FETCH_INDIRECTLY
		HLO PCJ
	#else
		PCI
		PCI
	#end

; ========== JZ ==========
#def JZ
const:
	#if zero
		FETCH_ARGS
		ARHLO PCJ
	#else
		PCI
		PCI
	#end
ind:
	#if zero
		FETCH_INDIRECTLY
		HLO PCJ
	#else
		PCI
		PCI
	#end

; ========== JNZ ==========
#def JNZ
const:
	#if !zero
		FETCH_ARGS
		ARHLO PCJ
	#else
		PCI
		PCI
	#end
ind:
	#if !zero 
		FETCH_INDIRECTLY
		HLO PCJ
	#else
		PCI
		PCI
	#end
; ========== NOT ==========
#def NOT
accumulator:
	OPNOT ALUO AI

#suf
; nothing

; ========== NAND ==========
#def NAND
imm:
	; get value from memory and store in B
	PCO MO BI
	PCI

	OPNAND ALUO AI
abs:
	FETCH_ARGS

	ARHLO MO BI

	OPNAND ALUO AI
zpage:
	FETCH_LOW

	_RAMSTART ARHLO MO BI

	OPNAND ALUO AI
ind:
	FETCH_INDIRECTLY

	HLO MO BI
	OPNAND ALUO AI
#suf
	IEND

; ========== SR ==========
#def SR
accumulator:
	OPSR ALUO AI

; ========== SL ==========
#def SL
accumulator:
	ALUO BI
	OPADD ALUO AI


; remove the suffix to save a step
;#suf
; nothing 

; ========== JSR ==========
#def JSR
const:	
	FETCH_ARGS
	
	; save current PC value
	PCO HLI

	; jump to subroutine
	ARHLO PCJ


	; HO ARHI
	; LO ARLI

	; store high byte on stack
	_RAMSTART _SPSTART SPOA HO MI
	
	SPO INCI
	INCE INCO SPI

	; store low byte on stack
	_RAMSTART _SPSTART SPOA LO MI
	
	SPO INCI
	INCE INCO SPI

;#suf
;	IEND

; ========== RTS ==========
#def RTS
imp:	
	SPO INCI
	DEC INCE INCO SPI
	
	_RAMSTART _SPSTART SPOA MO LI
	
	SPO INCI
	DEC INCE INCO SPI
	
	_RAMSTART _SPSTART SPOA MO HI
	
	HLO PCJ

; ========== PSA ==========
#def PSA
imp:	
	_RAMSTART _SPSTART SPOA ALUO MI
	
	SPO INCI
	INCE INCO SPI

; ========== PSF ==========
#def PSF
imp:	
	_RAMSTART _SPSTART SPOA FO MI
	
	SPO INCI
	INCE INCO SPI

; ========== POA ==========
#def POA
imp:	
	SPO INCI
	DEC INCE INCO SPI

	_RAMSTART _SPSTART SPOA MO AI
	

; ========== POF ==========
#def POF
imp:	
	SPO INCI
	DEC INCE INCO SPI

	_RAMSTART _SPSTART SPOA MO FI
	
; ========== TBA ==========
#def TBA 
imp:	
	BO AI

; ========== TAB ==========
#def TAB 
imp:	
	ALUO BI

; ========== TFA ==========
#def TFA 
imp:	
	FO AI

; ========== TAF ==========
#def TAF 
imp:	
	ALUO FI

; ========== HLT ==========
#def HLT
HLT