use serde::{Deserialize, Serialize};

use super::Nes;

static SAMPLE_FREQ: u32 = 40;

#[derive(Serialize, Deserialize)]
pub struct Apu {
    cycles: u16,
    sample_counter: u32,

    pulse_1: Pulse<1>,
    pulse_2: Pulse<0>,
    triangle: Triangle,
    noise: Noise,
    dmc: Dmc,
    frame_counter: FrameCounter,

    pulse_table: Vec<f32>,
    tnd_table: Vec<f32>,
}

impl Apu {
    pub(crate) fn new() -> Apu {
        let mut pulse_table = vec![0f32; 31];
        for n in 0..31 {
            pulse_table[n] = 95.52 / (8128f32 / n as f32 + 100f32);
        }

        let mut tnd_table = vec![0f32; 203];
        for n in 0..203 {
            tnd_table[n] = 163.67 / (24329f32 / n as f32 + 100f32);
        }

        Apu {
            cycles: 0,
            sample_counter: 0,
            pulse_1: Pulse::new(),
            pulse_2: Pulse::new(),
            triangle: Triangle::new(),
            noise: Noise::new(),
            dmc: Dmc::new(),
            frame_counter: FrameCounter::new(),

            pulse_table,
            tnd_table,
        }
    }
}

impl Nes {
    #[inline]
    /// https://wiki.nesdev.org/w/index.php?title=APU_Frame_Counter
    pub(crate) fn apu_tick(&mut self) {
        if self.apu.cycles % 2 == 0 {
            self.apu.pulse_1.clock();
            self.apu.pulse_2.clock();
        }

        // Use CPU cycles so I can get "half-APU-cycle" timing correct...
        self.apu.cycles = self.apu.cycles.wrapping_add(1);

        if self.apu.frame_counter.mode {
            match self.apu.cycles {
                7457 => {
                    //TODO: clock noise envelopes and triangle linear counter
                    self.apu.pulse_1.envelope.clock();
                    self.apu.pulse_2.envelope.clock();
                }
                14913 => {
                    //TODO: clock noise envelopes and triangle linear counter
                    self.apu.pulse_1.frame_clock();
                    self.apu.pulse_2.frame_clock();

                    self.apu.noise.length_counter.clock();
                    self.apu.triangle.length_counter.clock();
                }
                22371 => {
                    //TODO: clock noise envelopes and triangle linear counter
                    self.apu.pulse_1.envelope.clock();
                    self.apu.pulse_2.envelope.clock();
                }
                37281 => {
                    //TODO: clock noise envelopes and triangle linear counter
                    self.apu.pulse_1.frame_clock();
                    self.apu.pulse_2.frame_clock();

                    self.apu.noise.length_counter.clock();
                    self.apu.triangle.length_counter.clock();
                }
                37282 => {
                    self.apu.cycles = 0;
                }
                _ => (),
            }
        } else {
            match self.apu.cycles {
                0 => {
                    //TODO: clock noise envelopes and triangle linear counter
                }
                7457 => {
                    //TODO: clock noise envelopes and triangle linear counter
                    self.apu.pulse_1.envelope.clock();
                    self.apu.pulse_2.envelope.clock();
                }
                14913 => {
                    self.apu.pulse_1.frame_clock();
                    self.apu.pulse_2.frame_clock();

                    self.apu.noise.length_counter.clock();
                    self.apu.triangle.length_counter.clock();
                }
                22371 => {
                    self.apu.pulse_1.envelope.clock();
                    self.apu.pulse_2.envelope.clock();
                }
                29828 => {
                    /* if !self.apu.frame_counter.irq_inhibit {
                        self.cpu.irq_signal = true;
                    } */
                }
                29829 => {
                    self.apu.pulse_1.frame_clock();
                    self.apu.pulse_2.frame_clock();

                    self.apu.noise.length_counter.clock();
                    self.apu.triangle.length_counter.clock();

                    /* if !self.apu.frame_counter.irq_inhibit {
                        self.cpu.irq_signal = true;
                    } */
                }
                29830 => {
                    /* if !self.apu.frame_counter.irq_inhibit {
                        self.cpu.irq_signal = true;
                    } */

                    self.apu.cycles = 0;
                }
                _ => (),
            }
        }

        self.apu.sample_counter += 1;
        if self.apu.sample_counter == SAMPLE_FREQ {
            self.apu.sample_counter = 0;
            let _output = self.mixer();
        }
    }

