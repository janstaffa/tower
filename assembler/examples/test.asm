#include "test2.asm"

#macro MW
	LDA $1
	STA $2
#end

MW #0xFF, &0x4000

lda #0
_loop:
	jc _stop
	add #1
	sta &0x4001

	sta @0x4000
	jmp _loop

_stop:
	hlt