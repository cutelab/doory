#![no_std]
#![feature(const_fn)]
#![feature(proc_macro)]

extern crate bluepill_usbcdc;
extern crate r0;
#[macro_use] extern crate stm32f103xx;

extern crate blue_pill;
extern crate cortex_m_rtfm as rtfm;

use bluepill_usbcdc::*;
use blue_pill::gpio;
use blue_pill::gpio::*;
use blue_pill::prelude::*;
use blue_pill::time::{Seconds};
use blue_pill::{Timer, Spi};
use rtfm::{app, Resource, Threshold};

/* setup usb interrupts */

exception!(NMI, nmi_handler);
exception!(HARD_FAULT, hardfault_handler);
exception!(BUS_FAULT, bus_fault_handler);
exception!(SVCALL, svc_handler);
exception!(PENDSV, pend_sv_handler);
exception!(SYS_TICK, systick_handler);
interrupt!(CAN1_RX0, usb_lp_can1_rx0_irqhandler);

// These are "tuned" values, the calculation seems wrong
const BLINK_PERIOD: Seconds = Seconds(2); 
const INPUT_TIMEOUT: Seconds = Seconds(30);

const NUM_BLINKS: u8 = 4;

const OFF: [u8; 3] = [0, 0, 0];
const RED: [u8; 3] = [255, 0, 0];
//const GREEN: [u8; 3] = [0, 255, 0];
const BLUE: [u8; 3] = [0, 0, 255];

const NUM_ROW: usize = 4;
const NUM_COL: usize = 3;
const KEYS: [[char; NUM_COL]; NUM_ROW] = [
  ['1', '2', '3'],
  ['4', '5', '6'],
  ['7', '8', '9'],
  ['*', '0', '#'],
];

const ROW_PINS: [&GPIOPin; NUM_ROW] = [&PB12, &PB13, &PB14, &PB15];
const COL_PINS: [&GPIOPin; NUM_COL] = [&PB11, &PB10, &PB1];

app! {
    device: blue_pill::stm32f103xx,

    resources: {
        static COLOR: [u8; 3] = OFF;
        static BLINK_COUNT: u8 = 0;
    },

    idle: {
        resources: [TIM3, TIM4, BLINK_COUNT, COLOR],
    },

    tasks: {
        TIM3: {
            path: toggle,
            resources: [SPI1, TIM3, TIM4, BLINK_COUNT, COLOR]
        },
        TIM4: {
            path: timeout,
            resources: [TIM3, TIM4, BLINK_COUNT, COLOR]
        },
    },
}

fn bss_init_bugfix() {
    extern "C" {
        // Boundaries of the .bss section
        static mut _ebss: u32;
        static mut _sdata: u32;
    }
    unsafe {
        r0::zero_bss(&mut _ebss, &mut _sdata);
    }
}

fn init_usbcdc() {
    hal_init();
    system_clock_config();
    usb_init();
}

fn init(p: init::Peripherals, _r: init::Resources) {
    bss_init_bugfix();

    // USB Init
    init_usbcdc();

    // Timer Init
    let blink_timer = Timer(p.TIM3);
    let input_timeout = Timer(p.TIM4);
    blink_timer.init(BLINK_PERIOD, p.RCC);
    input_timeout.init(INPUT_TIMEOUT, p.RCC);
    input_timeout.resume(); // Seems to immediately generate an update interrupt

    // LED Init
    let spi = Spi(p.SPI1);
    spi.init(p.AFIO, p.GPIOA, p.RCC);
    spi.enable();

    // Keypad Init
    gpio::init(p.GPIOB, p.RCC);
    for rp in &ROW_PINS {
        rp.set_mode(GPIOMode::INPUT_PULL_UP);
    }
}

