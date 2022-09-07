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
abs:
	FETCH_ARGS

	ARHLO MO AI

#def ADD
imm:
	; get value from memory and store in B
	PCO MO BI
	PCI

	OPADD ALUO AI


#def HLT
HLT

#def STA
const:
	FETCH_ARGS

	ALUO ARHLO MI


#def JMP
const:
	FETCH_ARGS

	ARHLO PCJ