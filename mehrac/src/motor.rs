use defmt::*;
use embassy_stm32::gpio::{Output, AnyPin};
use embassy_time::{Duration, Timer};
use embassy_stm32::peripherals::{PA1, PA2};
use embassy_stm32::exti::ExtiInput;

#[embassy_executor::task]
pub async fn motor_forward_task(mut forward: ExtiInput<'static, PA1>, mut ctrl: Output<'static, AnyPin>) {
    info!("Motor forward task run!");
    loop {
        forward.wait_for_low().await;
        info!("Motor forward!");
        ctrl.set_high();
        Timer::after(Duration::from_millis(5000)).await;
        //forward.wait_for_high().await;
        info!("Motor forward done!");
        ctrl.set_low();
    }
}

#[embassy_executor::task]
pub async fn motor_retract_task(mut retract: ExtiInput<'static, PA2>, mut ctrl: Output<'static, AnyPin>) {
    info!("Motor retract task run!");
    loop {
        retract.wait_for_low().await;
        info!("Motor retract!");
        ctrl.set_high();
        Timer::after(Duration::from_millis(5000)).await;
        //retract.wait_for_high().await;
        info!("Motor retract done!");
        ctrl.set_low();
    }
}
