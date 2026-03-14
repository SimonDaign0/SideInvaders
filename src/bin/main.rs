#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use defmt::info; //info!("Hello world!");
use esp_hal::clock::CpuClock;
use esp_hal::peripherals::GPIO5;
use esp_hal::{ Async, gpio, main };
use esp_hal::time::{ Duration, Instant };
use sideInvaders::utils::sprites::ENEMY;
use sideInvaders::utils::stateMachine::StateMachine;
use ssd1306::mode::BufferedGraphicsMode;
use ::{ esp_backtrace as _, esp_println as _ };
esp_bootloader_esp_idf::esp_app_desc!();

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
//-----------------------------------------------------------------------------------------------------
use sideInvaders::utils::{ sprites::{ HEART, SHIP }, stateMachine::{ Event } };
use esp_hal::gpio::{ Input, InputConfig, Pull };
use esp_hal::i2c::master::{ I2c, Config as I2cConfig };
use esp_hal::time::Rate;
use ssd1306::{ I2CDisplayInterface, Ssd1306, prelude::* };

use embedded_graphics::{
    mono_font::{ MonoTextStyleBuilder, ascii::FONT_6X10 },
    pixelcolor::BinaryColor,
    prelude::Point,
    prelude::*,
    text::{ Baseline, Text },
    image::{ Image },
};
#[main]
fn main() -> ! {
    let perif_config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(perif_config);
    //Display config
    let i2c_bus = I2c::new(
        peripherals.I2C0,
        I2cConfig::default().with_frequency(Rate::from_khz(400)) //100 to save power
    )
        .unwrap()
        .with_scl(peripherals.GPIO1)
        .with_sda(peripherals.GPIO0)
        .into_async();
    let interface = I2CDisplayInterface::new(i2c_bus);
    let mut display = Ssd1306::new(
        interface,
        DisplaySize128x64,
        DisplayRotation::Rotate0
    ).into_buffered_graphics_mode();
    display.init().unwrap();
    //----------------------------------------
    let mut sm = StateMachine::new();
    let btn1 = Input::new(peripherals.GPIO3, InputConfig::default().with_pull(Pull::Up));
    let btn2 = Input::new(peripherals.GPIO2, InputConfig::default().with_pull(Pull::Up));
    let btn3 = Input::new(peripherals.GPIO5, InputConfig::default().with_pull(Pull::Up));
    let btn4 = Input::new(peripherals.GPIO6, InputConfig::default().with_pull(Pull::Up));
    let btn5 = Input::new(peripherals.GPIO7, InputConfig::default().with_pull(Pull::Up));

    loop {
        if btn1.is_low() {
            sm.event_handler(Event::BtnPressed(1));
        }
        if btn2.is_low() {
            sm.event_handler(Event::BtnPressed(2));
        }
        if btn3.is_low() {
            sm.event_handler(Event::BtnPressed(3));
        }
        if btn4.is_low() {
            sm.event_handler(Event::BtnPressed(4));
        }
        if btn5.is_low() {
            sm.event_handler(Event::BtnPressed(5));
        }
        sm.update(&mut display);
        blocking_delay(5);
    }
}

fn blocking_delay(delay: u64) {
    let delay_start = Instant::now();
    while delay_start.elapsed() < Duration::from_millis(delay) {}
}
