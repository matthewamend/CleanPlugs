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

        assets::register_noto_sans_light(cx);
        assets::register_noto_sans_thin(cx);

        Data {
            params: params.clone(),
            peak_meter: peak_meter.clone(),
        }
        .build(cx);

        VStack::new(cx, |cx| {
            title_column(cx);

            ParamButton::new(cx, Data::params, |params| &params.comp_or_limit).class("stack");

            ParamSlider::new(cx, Data::params, |params| &params.threshold);

            ParamSlider::new(cx, Data::params, |params| &params.ratio);

            ParamSlider::new(cx, Data::params, |params| &params.attack);

            ParamSlider::new(cx, Data::params, |params| &params.release);

            PeakMeter::new(
                cx,
                Data::peak_meter
                    .map(|peak_meter| util::gain_to_db(peak_meter.load(Ordering::Relaxed))),
                Some(Duration::from_millis(600)),
            )
            .top(Pixels(10.0));
        })
        .class("bg")
        .row_between(Pixels(0.0))
        .child_left(Stretch(1.0))
        .child_right(Stretch(1.0));
    })
}

fn title_column(cx: &mut Context) {
    HStack::new(cx, |cx| {
        Label::new(cx, "CleanComp").class("title");

        Label::new(cx, env!("CARGO_PKG_VERSION")).class("version");
    });
}
