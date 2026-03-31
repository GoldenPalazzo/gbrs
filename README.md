# gbrs

A personal project for emulating a GameBoy, written in Rust.

## Passed test roms

This is the list of the passed test suites by this emulator.

### Blargg's test suite

- [x] [CPU instructions](#cpu-instructions)
- [x] Instruction timing
- [ ] DMG sounds
- [ ] [Memory timing 1](#memory-timing-1)
- [ ] Memory timing 2
- [ ] [OAM bug](#oam-bug)
- [x] Halt bug

#### CPU instructions

1. [x] Special instructions
2. [x] Interrupts
3. [x] OP SP, HL
4. [x] OP R, IMM
5. [x] OP RP
6. [x] LD R, R
7. [x] JR, JP, CALL, RET, RST
8. [x] Miscellaneous instructions
9. [x] OP R, R
10. [x] Bit op
11. [x] OP A, (HL)

#### Memory timing 1

1. [ ] Read timing
2. [ ] Write timing
3. [ ] Modify timing

#### OAM bug

1. [ ] LCD sync
2. [ ] Causes
3. [x] Non-causes
4. [ ] Scanline timing
5. [ ] Timing bug
6. [x] Timing no bug
7. [ ] Timing effect
8. [ ] Instruction effect
