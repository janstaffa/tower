; works in Logisim simulation
; resends everything typed on keyboard into TTY

_loop:
	LDA *0xFF02
	SUB #0
	JZ _loop
	STA &0xFF01
	JMP _loop

_break:

_hlt:
	hlt