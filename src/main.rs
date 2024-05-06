//! Template

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::{PIO0, PIO1};

use embassy_rp::pio::{Config as PIOConfig, InterruptHandler, Pio, PioPin, ShiftDirection};

use fixed::traits::ToFixed;

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
    PIO1_IRQ_0 => InterruptHandler<PIO1>;
});

#[embassy_executor::task]
async fn dht11_task(pio: Pio<'static, PIO1>, pin: impl PioPin) {
    let prg = pio_proc::pio_file!("src/dht11.pio");
    let Pio {
        mut common,
        mut sm0,
        ..
    } = pio;
    // info!("Huh {:x}", prg.program.code.as_slice());
    let mut cfg = PIOConfig::default();
    cfg.use_program(&common.load_program(&prg.program), &[]);
    let mut data_pin = common.make_pio_pin(pin);
    data_pin.set_pull(embassy_rp::gpio::Pull::Up);
    cfg.set_set_pins(&[&data_pin]);
    cfg.set_in_pins(&[&data_pin]);
    cfg.set_jmp_pin(&data_pin);
    // x&y are set to 31 in the pio program, this helps initialize a loop of 1024*100 cycles which needs to add up to ~20ms according to whoever wrote it.
    // Cursory examination indicated a system clock frequency of 125MHz
    // 102400 cycles / 20ms = 5.12 MHz -> 125MHz/5.12MHz = 24.414, thus a clock divider of near to 24.414
    // cfg.clock_divider = 82.to_fixed();
    cfg.clock_divider = 125.to_fixed();
    cfg.shift_in.auto_fill = true;
    cfg.shift_in.threshold = 8;
    cfg.shift_in.direction = ShiftDirection::Left;
    loop {
        sm0.set_config(&cfg);

        sm0.set_enable(true);
        Timer::after_micros(5).await;

        let mut dht11_data_buf: [u32; 5] = [0; 5];
        for item in &mut dht11_data_buf {
            *item = sm0.rx().pull();
        }
        info!(
            "Temperature {}°C, Humidity: {}%",
            dht11_data_buf[2], dht11_data_buf[0]
        );
        sm0.restart();
        Timer::after_secs(1).await
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Hello World!");

    let p = embassy_rp::init(Default::default());

    let dht11_pio = Pio::new(p.PIO1, Irqs);

    let dht11_pin = p.PIN_15;

    unwrap!(spawner.spawn(dht11_task(dht11_pio, dht11_pin)));
}
