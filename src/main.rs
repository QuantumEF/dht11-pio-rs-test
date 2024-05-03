//! Template

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::i2c::{self, Config};
use embassy_time::Timer;
use embedded_hal_1::i2c::I2c;
use {defmt_rtt as _, panic_probe as _};

use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::{PIO0, PIO1};

use embassy_rp::pio::{
    Common, Config as PIOConfig, InterruptHandler, Pio, PioPin, ShiftDirection, StateMachine,
};

use fixed::traits::ToFixed;
use fixed_macro::types::U56F8;

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
    PIO1_IRQ_0 => InterruptHandler<PIO1>;
});

#[embassy_executor::task]
async fn pio_task_sm0(mut sm: StateMachine<'static, PIO1, 0>) {
    Timer::after_secs(5).await;
    loop {
        sm.set_enable(true);
        unsafe {
            sm.exec_instr(0x0000);
        }

        // Timer::after_micros(5).await;

        let mut dht11_data_buf: [u8; 5] = [0; 5];
        for item in &mut dht11_data_buf {
            *item = sm.rx().wait_pull().await as u8;
        }
        let mut data_checksum = 0u8;
        for item in dht11_data_buf.iter().take(4) {
            data_checksum = data_checksum.wrapping_add(*item);
        }
        if data_checksum == dht11_data_buf[4] {
            info!("Correctly Read Temp Data {:?}", dht11_data_buf);
        } else {
            warn!("Failed to properly read temp data {:?}", dht11_data_buf);
        }
        sm.set_enable(false);
        // sm.restart();

        Timer::after_secs(1).await
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Hello World!");

    let p = embassy_rp::init(Default::default());

    let Pio {
        mut common,
        irq3,
        mut sm0,
        ..
    } = Pio::new(p.PIO1, Irqs);

    let prg = pio_proc::pio_file!("src/dht11.pio");

    let pin = p.PIN_15;
    let mut cfg = PIOConfig::default();
    cfg.use_program(&common.load_program(&prg.program), &[]);
    let mut data_pin = common.make_pio_pin(pin);
    data_pin.set_pull(embassy_rp::gpio::Pull::Up);
    cfg.set_set_pins(&[&data_pin]);
    cfg.set_in_pins(&[&data_pin]);
    cfg.set_jmp_pin(&data_pin);
    cfg.clock_divider = 10.to_fixed();
    // cfg.clock_divider = U56F8!(7.5).to_fixed();
    cfg.shift_in.auto_fill = true;
    cfg.shift_in.threshold = 8;
    sm0.set_config(&cfg);

    unwrap!(spawner.spawn(pio_task_sm0(sm0)));
}
