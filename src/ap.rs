use alloc::string::String;
use defmt::info;
use embassy_net::Runner;
use embassy_time::{Duration, Timer};
use esp_radio::wifi::{
    AccessPointConfig, AuthMethod, ModeConfig, WifiApState, WifiController, WifiDevice, WifiEvent,
};

#[embassy_executor::task]
pub async fn start_ap(mut controller: WifiController<'static>) {
    info!("start connection task");
    info!("Device capabilities: {:?}", controller.capabilities());
    loop {
        if esp_radio::wifi::ap_state() == WifiApState::Started {
            // wait until we're no longer connected
            controller.wait_for_event(WifiEvent::ApStop).await;
            Timer::after(Duration::from_millis(5000)).await
        }
        if !matches!(controller.is_started(), Ok(true)) {
            let client_config = ModeConfig::AccessPoint(
                AccessPointConfig::default()
                    .with_auth_method(AuthMethod::Wpa2Personal)
                    .with_ssid("esp-radio".into())
                    .with_password(String::from("password")),
            );
            controller.set_config(&client_config).unwrap();
            info!("Starting wifi");
            controller.start_async().await.unwrap();
            info!("Wifi started!");
        }
    }
}

#[embassy_executor::task]
pub async fn net_task(mut runner: Runner<'static, WifiDevice<'static>>) {
    runner.run().await
}
