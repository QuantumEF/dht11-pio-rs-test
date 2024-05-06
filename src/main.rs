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
    Timer::after_secs(1).await;

    loop {
        sm.set_enable(true);
        // sm.restart();
        // sm.clear_fifos();
        // unsafe {
        //     sm.exec_instr(0x0000);
        // }

        // Timer::after_micros(5).await;

        let mut dht11_data_buf: [u32; 5] = [0; 5];
        for item in &mut dht11_data_buf {
            *item = sm.rx().pull();
            // *item = 255 - (sm.rx().pull() >> 24);
        }
        let mut data_checksum = 0u32;
        for item in dht11_data_buf.iter().take(4) {
            data_checksum = data_checksum.wrapping_add(*item);
        }
        if data_checksum == dht11_data_buf[4] {
            info!("u Correctly Read Temp Data {:x}", dht11_data_buf);
        } else {
            warn!("u Failed to properly read temp data {:x}", dht11_data_buf);
        }
        // sm.set_enable(false);
        // sm.restart();
        // unsafe {
        //     sm.exec_instr(0x0000);
        // }

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
    info!("Huh {:x}", prg.program.code.as_slice());

    let pin = p.PIN_15;
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

        // while let Some(data) = sm0.rx().try_pull() {
        //     info!("AA Clear data {:?}", data);
        // }
        // unwrap!(spawner.spawn(pio_task_sm0(sm0)));

        sm0.set_enable(true);
        Timer::after_micros(5).await;

        // let mut buffer: [u32; 32] = [0; 32];
        // let mut buffer_index: usize = 0;
        // while let Some(data) = sm0.rx().try_pull() {
        //     // info!("Data {:?}", data);
        //     buffer[buffer_index] = data;
        //     buffer_index += 1;
        // }
        // info!("Buffer: {:04x}", buffer);
        let mut dht11_data_buf: [u32; 5] = [0; 5];
        for item in &mut dht11_data_buf {
            *item = sm0.rx().pull();
            // *item = 255 - (sm.rx().pull() >> 24);
        }
        // let mut data_checksum = 0u32;
        // for item in dht11_data_buf.iter().take(4) {
        //     data_checksum = data_checksum.wrapping_add(*item);
        // }
        // if data_checksum == dht11_data_buf[4] {
        // info!("u Correctly Read Temp Data {:x}", dht11_data_buf);
        // } else {
        //     warn!("u Failed to properly read temp data {:x}", dht11_data_buf);
        // }
        // sm0.set_enable(false);
        info!("Data {:x}", dht11_data_buf);

        sm0.restart();
        // unsafe {
        //     sm.exec_instr(0x0000);
        // }

        Timer::after_secs(1).await
    }
}
