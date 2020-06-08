use stm32f3xx_hal as p_hal;

use p_hal::stm32 as pac;

use pac::{I2C1, USART1};

use embedded_hal::digital::v2::{OutputPin};
use p_hal::flash::FlashExt;
use p_hal::gpio::GpioExt;
use p_hal::rcc::RccExt;
use p_hal::time::{Hertz, U32Ext};

pub fn setup_peripherals() -> (
    // LED output pin
    UserLed1Type,
    DelaySourceType,
    I2c1PortType,
    Spi1PortType,
    ChipSelectPinType,
    Usart1PortType,
) {
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();
    let i2c_freq: Hertz = 400.khz().into();
    // Set up the system clock
    let mut rcc = dp.RCC.constrain();
    let mut flash = dp.FLASH.constrain();
    
    // HSE: external crystal oscillator must be connected
    let clocks = rcc
        .cfgr
        .use_hse(8.mhz()) // 8 MHz external crystal
        .sysclk(64.mhz()) //TODO 72 used to work?
        .pclk1(24.mhz()) // 24 works
        .freeze(&mut flash.acr);
    //TODO enable LSE crystal (32 kHZ?)

    let delay_source = p_hal::delay::Delay::new(cp.SYST, clocks);

    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb);
    let mut gpiob = dp.GPIOB.split(&mut rcc.ahb);
    let mut gpioc = dp.GPIOC.split(&mut rcc.ahb);

    //stm32f334discovery:
    // let mut user_led1 = gpiob.pb6.into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);
    // stm32f303 robodyn:
    let mut user_led1 = gpioc
        .pc13
        .into_push_pull_output(&mut gpioc.moder, &mut gpioc.otyper);
    user_led1.set_high().unwrap();

    let i2c1_port = {
        // setup i2c1 and imu driver
        let scl = gpiob
            .pb8
            .into_open_drain_output(&mut gpiob.moder, &mut gpiob.otyper)
            .into_af4(&mut gpiob.moder, &mut gpiob.afrh);

        let sda = gpiob
            .pb9
            .into_open_drain_output(&mut gpiob.moder, &mut gpiob.otyper)
            .into_af4(&mut gpiob.moder, &mut gpiob.afrh);

        p_hal::i2c::I2c::i2c1(
            dp.I2C1,
            (scl, sda),
            i2c_freq,
            clocks,
            &mut rcc.apb1,
        )
    };

    let spi1_port = {
        // SPI1 port setup
        let sck = gpioa.pa5.into_af5(&mut gpioa.moder, &mut gpioa.afrl);
        let miso = gpioa.pa6.into_af5(&mut gpioa.moder, &mut gpioa.afrl);
        let mosi = gpioa.pa7.into_af5(&mut gpioa.moder, &mut gpioa.afrl);

        p_hal::spi::Spi::spi1(
            dp.SPI1,
            (sck, miso, mosi),
            embedded_hal::spi::MODE_0,
            3_000_000.hz(),
            clocks,
            &mut rcc.apb2,
        )
    };

    // SPI chip select CS
    let csn = gpioa
        .pa15
        .into_open_drain_output(&mut gpioa.moder, &mut gpioa.otyper);

    let usart1_port = {
        let rx = gpiob.pb7.into_af7(&mut gpiob.moder, &mut gpiob.afrl);
        let tx = gpiob.pb6.into_af7(&mut gpiob.moder, &mut gpiob.afrl);
        p_hal::serial::Serial::usart1(dp.USART1, (tx,rx), 9600.bps(), clocks, &mut rcc.apb2)
    };

    (user_led1, delay_source, i2c1_port, spi1_port, csn, usart1_port)
}

type I2c1PortType = p_hal::i2c::I2c<
    I2C1,
    (
        p_hal::gpio::gpiob::PB8<p_hal::gpio::AF4>,
        p_hal::gpio::gpiob::PB9<p_hal::gpio::AF4>,
    ),
>;

pub type Spi1PortType = p_hal::spi::Spi<
    pac::SPI1,
    (
        // p_hal::gpio::gpiob::PB3<p_hal::gpio::AF5>, //SCLK
        // p_hal::gpio::gpiob::PB4<p_hal::gpio::AF5>, //MISO?
        // p_hal::gpio::gpiob::PB5<p_hal::gpio::AF5>, //MOSI?
        p_hal::gpio::gpioa::PA5<p_hal::gpio::AF5>, //SCLK
        p_hal::gpio::gpioa::PA6<p_hal::gpio::AF5>, //MISO?
        p_hal::gpio::gpioa::PA7<p_hal::gpio::AF5>, //MOSI?
    ),
>;

pub type ChipSelectPinType =
    p_hal::gpio::gpioa::PA15<p_hal::gpio::Output<p_hal::gpio::OpenDrain>>; //CSN

pub type Usart1PortType = p_hal::serial::Serial<
    USART1,
    (
        p_hal::gpio::gpiob::PB6<p_hal::gpio::AF7>, //tx
        p_hal::gpio::gpiob::PB7<p_hal::gpio::AF7>, //rx
    ),
>;

pub type UserLed1Type =
    p_hal::gpio::gpioc::PC13<p_hal::gpio::Output<p_hal::gpio::PushPull>>;

pub type DelaySourceType =  p_hal::delay::Delay;
