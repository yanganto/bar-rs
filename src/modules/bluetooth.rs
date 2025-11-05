use std::collections::BTreeMap;
use std::collections::HashMap;

use bar_rs_derive::Builder;
use handlebars::Handlebars;
use iced::widget::button::Style;
use iced::widget::container;
use iced::{widget::text, Element};

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

#[derive(Debug, Builder)]
pub struct BluetoothMod {
    cfg_override: ModuleConfigOverride,
    popup_cfg_override: PopupConfigOverride,
    icons: BTreeMap<bool, String>,
}

impl Default for BluetoothMod {
    fn default() -> Self {
        Self {
            cfg_override: Default::default(),
            popup_cfg_override: PopupConfigOverride {
                width: Some(250),
                height: Some(250),
                ..Default::default()
            },
            icons: BTreeMap::from([(true, "".to_string()), (false, "".to_string())]),
        }
    }
}

impl BluetoothMod {
    fn icon(&self) -> &String {
        self.icons
            .get(&false)
            .expect("bluetooth icon already set in module")
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
        button(
            list![
                anchor,
                container(
                    text(self.icon())
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
        None
    }
}
