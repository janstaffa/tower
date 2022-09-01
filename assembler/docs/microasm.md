## Tower Docs - microassembler

The Tower microassembler is used to assemble microcode for the computer. It uses the syntax defined in this document.


### Syntax
Keywords are identified by the # symbol prefixed to the keyword itself.
- def - define the microcode for an instruction
- pref - define a prefix to be added to every instruction definition
- suf - define a suffox to be added to every instruction definition
- macro - define a macro
- if - conditionally add microcode to an instruction
- else - conditionally add microcode to an instruction
- end - end an if or else statement



### Macros
Macros can be defined by using the `#macro` keyword followed by the macro name. To use a macro, write its name instead of a control signal inside an instruction definition or another macro.

#### Inline vs multiple line
If your macro contains only a single step, it can be added to individual steps or as a separate step. If your macro contains multiple steps, you can only add it as a separate step.


**Example**
```asm
; inline
#macro m1 
sig1, sig2

#def IN1
sig3, m1

; multi-line
#macro m2
sig1
sig2

#def IN2
sig3
m2

``` 