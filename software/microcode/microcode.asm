#pref
; fetch the instruction
PCO MO INI
PCI

#suf
IEND

#macro FETCH_LOW
PCO MO ARLI
PCI

#macro FETCH_HIGH
PCO MO ARHI
PCI


#macro FETCH_ARGS
FETCH_HIGH
FETCH_LOW


#def NOP
IEND



#def LDA
imm:
	PCO MO AI
	PCI
abs:
	FETCH_ARGS

	ARHLO MO AI

#def ADD
imm:
	; get value from memory and store in B
	PCO MO BI
	PCI

	OPADD FI ALUO AI


#def HLT
HLT

#def STA
const:
	FETCH_ARGS

	ALUO ARHLO MI

ind:
	FETCH_ARGS

	ARHLO MO HI

	ARLO INCI
	INCE INCO ARLI

	#if incarry
		ARHO INCI
		INCE INCO ARHI
	#end

	ARHLO MO LI

	HLO ALUO MI




#def JMP
const:
	FETCH_ARGS

	ARHLO PCJ



#def JC
const:
	#if carry
		FETCH_ARGS
		ARHLO PCJ
	#else
		PCI
		PCI
	#end