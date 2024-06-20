use atomic_float::AtomicF32;
use nih_plug::prelude::{util, Editor};
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::widgets::*;
use nih_plug_vizia::{assets, create_vizia_editor, ViziaState, ViziaTheming};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

use crate::CleanCompParams;

#[derive(Lens)]
struct Data {
    params: Arc<CleanCompParams>,
    peak_meter: Arc<AtomicF32>,
}

impl Model for Data {}

pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::new(|| (400, 300))
}

pub(crate) fn create(
    params: Arc<CleanCompParams>,
    peak_meter: Arc<AtomicF32>,
    editor_state: Arc<ViziaState>,
) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, ViziaTheming::Custom, move |cx, _| {
        cx.add_stylesheet(include_style!("src/style.css"))
            .expect("Failed to load stylesheet");

        Data {
            params: params.clone(),
            peak_meter: peak_meter.clone(),
        }
        .build(cx);

        VStack::new(cx, |cx| {
            title(cx);

            body(cx);
        });
    })
}

fn title(cx: &mut Context) {
    HStack::new(cx, |cx| {
        Label::new(cx, "CleanComp").class("title");

        Label::new(cx, env!("CARGO_PKG_VERSION")).class("version");
    });
}

fn body(cx: &mut Context) {
    HStack::new(cx, |cx| {});
}