    #[inline]
    fn mixer(&mut self) -> f32 {
        //The APU mixer formulas can be efficiently implemented using two lookup tables: a 31-entry table
        //for the two pulse channels and a 203-entry table for the remaining channels (due to the approximation
        //of tnd_out, the numerators are adjusted slightly to preserve the normalized output range).
        //
        //output = pulse_out + tnd_out
        //
        //pulse_table [n] = 95.52 / (8128.0 / n + 100)
        //
        //pulse_out = pulse_table [pulse1 + pulse2]
        //
        //The tnd_out table is approximated (within 4%) by using a base unit close to the DMC's DAC.
        //
        //tnd_table [n] = 163.67 / (24329.0 / n + 100)
        //
        //tnd_out = tnd_table [3 * triangle + 2 * noise + dmc]

        let pulse_1 = self.apu.pulse_1.output() as usize;
        let pulse_2 = self.apu.pulse_2.output() as usize;
        let pulse_out = self.apu.pulse_table[pulse_1 + pulse_2];

        let triangle = 0;
        let noise = 0;
        let dmc = 0;
        let tnd_out = self.apu.tnd_table[3 * triangle + 2 * noise + dmc];

        pulse_out + tnd_out
    }

    /// https://wiki.nesdev.org/w/index.php?title=APU_registers
    #[inline]
    pub(crate) fn apu_write_reg(&mut self, addr: usize, val: u8) {
        match addr {
            0x4000 => self.apu.pulse_1.set_dlcv(val),
            0x4001 => self.apu.pulse_1.set_epns(val),
            0x4002 => self.apu.pulse_1.set_t(val),
            0x4003 => self.apu.pulse_1.set_lt(val),
            0x4004 => self.apu.pulse_2.set_dlcv(val),
            0x4005 => self.apu.pulse_2.set_epns(val),
            0x4006 => self.apu.pulse_2.set_t(val),
            0x4007 => self.apu.pulse_2.set_lt(val),
            0x4008 => self.apu.triangle.set_c(val),
            0x400A => self.apu.triangle.set_tl(val),
            0x400B => self.apu.triangle.set_l(val),
            0x400C => self.apu.noise.set_lcn(val),
            0x400E => self.apu.noise.set_lp(val),
            0x400F => self.apu.noise.set_l(val),
            0x4010 => self.apu.dmc.set_ilf(val),
            0x4011 => self.apu.dmc.set_d(val),
            0x4012 => self.apu.dmc.set_a(val),
            0x4013 => self.apu.dmc.set_l(val),
            0x4015 => self.apu_write_status(val),
            0x4017 => {
                /* Writing to $4017 with bit 7 set ($80) will immediately clock all of its controlled units
                at the beginning of the 5-step sequence; with bit 7 clear, only the sequence is reset
                without clocking any of its units. */

                if val & 0x80 != 0 {
                    self.apu.pulse_1.frame_clock();
                    self.apu.pulse_2.frame_clock();

                    self.apu.noise.length_counter.clock();
                    self.apu.triangle.length_counter.clock();
                }
                // TODO: reset APU cycles on chaning frame counter

                self.apu.frame_counter.set_mi(val, &mut self.cpu.irq_signal)
            }
            _ => (),
        }
    }

