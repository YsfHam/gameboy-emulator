use gbe_core::{joypad::JoypButton, Cartridge, Gameboy};

/// Regression test for the Super Mario Land 2 joypad-interrupt stall (PC stuck
/// around 0x0085 after pressing START on the title screen).
///
/// What went wrong before the fix:
///  1. `InterruptFlags::new()` pre-set IE bit 4 (joypad) and the IE write path
///     OR-ed 0x10 into every IE write, so SML2's `LDH ($FF),A` of $01 became
///     IE=$11 and its joypad dispatch was no longer masked.
///  2. START press set IF bit 4; on HALT wake + EI the CPU dispatched into
///     0x0060. SML2 keeps nothing valid there: the region 0x0048..0x0090 is
///     filler data which decodes an illegal `FD` opcode at 0x0084, and the CPU
///     hung forever (the stall you saw).
///  3. SML2 polls P1 in an HRAM/poll loop instead of using IF bit 4 — like many
///     games — so a hardware-accurate level-triggered joypad IF policy is also
///     kept rather than one-latching the bit on every press.
fn sml2_rom() -> Vec<u8> {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "\\..\\roms\\Super Mario Land 2 - 6 Golden Coins (USA, Europe)\\Super Mario Land 2 - 6 Golden Coins (USA, Europe).gb"
    );
    std::fs::read(path).expect("SML2 ROM must be present for this test")
}

#[test]
fn sml2_press_start_does_not_stall_cpu_at_vector_filler() {
    let mut gb = Gameboy::new(Cartridge::from_memory(sml2_rom()));

    // Run 120 frames: boot, banner and title screen settle.
    for _ in 0..120 {
        gb.next_frame();
    }

    let before = gb.get_snapshot().cpu.pc;
    gb.presse_button(JoypButton::Start);

    // After the press the CPU must keep making forward progress: PC must never
    // park in the non-executable vector region, and frames must keep completing.
    for frame in 0..30 {
        gb.next_frame();
        let pc = gb.get_snapshot().cpu.pc;
        assert!(
            !(0x40..0x90).contains(&pc),
            "CPU stalled in SML2 vector filler at PC={pc:04X} (was {before:04X}) on frame {frame}"
        );
    }
}

#[test]
fn trace_last_instructions_before_stall() {
    // Diagnostic companion to the stall test: follows the PC after a START press
    // until it settles. The game ends in its 02FF HALT / 0300 P1-poll loop, which
    // is normal; the failure condition is PC entering the 0040..0090 filler.
    let mut gb = Gameboy::new(Cartridge::from_memory(sml2_rom()));
    for _ in 0..120 {
        gb.next_frame();
    }
    eprintln!("PC before press: {:04X}, IE: {:02X}", gb.get_snapshot().cpu.pc, gb.peek(0xFFFF));
    gb.presse_button(JoypButton::Start);

    let mut trail: Vec<u16> = Vec::new();
    for _ in 0..100_000 {
        gb.next_instruction();
        let pc = gb.get_snapshot().cpu.pc;
        trail.push(pc);
        if trail.len() > 60 {
            trail.remove(0);
        }
        let n = trail.len();
        if n >= 4 && trail[n - 1] == trail[n - 2] && trail[n - 2] == trail[n - 3] {
            break;
        }
    }
    eprintln!("low-region PC trail: {:04X?}", trail);
    assert!(
        !trail.iter().any(|pc| (0x40..0x90).contains(pc)),
        "CPU entered SML2 vector filler: {:04X?}", &trail
    );
}