fn idle(_t: &mut Threshold, mut r: idle::Resources) -> ! {
    let mut cdc_send_data: [u8; 16] = [0; 16];
    let mut i = 0;
    loop {
        hal_delay(100);
        for col in 0..NUM_COL {
            COL_PINS[col].set_mode(GPIOMode::OUTPUT);
            COL_PINS[col].set_low();
            hal_delay(10);
            for row in 0..NUM_ROW {
                if ROW_PINS[row].is_low() {
                    r.TIM4.claim(_t, |tim4, _| {
                        let timer = Timer(&**tim4);
                        timer.pause();
                        timer.restart();
                        while ROW_PINS[row].is_low() { hal_delay(100); }
                        timer.resume();
                    });
                    cdc_send_data[i] = KEYS[row][col] as u8;


                    // FIXME send different color for nfc reader
                    // if cdc_send_data[i] == '*' as u8 {
                    // }

                    if cdc_send_data[i] == '#' as u8 {
                        r.TIM4.claim(_t, |tim4, _| {
                            let timer = Timer(&**tim4);
                            timer.pause();
                        });
                        r.BLINK_COUNT.claim_mut(_t, |blink_count, _| {
                            **blink_count = 0;
                        });
                        r.COLOR.claim_mut(_t, |color, _| {
                            **color = BLUE;
                        });
                        r.TIM3.claim(_t, |tim3, _| {
                            let timer = Timer(&**tim3);
                            timer.resume();
                        });
                        cdc_send_data[i] = '\n' as u8;

                        while !cdc_send(&mut cdc_send_data, i+1) {
                            hal_delay(100);
                        }
                        for j in 0..cdc_send_data.len() {
                            cdc_send_data[j] = 0;
                        }
                        i = 0;
                    }

                    i = i + 1;

                    if i >= cdc_send_data.len()-2 {
                        for j in 0..cdc_send_data.len() {
                            cdc_send_data[j] = 0;
                        }
                        i = 0;
                        r.TIM4.claim(_t, |tim4, _| {
                            let timer = Timer(&**tim4);
                            timer.pause();
                        });
                        r.BLINK_COUNT.claim_mut(_t, |blink_count, _| {
                            **blink_count = 0;
                        });
                        r.COLOR.claim_mut(_t, |color, _| {
                            **color = RED;
                        });
                        r.TIM3.claim(_t, |tim3, _| {
                            let timer = Timer(&**tim3);
                            timer.resume();
                        });
                    }
                }
            }
            COL_PINS[col].set_high();
            COL_PINS[col].set_mode(GPIOMode::INPUT_PULL_UP);
        }
    }
}

fn toggle(_t: &mut Threshold, r: TIM3::Resources) {
    let timer = Timer(&**r.TIM3);
    timer.wait().unwrap();

    if **r.BLINK_COUNT > (NUM_BLINKS-1)*2 {
        timer.pause();
    }
    **r.BLINK_COUNT = **r.BLINK_COUNT + 1;

    let mut pixel = OFF;
    if **r.BLINK_COUNT % 2 == 1{
        pixel = **r.COLOR;
    }
    
    let spi = Spi(&**r.SPI1);
    set_pixel(spi, pixel);
}

fn timeout(_t: &mut Threshold, r: TIM4::Resources) {
    // FIXME Needs to reset the usb output buffer
    let timer = Timer(&**r.TIM4);
    timer.wait().unwrap();
    timer.pause();

    r.BLINK_COUNT.claim_mut(_t, |blink_count, _| {
        **blink_count = 0;
    });
    r.COLOR.claim_mut(_t, |color, _| {
        **color = RED;
    });
    r.TIM3.claim(_t, |tim3, _| {
        let timer = Timer(&**tim3);
        timer.resume();
    });
}

fn set_pixel(spi: Spi<stm32f103xx::SPI1>, rgb: [u8; 3]) {
    for _i in 0..4 {
        while spi.send(0).is_err() {}
        let _junk = loop {
        if let Ok(byte) = spi.read() {
                break byte;
            }
        };
    }

    let r = rgb[0];
    let g = rgb[1];
    let b = rgb[2];
    let top = 0xC0 | ((!b & 0xC0) >> 2) | ((!g & 0xC0) >> 4) | ((!r & 0xC0) >> 6);

    while spi.send(top).is_err() {}
    let _junk = loop {
        if let Ok(byte) = spi.read() {
            break byte;
        }
    };
    while spi.send(b).is_err() {}
    let _junk = loop {
        if let Ok(byte) = spi.read() {
            break byte;
        }
    };
    while spi.send(g).is_err() {}
    let _junk = loop {
        if let Ok(byte) = spi.read() {
            break byte;
        }
    };
    while spi.send(r).is_err() {}
    let _junk = loop {
        if let Ok(byte) = spi.read() {
            break byte;
        }
    };

    for _i in 0..4 {
        while spi.send(0).is_err() {}
        let _junk = loop {
        if let Ok(byte) = spi.read() {
                break byte;
            }
        };
    }
}
