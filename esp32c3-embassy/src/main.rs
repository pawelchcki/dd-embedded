// Copyright Claudio Mattera 2024.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Main crate

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]
#![allow(static_mut_refs)]

use core::{convert::Infallible, mem::MaybeUninit};

use log::{error, info};

#[global_allocator]
static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();
fn init_heap() {
    const HEAP_SIZE: usize = 128 * 1024;
    static mut HEAP: MaybeUninit<[u8; HEAP_SIZE]> = MaybeUninit::uninit();
    unsafe {
        ALLOCATOR.init(HEAP.as_mut_ptr() as *mut u8, HEAP_SIZE);
    }
}

use embassy_executor::Spawner;

use embassy_time::{Delay, Duration, Timer};

use embassy_sync::{blocking_mutex::raw::NoopRawMutex, channel::Channel};

use esp_hal::{
    clock::ClockControl,
    dma::{Channel0, Dma, DmaDescriptor, DmaPriority},
    embassy,
    i2c::I2C,
    peripherals::{Peripherals, SPI2},
    prelude::{_esp_hal_system_SystemExt, _fugit_RateExtU32, entry, main, ram},
    spi::{
        master::{
            dma::{SpiDma, WithDmaSpi2},
            Spi,
        },
        FullDuplexMode, SpiMode,
    },
    timer::TimerGroup,
    Delay as EspDelay, Rng, IO,
};

use time::OffsetDateTime;

use heapless::{HistoryBuffer, String};

use embedded_hal_bus::spi::ExclusiveDevice;

use embedded_hal::digital::OutputPin;

use esp_backtrace as _;

use static_cell::StaticCell;

mod logging;
use self::logging::setup as setup_logging;

mod clock;
use self::clock::{Clock, Error as ClockError};

mod http;
use self::http::Client as HttpClient;

mod random;
use self::random::RngWrapper;

mod sleep;
use self::sleep::enter_deep as enter_deep_sleep;

mod wifi;
use self::wifi::{connect as connect_to_wifi, Error as WifiError, STOP_WIFI_SIGNAL};

mod worldtimeapi;

mod dd;

/// Duration of deep sleep
const DEEP_SLEEP_DURATION: Duration = Duration::from_secs(300);

/// Period to wait before going to deep sleep
const AWAKE_PERIOD: Duration = Duration::from_secs(300);

/// SSID for WiFi network
const WIFI_SSID: &str = env!("WIFI_SSID");

/// Password for WiFi network
const WIFI_PASSWORD: &str = env!("WIFI_PASSWORD");

/// Size of SPI DMA descriptors
const DESCRIPTORS_SIZE: usize = 8 * 3;

/// Descriptors for SPI DMA
static DESCRIPTORS: StaticCell<[DmaDescriptor; DESCRIPTORS_SIZE]> = StaticCell::new();

/// RX descriptors for SPI DMA
static RX_DESCRIPTORS: StaticCell<[DmaDescriptor; DESCRIPTORS_SIZE]> = StaticCell::new();

/// Stored boot count between deep sleep cycles
///
/// This is a statically allocated variable and it is placed in the RTC Fast
/// memory, which survives deep sleep.
#[ram(rtc_fast)]
static mut BOOT_COUNT: u32 = 0;

/// Main task
#[main]
async fn main(spawner: Spawner) {
    setup_logging();

    // SAFETY:
    // There is only one thread
    let boot_count = unsafe { &mut BOOT_COUNT };
    info!("Current boot count = {boot_count}");
    *boot_count += 1;

    if let Err(error) = main_fallible(&spawner).await {
        error!("Error while running firmware: {error:?}");
    }
}

/// Main task that can return an error
async fn main_fallible(spawner: &Spawner) -> Result<(), Error> {
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::max(system.clock_control).freeze();
    embassy::init(&clocks, TimerGroup::new(peripherals.TIMG0, &clocks));

    let rng = Rng::new(peripherals.RNG);

    let clock = if let Some(clock) = Clock::from_rtc_memory() {
        info!("Clock loaded from RTC memory");
        clock
    } else {
        let ssid = String::<32>::try_from(WIFI_SSID).map_err(|()| Error::ParseCredentials)?;
        let password =
            String::<64>::try_from(WIFI_PASSWORD).map_err(|()| Error::ParseCredentials)?;

        info!("Connect to WiFi");
        let stack = connect_to_wifi(
            spawner,
            peripherals.SYSTIMER,
            rng,
            peripherals.WIFI,
            system.radio_clock_control,
            &clocks,
            (ssid, password),
        )
        .await?;

        info!("Synchronize clock from server");
        let mut http_client = HttpClient::new(stack, RngWrapper::from(rng));
        let clock = Clock::from_server(&mut http_client).await?;

        info!("Request to disconnect wifi");
        STOP_WIFI_SIGNAL.signal(());

        clock
    };

    info!("Now is {}", clock.now()?);

    let common = dd::Telemetry {
        api_version: dd::ApiVersion::V2,
        tracer_time: clock.now()?.unix_timestamp() as u64,
        runtime_id: "todo".into(),
        seq_id: 1,
        application: &dd::Application {
            service_name: String::try_from("embedded-apm").unwrap(),
            service_version: Some(String::try_from(env!("CARGO_PKG_VERSION")).unwrap()),
            env: Some(String::try_from("embedded").unwrap()),
            language_name: String::try_from("Rust").unwrap(),
            language_version: String::try_from(env!("CARGO_PKG_RUST_VERSION")).unwrap(),
            tracer_version: String::try_from(env!("CARGO_PKG_VERSION")).unwrap(),
            runtime_name: None,
            runtime_version: None,
            runtime_patches: None,
        },
        host: &dd::Host {
            hostname: String::try_from("esp32c3 - todo").unwrap(),
            os_version: None,
            container_id: None,
            os: None,
            kernel_name: None,
            kernel_release: None,
            kernel_version: None,
        },
        payload: &dd::Payload::AppStarted(dd::AppStarted {
            configuration: heapless::Vec::new(),
        }),
    };

    let json = serde_json::to_string(&common).unwrap();

    info!("JSON: {json}");

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    Ok(())
}

/// An error
#[derive(Debug)]
enum Error {
    /// An impossible error existing only to satisfy the type system
    Impossible(Infallible),

    /// Error while parsing SSID or password
    ParseCredentials,

    /// An error within WiFi operations
    #[allow(unused)]
    Wifi(WifiError),

    /// An error within clock operations
    #[allow(unused)]
    Clock(ClockError),
}

impl From<Infallible> for Error {
    fn from(error: Infallible) -> Self {
        Self::Impossible(error)
    }
}

impl From<WifiError> for Error {
    fn from(error: WifiError) -> Self {
        Self::Wifi(error)
    }
}

impl From<ClockError> for Error {
    fn from(error: ClockError) -> Self {
        Self::Clock(error)
    }
}