    /// https://wiki.nesdev.org/w/index.php?title=APU#Status_.28.244015.29
    #[inline]
    pub(crate) fn apu_read_status(&mut self) -> u8 {
        let mut result = 0;
        if self.apu.pulse_1.length_counter.counter > 0 {
            result |= 1;
        }

        if self.apu.pulse_2.length_counter.counter > 0 {
            result |= 2;
        }

        if self.apu.triangle.length_counter.counter > 0 {
            result |= 4;
        }

        if self.apu.noise.length_counter.counter > 0 {
            result |= 8;
        }

        //TODO: set DMC active bit

        if self.apu.frame_counter.irq_inhibit {
            result |= 0x40;
        }

        if self.apu.dmc.irq_enable {
            result |= 0x80;
        }

        self.apu.frame_counter.irq_inhibit = false;
        self.cpu.irq_signal = false;

        result
    }

    //$4015 write ---D NT21   Enable DMC (D), noise (N), triangle (T), and pulse channels (2/1)
    //Writing a zero to any of the channel enable bits will silence that channel and immediately set its length counter to 0.
    //If the DMC bit is clear, the DMC bytes remaining will be set to 0 and the DMC will silence when it empties.
    //If the DMC bit is set, the DMC sample will be restarted only if its bytes remaining is 0. If there are bits remaining in the 1-byte sample buffer, these will finish playing before the next sample is fetched.
    //Writing to this register clears the DMC interrupt flag.
    #[inline]
    fn apu_write_status(&mut self, val: u8) {
        self.apu.dmc.irq_enable = false;

        let _d = val & 0x10 != 0;
        let n = val & 8 != 0;
        let t = val & 4 != 0;
        let p_2 = val & 2 != 0;
        let p_1 = val & 1 != 0;

        //TODO: manage DMC

        if !n {
            self.apu.noise.volume = 0;
            self.apu.noise.length_counter.counter = 0;
        }
        self.apu.noise.length_counter.enabled = n;

        if !t {
            self.apu.triangle.length_counter.counter = 0;
        }
        self.apu.triangle.length_counter.enabled = t;

        if !p_2 {
            self.apu.pulse_2.length_counter.counter = 0;
        }
        self.apu.pulse_2.length_counter.enabled = p_2;

        if !p_1 {
            self.apu.pulse_1.length_counter.counter = 0;
        }
        self.apu.pulse_1.length_counter.enabled = p_1;
    }
}

/** The reason for the odd output from the sequencer is that the counter is initialized to zero
but counts downward rather than upward. Thus it reads the sequence lookup table in the
order 0, 7, 6, 5, 4, 3, 2, 1.

Duty  Sequence lookup table   Output waveform
0     0 0 0 0 0 0 0 1         0 1 0 0 0 0 0 0 (12.5%)
1     0 0 0 0 0 0 1 1         0 1 1 0 0 0 0 0 (25%)
2     0 0 0 0 1 1 1 1         0 1 1 1 1 0 0 0 (50%)
3     1 1 1 1 1 1 0 0         1 0 0 1 1 1 1 1 (25% negated) **/
#[rustfmt::skip]
static DUTY_SEQUENCE: [bool; 0x20] = [
    false, false, false, false, false, false, false, true,
    false, false, false, false, false, false, true, true, false,
    false, false, false, true, true, true, true, true,
    true, true, true, true, true, false, false,
];

/// https://wiki.nesdev.org/w/index.php?title=APU_Pulse
#[derive(Serialize, Deserialize)]
struct Pulse<const ADDER: u16> {
    duty_cycle: u8,
    duty_seq: u8,
    envelope: Envelope,

    sweep: Sweep<ADDER>,
    length_counter: LengthCounter,
    //enabled: bool,
}

impl<const ADDER: u16> Pulse<ADDER> {
    fn new() -> Pulse<ADDER> {
        Pulse {
            duty_cycle: 0,
            duty_seq: 0,
            envelope: Envelope::new(),

            sweep: Sweep::new(),
            length_counter: LengthCounter::new(),
            //enabled: false,
        }
    }

