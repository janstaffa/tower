## Tower Docs - microassembler

The Tower microassembler is used to assemble microcode for the computer. It uses the syntax defined in this document. **The assembler is case in-sensitive.**

**Table of contents:**
1. Syntax
2. Macros
3. Prefixes and suffixes
4. Instruction modes
5. Conditional definitions


### 1. Syntax

Keywords are identified by the # symbol prefixed to the keyword itself.

- def - define the microcode for an instruction
- pref - define a prefix to be added to every instruction definition
- suf - define a suffox to be added to every instruction definition
- macro - define a macro
- if - conditionally add microcode to an instruction
- else - conditionally add microcode to an instruction
- end - end an if or else statement

### 2. Macros

Macros can be defined by using the `#macro` keyword followed by the macro name. To use a macro, write its name instead of a control signal inside an instruction definition or another macro.

#### Inline vs multi-line

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

### 3. Prefixes and suffixes

These keywords can be used to define a prefix or a suffix respectively that is to be added to every **following** instruction definition. The pref and suf contents can consist of multiple steps and can be redefined at any time, this will not affect instructions defined before this change happened.

Note: code specified in pref or suf will not be added to macro definitions.

**Example**

```asm
; no prefix or suffix will be added
#def IN1
sig1 sig2

#pref
sig3

#suf
sig4

; prefix 'sig3' and suffix 'sig4' will be added
#def IN2
sig5
```

### 4. Instruction modes

A single instruction can have different definitions defined for different instruction modes. In order to define the behaviour for a specific instruction mode you have to include a special label specifying the mode. Any code before the first label will be used as a default for all undefined modes. These are the available modes and their labels:

| Instruction mode | label    |
| ---------------- | -------- |
| Implied          | imp      |
| Immediate        | imm      |
| Constant         | const    |
| Absolute         | abs      |
| Indirect         | ind      |
| Zero page        | zpage    |
| Register A/B     | reg(a/b) |

Note: prefixes and suffixes are added to all mode definitions

**Example**

```asm
#def ADD
sig10 ; default for imp, const, abs,...
imm:
    sig1
    sig2
ind:
    sig3
    sig4
```

### 5. Conditional definitions
A single instruction can have different definitions defined for different flag combinations. Available flags are:

| Flag  | identifier |
|-------|------------|
| Carry | carry      |
| Zero  | zero       |

To initialize a conditional definition, you have to use the `#if` keyword followed by the flag identifier. After the conditional block of code, you can either write the `#end` keyword to exit the block or the `#else` keyword to enter an else block. Note: the `#end` keyword is also used after else blocks.  

**Example:**
```asm
#def ADD
sig1 ; this will be always added at the beggining 
; if the carry flag is set sig2 will be added, otherwise sig3 will be
#if carry
    sig2
#else
    sig3
#end
sig4 ; this will be always added at the end 

```