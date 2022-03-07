.set NULL, 0
LOOP = -1

.data
            .space 5                ; Just to test alignment
            .even
my_word:    .word NULL
my_vec:     .byte 1, 2, 3, 4, 5

.text
            MOVI  R0, lo(my_vec)    ; Load @my_vec in R0
            MOVHI R0, hi(my_vec)
            XOR   R1, R1, R1        ; Set R1 to 5
            ADDI  R1, R1, 5
            XOR   R2, R2, R2        ; Set R2 to 0 (reference value)
            XOR   R3, R3, R3        ; Set R3 to 0 (accumulator)

while:      CMPEQ R2, R2, R1        ; Do 5 iterations
            BNZ   R2, endwhile
            LDB   R4, 0(R0)         ; Load next vector element
            ADD   R3, R3, R4        ; Accumulate
            ADDI  R0, R0, 1         ; Advance vector pointer
            ADDI  R1, R1, -1        ; --i
            BZ    R2, while         ; Repeat

endwhile:   MOVI  R0, lo(my_word)   ; Load @my_word in R0
            MOVHI R0, hi(my_word)
            ST    0(R0), R3         ; Store the result
            BZ    R2, LOOP          ; Loop indefinitely
.end

This program should save the sum of the contents of my_vec in my_word