    #[inline]
    fn set_dlcv(&mut self, val: u8) {
        self.duty_seq = (val & 0xC0) >> 3;
        self.length_counter.enabled = (val & 0x20) == 0;
        self.envelope._loop = (val & 0x20) != 0;
        self.envelope.constant_volume = (val & 0x10) != 0;
        self.envelope.period = val & 0xF;
    }

    #[inline]
    fn set_epns(&mut self, val: u8) {
        self.sweep.load(val);
    }

    #[inline]
    fn set_t(&mut self, val: u8) {
        self.sweep.timer = (self.sweep.timer & !0xFF) | u16::from(val);
    }

    #[inline]
    fn set_lt(&mut self, val: u8) {
        self.duty_cycle = 0;
        self.length_counter.load((val & 0xF8) >> 3);
        self.sweep.timer = (self.sweep.timer & !0x700) | (u16::from(val & 7) << 8);
    }

    #[inline]
    fn clock(&mut self) {
        if self.sweep.timer > 0 {
            self.sweep.timer -= 1;
        } else {
            self.duty_cycle = (self.duty_cycle + 1) & 7;
            self.sweep.timer = self.sweep.period;
        }
    }

    #[inline]
    fn frame_clock(&mut self) {
        self.length_counter.clock();
        self.envelope.clock();
        self.sweep.clock();
    }

    /** The mixer receives the current envelope volume except when The sequencer output is zero,
    or overflow from the sweep unit's adder is silencing the channel, or the length counter is
    zero, or the timer has a value less than eight. **/
    #[inline]
    fn output(&mut self) -> u8 {
        let active = DUTY_SEQUENCE[(self.duty_seq | self.duty_cycle) as usize];

        if active
            && self.length_counter.counter > 0
            && self.sweep.timer >= 8
            && self.sweep.period < 0x800
        {
            /* The envelope unit's volume output depends on the constant volume flag: if set, the
            envelope parameter directly sets the volume, otherwise the decay level is the current
            volume. The constant volume flag has no effect besides selecting the volume source;
            the decay level will still be updated when constant volume is selected. */
            if self.envelope.constant_volume {
                return self.envelope.period;
            } else {
                return self.envelope.step;
            }
        } else {
            0
        }
    }
}

//$4008   CRRR.RRRR   Linear counter setup (write)
//bit 7   C---.----   Control flag (this bit is also the length counter halt flag)
//bits 6-0-RRR RRRR   Counter reload value
//
//$400A   LLLL.LLLL   Timer low (write)
//bits 7-0LLLL LLLL   Timer low 8 bits
//
//$400B   llll.lHHH   Length counter load and timer high (write)
//bits 2-0---- -HHH   Timer high 3 bits
//Side effects: Sets the linear counter reload flag
#[derive(Serialize, Deserialize)]
struct Triangle {
    counter_control: bool,
    counter_reload: u8,
    timer: u16,

    length_counter: LengthCounter,
}

impl Triangle {
    fn new() -> Triangle {
        Triangle {
            counter_control: false,
            counter_reload: 0,
            timer: 0,

            length_counter: LengthCounter::new(),
        }
    }

    #[inline]
    fn set_c(&mut self, val: u8) {
        self.counter_control = val & 0x80 != 0;
        self.length_counter.enabled = val & 0x80 == 0;
        self.counter_reload = val & 0x7F;
    }

    #[inline]
    fn set_tl(&mut self, val: u8) {
        self.timer = (self.timer & !0xFF) | u16::from(val);
    }

    #[inline]
    fn set_l(&mut self, val: u8) {
        self.length_counter.load((val & 0xF8) >> 3);
        self.timer = (self.timer & !0x700) | (u16::from(val & 7) << 8);
        //TODO: set linear control reload flag
    }
}

//static PERIOD_NOISE: [u16; 0x10] = [
//    4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068,
//];

