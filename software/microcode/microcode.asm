#macro FETCH_ARGS
MO ARLI
PCE
PCO MO ARHI
PCE


#def LDA
MO
abs:
	FETCH_ARGS

	ARHLO MO AI

#def ADD
imm:
	; get value from memory and store in B
	PCO MO BI
	PCE

	OPADD RSO AI


#def HLT
HLT

#def STA
const:
	FETCH_ARGS

	AO ARHLO MI
