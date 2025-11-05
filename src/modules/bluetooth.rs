use std::{
    collections::{BTreeMap, HashMap, HashSet},
    time::Duration,
};

use bar_rs_derive::Builder;
use bluer::Adapter;
use handlebars::Handlebars;
use iced::widget::button::Style;
use iced::widget::container;
use iced::{futures::SinkExt, stream, widget::text, Element, Subscription};
use tokio::{io, time::sleep};

use crate::button::button;
use crate::config::popup_config::{PopupConfig, PopupConfigOverride};
use crate::{
    config::{
        anchor::BarAnchor,
        module_config::{LocalModuleConfig, ModuleConfigOverride},
    },
    fill::FillExt,
    Message, NERD_FONT,
};
use crate::{impl_on_click, impl_wrapper};

use super::Module;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Device {
    pub icon: &'static str,
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct Controller {
    pub is_powered: bool,
    pub connected_devices: HashSet<Device>,
    // TODO show more information and control pannel when clicked
    // pub paired_devices: Vec<String>,
    // pub adapter: Adapter,
    // pub name: String,
    // pub is_pairable: bool,
    // pub is_discoverable: bool,
}

#[derive(Debug, Builder)]
pub struct BluetoothMod {
    controllers: Vec<Controller>,
    cfg_override: ModuleConfigOverride,
    popup_cfg_override: PopupConfigOverride,
    icons: BTreeMap<bool, &'static str>,
}

impl Default for BluetoothMod {
    fn default() -> Self {
        Self {
            controllers: Vec::new(),
            cfg_override: Default::default(),
            popup_cfg_override: PopupConfigOverride {
                width: Some(250),
                height: Some(250),
                ..Default::default()
            },
            icons: BTreeMap::from([(true, ""), (false, "")]),
        }
    }
}

impl BluetoothMod {
    fn icon(&self) -> &'static str {
        let enabled = self.controllers.iter().any(|c| c.is_powered);
        self.icons
            .get(&enabled)
            .expect("bluetooth icon already set in module")
    }
    fn connected_devices(&self) -> HashSet<&Device> {
        let mut devices = HashSet::new();
        for c in self.controllers.iter() {
            devices.extend(&c.connected_devices);
        }
        devices
    }
}

impl Module for BluetoothMod {
    fn name(&self) -> String {
        "bluetooth".to_string()
    }

    fn view(
        &self,
        config: &LocalModuleConfig,
        popup_config: &PopupConfig,
        anchor: &BarAnchor,
        _handlebars: &Handlebars,
    ) -> Element<Message> {
        let connected_devices = self.connected_devices();
        let bt_text = match connected_devices.len() {
            0 => self.icon().to_string(),
            // show name if only one connected device
            1 => {
                let device = connected_devices.iter().next().unwrap();
                format!("{} {}", device.icon, device.name)
            }
            // show icons for connected bluetooth devices
            _ => connected_devices
                .iter()
                .fold(String::new(), |mut acc, elem| {
                    acc.push_str(elem.icon);
                    acc
                }),
        };

        button(
            list![
                anchor,
                container(
                    text(bt_text)
                        .fill(anchor)
                        .color(self.cfg_override.icon_color.unwrap_or(config.icon_color))
                        .size(self.cfg_override.icon_size.unwrap_or(config.icon_size))
                        .font(NERD_FONT)
                )
                .padding(self.cfg_override.icon_margin.unwrap_or(config.icon_margin))
            ]
            .spacing(self.cfg_override.spacing.unwrap_or(config.spacing)),
        )
        .on_event_with(Message::popup::<Self>(
            self.popup_cfg_override.width.unwrap_or(popup_config.width),
            self.popup_cfg_override
                .height
                .unwrap_or(popup_config.height),
            anchor,
        ))
        .style(|_, _| Style::default())
        .into()
    }

    impl_wrapper!();

    fn read_config(
        &mut self,
        config: &HashMap<String, Option<String>>,
        popup_config: &HashMap<String, Option<String>>,
        _templates: &mut Handlebars,
    ) {
        self.cfg_override = config.into();
        self.popup_cfg_override.update(popup_config);
    }

    impl_on_click!();

    fn subscription(&self) -> Option<iced::Subscription<Message>> {
        Some(Subscription::run(|| {
            stream::channel(1, |mut sender| async move {
                if let Ok(mut session) = bluer::Session::new().await {
                    loop {
                        let controllers = get_controllers(&mut session).await.unwrap();
                        if sender
                            .send(Message::update(move |reg| {
                                let m = reg.get_module_mut::<BluetoothMod>();
                                m.controllers = controllers
                            }))
                            .await
                            .is_err()
                        {
                            return;
                        }
                        sleep(Duration::from_secs(1)).await;
                    }
                }
            })
        }))
    }
}
async fn get_controllers(session: &mut bluer::Session) -> Result<Vec<Controller>, io::Error> {
    let mut controllers: Vec<Controller> = Vec::new();
    let adapter_names = session.adapter_names().await?;
    for adapter_name in adapter_names {
        if let Ok(adapter) = session.adapter(&adapter_name) {
            let is_powered = adapter.is_powered().await?;
            // let name = adapter.name().to_owned();
            // let is_pairable = adapter.is_pairable().await?;
            // let is_discoverable = adapter.is_discoverable().await?;

            let connected_devices = get_all_devices(&adapter).await?;

            let controller = Controller {
                is_powered,
                connected_devices,
                // name,
                // is_pairable,
                // is_discoverable,
            };
            controllers.push(controller);
        }
    }
    Ok(controllers)
}
pub async fn get_all_devices(adapter: &Adapter) -> Result<HashSet<Device>, io::Error> {
    // TODO get paired_deviced at the same time

    let mut connected_devices = HashSet::new();

    let connected_devices_addresses = adapter.device_addresses().await?;
    for addr in connected_devices_addresses {
        let device = adapter.device(addr)?;

        let icon = match device.icon().await?.unwrap_or("None".to_string()).as_ref() {
            "audio-card" => "󰓃",
            "audio-input-microphone" => "",
            "audio-headphones" | "audio-headset" => "󰋋",
            "battery" => "󰂀",
            "camera-photo" => "󰻛",
            "computer" => "",
            "input-keyboard" => "󰌌",
            "input-mouse" => "󰍽",
            "input-gaming" => "󰊴",
            "phone" => "󰏲",
            "None" => "",
            _ => "",
        };
        if device.is_connected().await? {
            connected_devices.insert(Device {
                icon,
                name: device.alias().await?,
            });
        }
    }
    Ok(connected_devices)
}