//$400C   --LC NNNN   Loop envelope/disable length counter, constant volume, envelope period/volume
//$400E   L--- PPPP   Loop noise, noise period
//$400F   LLLL L---   Length counter load (also starts envelope)
#[derive(Serialize, Deserialize)]
struct Noise {
    constant_volume: bool,
    volume: u8,

    loop_noise: bool,
    noise_period: u8,

    length_counter: LengthCounter,
}

impl Noise {
    fn new() -> Noise {
        Noise {
            constant_volume: false,
            volume: 0,

            loop_noise: false,
            noise_period: 0,

            length_counter: LengthCounter::new(),
        }
    }

    #[inline]
    fn set_lcn(&mut self, val: u8) {
        self.length_counter.enabled = (val & 0x20) == 0;
        self.constant_volume = (val & 0x10) != 0;
        self.volume = val & 0xF;
    }

    #[inline]
    fn set_lp(&mut self, val: u8) {
        self.loop_noise = (val & 0x80) != 0;
        self.noise_period = val & 0xF;
    }

    #[inline]
    fn set_l(&mut self, val: u8) {
        self.length_counter.load((val & 0xF8) >> 3);
    }
}

//$4010   IL-- FFFF   IRQ enable, loop sample, frequency index
//$4011   -DDD DDDD   Direct load
//$4012   AAAA AAAA   Sample address %11AAAAAA.AA000000
//$4013   LLLL LLLL   Sample length %0000LLLL.LLLL0001
#[derive(Serialize, Deserialize)]
struct Dmc {
    irq_enable: bool,
    loop_sample: bool,
    frequency_index: u8,

    direct_load: u8,

    sample_address: u16,

    sample_length: u16,
}

impl Dmc {
    fn new() -> Dmc {
        Dmc {
            irq_enable: false,
            loop_sample: false,
            frequency_index: 0,

            direct_load: 0,
            sample_address: 0,

            sample_length: 0,
        }
    }

    #[inline]
    fn set_ilf(&mut self, val: u8) {
        self.irq_enable = (val & 0x80) != 0;
        self.loop_sample = (val & 0x40) != 0;
        self.frequency_index = val & 0xF;
    }

    #[inline]
    fn set_d(&mut self, val: u8) {
        self.direct_load = val & 0x7F;
    }

    #[inline]
    fn set_a(&mut self, val: u8) {
        self.sample_address = 0xC000 | (u16::from(val) << 6);
    }

    #[inline]
    fn set_l(&mut self, val: u8) {
        self.sample_length = 1 | (u16::from(val) << 4);
    }
}

//$4017   MI--.----   Set mode and interrupt (write)
//Bit 7   M--- ----   Sequencer mode: 0 selects 4-step sequence, 1 selects 5-step sequence
//Bit 6   -I-- ----   Interrupt inhibit flag. If set, the frame interrupt flag is cleared,
//otherwise it is unaffected.
//Side effects: After 3 or 4 CPU clock cycles*, the timer is reset.
//If the mode flag is set, then both "quarter frame" and "half frame" signals are also generated
#[derive(Serialize, Deserialize)]
struct FrameCounter {
    mode: bool, //true -5-step, false-4-step
    odd_cycle: bool,
    irq_inhibit: bool,
}

impl FrameCounter {
    fn new() -> FrameCounter {
        FrameCounter {
            mode: false,
            odd_cycle: false,
            irq_inhibit: true,
        }
    }

    #[inline]
    /// https://wiki.nesdev.org/w/index.php?title=APU_Frame_Counter
    fn set_mi(&mut self, val: u8, irq_signal: &mut bool) {
        self.mode = val & 0x80 != 0;
        self.irq_inhibit &= val & 0x40 == 0;

        if self.irq_inhibit {
            *irq_signal = false;
        }
    }
}

static LENGTH_TABLE: [u8; 0x20] = [
    10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14, 12, 16, 24, 18, 48, 20, 96, 22,
    192, 24, 72, 26, 16, 28, 32, 30,
];

#[derive(Serialize, Deserialize)]
struct LengthCounter {
    enabled: bool,
    counter: u8,
}

