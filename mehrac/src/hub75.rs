use core::sync::atomic::{AtomicU8, Ordering};

use defmt::*;
use embassy_stm32::gpio::{Output, AnyPin};
use embassy_time::{Duration, Timer};

pub static IMAGE_BANK: AtomicU8 = AtomicU8::new(0);

const FACE1: &[u8] = include_bytes!("../asset/1.bmp");
const FACE2: &[u8] = include_bytes!("../asset/2.bmp");
const FACE3: &[u8] = include_bytes!("../asset/3.bmp");
const FACE4: &[u8] = include_bytes!("../asset/4.bmp");

pub struct Hub75Pin {
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
pub async fn hub75_task(mut hub75: Hub75Pin) {
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
            hub75.oe.set_low();
        }

        if bank != prev_bank {
            info!("Bank changed to {}", bank);
            prev_bank = bank;
        }
    }
}