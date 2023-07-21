#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use core::sync::atomic::{AtomicU8, Ordering};

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed, AnyPin};
use embassy_stm32::peripherals::{PA1, PA2};
use embassy_stm32::time::Hertz;
use embassy_stm32::Config;
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

static IMAGE_BANK: AtomicU8 = AtomicU8::new(0);

const FACE1: &[u8] = include_bytes!("../asset/1.bmp");
const FACE2: &[u8] = include_bytes!("../asset/2.bmp");
const FACE3: &[u8] = include_bytes!("../asset/3.bmp");
const FACE4: &[u8] = include_bytes!("../asset/4.bmp");

struct Hub75Pin {
    pub r1: Output<'static, AnyPin>,
    pub g1: Output<'static, AnyPin>,
    pub b1: Output<'static, AnyPin>,
    pub r2: Output<'static, AnyPin>,
    pub g2: Output<'static, AnyPin>,
    pub b2: Output<'static, AnyPin>,
    pub a: Output<'static, AnyPin>,
    pub b: Output<'static, AnyPin>,
    pub c: Output<'static, AnyPin>,
    pub d: Output<'static, AnyPin>,
    pub e: Output<'static, AnyPin>,
    pub oe: Output<'static, AnyPin>,
    pub clk: Output<'static, AnyPin>,
    pub lat: Output<'static, AnyPin>,
}

#[embassy_executor::task]
async fn hub75_task(mut hub75: Hub75Pin) {
    info!("Hub75 task spawned.");
    let mut prev_bank = 0;
    let faces: [&[u8]; 4] = [FACE1, FACE2, FACE3, FACE4];

    loop {
        let bank = IMAGE_BANK.load(Ordering::Relaxed);
        let chunk = &faces[bank as usize][0x3B..];

        for row in 0..16 {
            //debug!("Row: {}", row);

            // ARM the row
            for col in 0..64 {
                let idx_1 = 256 - (row * 64 + col) / 8;
                let idx_2 = 256 - (row * 2 * 64 + col) / 8;
                let off_1 = (row * 64 + col) % 8;
                let off_2 = (row * 2 * 64 + col) % 8;

                let bit1 = (chunk[idx_1] >> off_1) & 0x1;
                let bit2 = (chunk[idx_2] >> off_2) & 0x1;

                if bit1 != 0 {
                    hub75.g1.set_high();
                } else {
                    hub75.g1.set_low();
                }

                if bit2 != 0 {
                    hub75.g2.set_high();
                } else {
                    hub75.g2.set_low();
                }

                hub75.clk.set_high();
                //Timer::after(Duration::from_micros(1)).await;
                hub75.clk.set_low();
            }
            
            hub75.oe.set_high();

            hub75.lat.set_low();
            Timer::after(Duration::from_micros(1)).await;
            hub75.lat.set_high();

            let addr = row;
            if addr & 0x01 != 0 {
                hub75.a.set_high();
            } else {
                hub75.a.set_low();
            }

            if addr & 0x02 != 0 {
                hub75.b.set_high();
            } else {
                hub75.b.set_low();
            }

            if addr & 0x04 != 0 {
                hub75.c.set_high();
            } else {
                hub75.c.set_low();
            }

            if addr & 0x08 != 0 {
                hub75.d.set_high();
            } else {
                hub75.d.set_low();
            }
            //Timer::after(Duration::from_millis(1)).await;
            hub75.oe.set_low();
            //Timer::after(Duration::from_millis(1)).await;
        }

        //debug!("Frame output done!");

        //debug!("Dummy output done!");
        if bank != prev_bank {
            info!("Bank changed to {}", bank);
            prev_bank = bank;

            //hub75.r1.set_high();

            //

            // Do the business logic
        }
        //Timer::after(Duration::from_millis(10)).await;
    }
}

#[embassy_executor::task]
async fn motor_forward_task(mut forward: ExtiInput<'static, PA1>, mut ctrl: Output<'static, AnyPin>) {
    info!("Motor forward task run!");
    loop {
        forward.wait_for_low().await;
        info!("Motor forward!");
        ctrl.set_high();
        forward.wait_for_high().await;
        ctrl.set_low();
    }
}

#[embassy_executor::task]
async fn motor_retract_task(mut retract: ExtiInput<'static, PA2>, mut ctrl: Output<'static, AnyPin>) {
    info!("Motor retract task run!");
    loop {
        retract.wait_for_low().await;
        info!("Motor retract!");
        ctrl.set_high();
        retract.wait_for_high().await;
        ctrl.set_low();
    }
}

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

    let hub75 = Hub75Pin {
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
    spawner.spawn(hub75_task(hub75)).unwrap();
    spawner.spawn(motor_forward_task(motor_forward_interrupt, motor_forward_ctrl)).unwrap();
    spawner.spawn(motor_retract_task(motor_retract_interrupt, motor_retract_ctrl)).unwrap();

    loop {
        face_interrupt.wait_for_low().await;
        info!("Face change button pressed!");

        let mut image_bank = IMAGE_BANK.load(Ordering::Relaxed);
        debug!("Current bank: {}", image_bank);
        image_bank += 1;

        if image_bank > 3 {
            image_bank = 0;
        }

        debug!("New bank: {}", image_bank);
        IMAGE_BANK.store(image_bank, Ordering::Relaxed);
        face_interrupt.wait_for_high().await;
    }
}
