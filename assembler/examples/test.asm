    ; this is a comment
    #macro TEST
    PCE PCO
    #pref
    HLT
    #suf
    PCJ IEND
    
    ; a comment in the middle of nowhere
    #def ADD
    IEND, HLT, PCE, PCO, PCJ, AI, AO, BI, BO
    imm:
        OPSUB
        #if carry
            HLT IEND TEST
            #if zero
                DVE OPADD
            #end
        #else
            DVE DVW
        #end
        OPSR