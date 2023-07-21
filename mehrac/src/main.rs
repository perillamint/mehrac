#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use core::sync::atomic::Ordering;

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed, AnyPin};
use embassy_stm32::time::Hertz;
use embassy_stm32::Config;
use {defmt_rtt as _, panic_probe as _};

mod hub75;
mod motor;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config = Config::default();

    // Initialize RCC
    config.rcc.hse = Some(Hertz(8_000_000));
    config.rcc.sys_ck = Some(Hertz(72_000_000));
    config.rcc.pclk1 = Some(Hertz(24_000_000));
    let p = embassy_stm32::init(config);

    info!("Mehrac initialized.");

    let face_button = Input::new(p.PA0, Pull::Up);
    let mut face_interrupt = ExtiInput::new(face_button, p.EXTI0);
    let motor_forward = Input::new(p.PA1, Pull::Up);
    let motor_forward_interrupt = ExtiInput::new(motor_forward, p.EXTI1);
    let motor_forward_ctrl = Output::new(Into::<AnyPin>::into(p.PA4), Level::Low, Speed::VeryHigh);
    let motor_retract = Input::new(p.PA2, Pull::Up);
    let motor_retract_interrupt = ExtiInput::new(motor_retract, p.EXTI2);
    let motor_retract_ctrl = Output::new(Into::<AnyPin>::into(p.PA5), Level::Low, Speed::VeryHigh);

    let hub75 = hub75::Hub75Pin {
        r1: Output::new(Into::<AnyPin>::into(p.PB0), Level::Low, Speed::VeryHigh),
        g1: Output::new(Into::<AnyPin>::into(p.PB1), Level::Low, Speed::VeryHigh),
        b1: Output::new(Into::<AnyPin>::into(p.PA8), Level::Low, Speed::VeryHigh),
        r2: Output::new(Into::<AnyPin>::into(p.PA9), Level::Low, Speed::VeryHigh),
        g2: Output::new(Into::<AnyPin>::into(p.PB5), Level::Low, Speed::VeryHigh),
        b2: Output::new(Into::<AnyPin>::into(p.PB6), Level::Low, Speed::VeryHigh),
        a: Output::new(Into::<AnyPin>::into(p.PB7), Level::Low, Speed::VeryHigh),
        b: Output::new(Into::<AnyPin>::into(p.PB8), Level::Low, Speed::VeryHigh),
        c: Output::new(Into::<AnyPin>::into(p.PB9), Level::Low, Speed::VeryHigh),
        d: Output::new(Into::<AnyPin>::into(p.PB10), Level::Low, Speed::VeryHigh),
        e: Output::new(Into::<AnyPin>::into(p.PB11), Level::Low, Speed::VeryHigh),
        clk: Output::new(Into::<AnyPin>::into(p.PB13), Level::Low, Speed::VeryHigh),
        lat: Output::new(Into::<AnyPin>::into(p.PB14), Level::High, Speed::VeryHigh),
        oe: Output::new(Into::<AnyPin>::into(p.PB15), Level::High, Speed::VeryHigh),
    };

    // Spawn the tasks
    spawner.spawn(hub75::hub75_task(hub75)).unwrap();
    spawner.spawn(motor::motor_forward_task(motor_forward_interrupt, motor_forward_ctrl)).unwrap();
    spawner.spawn(motor::motor_retract_task(motor_retract_interrupt, motor_retract_ctrl)).unwrap();

    loop {
        face_interrupt.wait_for_low().await;
        info!("Face change button pressed!");

        let mut image_bank = hub75::IMAGE_BANK.load(Ordering::Relaxed);
        debug!("Current bank: {}", image_bank);
        image_bank += 1;

        if image_bank > 3 {
            image_bank = 0;
        }

        debug!("New bank: {}", image_bank);
        hub75::IMAGE_BANK.store(image_bank, Ordering::Relaxed);
        face_interrupt.wait_for_high().await;
    }
}