impl LengthCounter {
    fn new() -> LengthCounter {
        LengthCounter {
            enabled: true,
            counter: 0,
        }
    }

    #[inline]
    fn load(&mut self, val: u8) {
        if self.enabled {
            self.counter = LENGTH_TABLE[val as usize];
        }
    }

    #[inline]
    fn clock(&mut self) {
        if self.counter > 0 && self.enabled {
            self.counter -= 1;
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Sweep<const ADDER: u16> {
    enabled: bool,
    negate: bool,
    shift: u8,

    period: u16,
    counter: u16,

    reload: bool,
    timer: u16,
}

impl<const ADDER: u16> Sweep<ADDER> {
    fn new() -> Sweep<ADDER> {
        Sweep {
            enabled: false,
            negate: false,
            shift: 0,

            period: 0,
            counter: 0,

            reload: false,
            timer: 0,
        }
    }

    #[inline]
    fn load(&mut self, val: u8) {
        self.enabled = (val & 0x80) != 0;
        self.period = (val as u16 & 0x70) >> 4;
        self.negate = (val & 8) != 0;
        self.shift = val & 7;
        self.reload = true;
    }

    //When the frame counter sends a half-frame clock (at 120 or 96 Hz), two things happen.
    //If the divider's counter is zero, the sweep is enabled, and the sweep unit is not muting the
    //channel: The pulse's period is adjusted.

    //If the divider's counter is zero or the reload flag is true: The counter is set to P and the
    //reload flag is cleared. Otherwise, the counter is decremented.
    #[inline]
    fn clock(&mut self) {
        if self.counter == 0 || self.reload {
            self.counter = self.period + 1;
            self.reload = false;
        } else {
            self.counter -= 1;
        }

        let mute = false;
        if self.counter == 0 && self.enabled && !mute {
            //The sweep unit continuously calculates each channel's target period in this way:

            //A barrel shifter shifts the channel's 11-bit raw timer period right by the shift count,
            //producing the change amount. If the negate flag is true, the change amount is made negative.
            //The target period is the sum of the current period and the change amount.

            //For example, if the negate flag is false and the shift amount is zero, the change amount
            //equals the current period, making the target period equal to twice the current period.

            //The two pulse channels have their adders' carry inputs wired differently, which produces
            //different results when each channel's change amount is made negative:

            //Pulse 1 adds the ones' complement (−c − 1). Making 20 negative produces a change amount
            //of −21.
            //Pulse 2 adds the two's complement (−c). Making 20 negative produces a change amount of −20.
            self.counter = self.period + 1;

            let change = self.timer >> self.shift;
            if !self.negate {
                self.period += change;
            } else {
                self.period += ADDER - change;
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Envelope {
    start: bool,
    period: u8,
    step: u8,
    constant_volume: bool,
    decay_counter: u8,
    _loop: bool,
}

impl Envelope {
    fn new() -> Envelope {
        Envelope {
            start: false,
            period: 0,
            step: 0,
            constant_volume: false,
            decay_counter: 0,
            _loop: false,
        }
    }

    //When clocked by the frame counter, one of two actions occurs: if the start flag is clear, the divider
    //is clocked, otherwise the start flag is cleared, the decay level counter is loaded with 15, and the
    //divider's period is immediately reloaded.

    //When the divider is clocked while at 0, it is loaded with V and clocks the decay level counter.
    //Then one of two actions occurs: If the counter is non-zero, it is decremented, otherwise if the
    //loop flag is set, the decay level counter is loaded with 15.
    #[inline]
    fn clock(&mut self) {
        if !self.start {
            if self.step == 0 {
                self.step = self.period;
                if self.decay_counter != 0 {
                    self.decay_counter -= 1;
                } else if self._loop {
                    self.decay_counter = 15;
                }
            } else {
                self.step -= 1;
            }
        } else {
            self.start = false;
            self.decay_counter = 15;
            self.step = self.period;
        }
    }
}
