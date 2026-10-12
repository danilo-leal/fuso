use gpui::{
    Application, Bounds, Context, Entity, IntoElement, Render, Subscription, Window, WindowBounds,
    WindowOptions, div, prelude::*, px, rgb, size,
};
use gpui_component::{
    Root, Sizable as _,
    button::{Button, ButtonVariants as _},
    input::{Input, InputEvent, InputState},
    slider::{Slider, SliderEvent, SliderState},
};
use time_core::{City, Timeline, chrono_tz, default_cities, search_available_cities};

struct Converter {
    timeline: Timeline,
    cities: Vec<City>,
    sliders: Vec<Entity<SliderState>>,
    _subscriptions: Vec<Subscription>,
    adding_city: bool,
    search_input: Entity<InputState>,
    search_query: String,
    _search_sub: Subscription,
}

impl Converter {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self::with_timeline(Timeline::now(), window, cx)
    }

    fn with_timeline(timeline: Timeline, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let cities = default_cities();
        let mut sliders = Vec::new();
        let mut subscriptions = Vec::new();

        for city in &cities {
            let minute = timeline.minute_of_day(city.zone) as f32;
            let slider = cx.new(|_| SliderState::new().min(0.0).max(1440.0).step(1.0));
            slider.update(cx, |slider, cx| slider.set_value(minute, window, cx));

            let zone = city.zone;
            let sub = cx.subscribe(&slider, move |this, _, event, cx| {
                let SliderEvent::Change(value) = event;
                this.timeline
                    .set_city_minute(zone, value.start().round() as u32);
                cx.notify();
            });

            sliders.push(slider);
            subscriptions.push(sub);
        }

        let search_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Search city or time zone (e.g. Paris, Tokyo, Dubai)...")
        });

        let _search_sub = cx.subscribe(&search_input, |this, input, event: &InputEvent, cx| {
            if matches!(event, InputEvent::Change) {
                this.search_query = input.read(cx).text().to_string();
                cx.notify();
            }
        });

        Self {
            timeline,
            cities,
            sliders,
            _subscriptions: subscriptions,
            adding_city: false,
            search_input,
            search_query: String::new(),
            _search_sub,
        }
    }

    fn add_city(&mut self, city: City, window: &mut Window, cx: &mut Context<Self>) {
        if self.cities.iter().any(|c| c.zone == city.zone) {
            return;
        }

        let minute = self.timeline.minute_of_day(city.zone) as f32;
        let slider = cx.new(|_| SliderState::new().min(0.0).max(1440.0).step(1.0));
        slider.update(cx, |slider, cx| slider.set_value(minute, window, cx));

        let zone = city.zone;
        let sub = cx.subscribe(&slider, move |this, _, event, cx| {
            let SliderEvent::Change(value) = event;
            this.timeline
                .set_city_minute(zone, value.start().round() as u32);
            cx.notify();
        });

        self.cities.push(city);
        self.sliders.push(slider);
        self._subscriptions.push(sub);
    }

    fn remove_city(&mut self, index: usize) {
        if self.cities.len() <= 1 {
            return;
        }
        let removed_zone = self.cities[index].zone;
        self.cities.remove(index);
        self.sliders.remove(index);
        let _ = self._subscriptions.remove(index);
        if self.timeline.reference == removed_zone {
            self.timeline.reference = self.cities.first().map(|c| c.zone).unwrap_or(removed_zone);
        }
    }
}

impl Render for Converter {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        for (index, city) in self.cities.iter().enumerate() {
            let minute = self.timeline.minute_of_day(city.zone) as f32;
            self.sliders[index].update(cx, |slider, cx| {
                if (slider.value().start() - minute).abs() >= 0.5 {
                    slider.set_value(minute, window, cx);
                }
            });
        }

        let mut references = div().flex().flex_wrap().gap_2();
        for (index, city) in self.cities.iter().enumerate() {
            let zone = city.zone;
            references = references.child(
                Button::new(("reference", index))
                    .label(city.name.clone())
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.timeline.reference = zone;
                        cx.notify();
                    })),
            );
        }
        if !self.cities.iter().any(|c| c.zone == chrono_tz::UTC) {
            references = references.child(Button::new("reference-utc").label("UTC").on_click(
                cx.listener(|this, _, _, cx| {
                    this.timeline.reference = chrono_tz::UTC;
                    cx.notify();
                }),
            ));
        }

        let can_remove = self.cities.len() > 1;
        let mut rows = div().flex().flex_col().gap_3();
        for (index, city) in self.cities.iter().enumerate() {
            let delta = self.timeline.day_difference(city.zone);
            let mut row_header = div()
                .flex()
                .justify_between()
                .items_center()
                .child(
                    div()
                        .w(px(140.0))
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .child(city.name.clone()),
                )
                .child(div().flex_1().child(self.timeline.display(city.zone)))
                .child(div().w(px(90.0)).text_sm().child(match delta {
                    0 => "same day".into(),
                    _ => format!("{delta:+} day"),
                }));

            if can_remove {
                row_header = row_header.child(
                    Button::new(("remove-city", index))
                        .label("✕ Remove")
                        .ghost()
                        .small()
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.remove_city(index);
                            cx.notify();
                        })),
                );
            }

            rows = rows.child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .p_3()
                    .rounded_md()
                    .bg(rgb(0x1e293b))
                    .child(row_header)
                    .child(
                        div()
                            .w_full()
                            .debug_selector(move || {
                                if index == 0 {
                                    "time-slider".into()
                                } else {
                                    format!("time-slider-{index}")
                                }
                            })
                            .child(Slider::new(&self.sliders[index]).horizontal().w_full()),
                    ),
            );
        }

        let add_section = if self.adding_city {
            let search_results = search_available_cities(&self.search_query);
            let available: Vec<_> = search_results
                .into_iter()
                .filter(|c| !self.cities.iter().any(|existing| existing.zone == c.zone))
                .take(24)
                .collect();

            let mut candidate_buttons = div()
                .id("candidates-scroll")
                .flex()
                .flex_wrap()
                .gap_2()
                .max_h(px(180.0))
                .overflow_y_scroll();

            for (i, candidate) in available.into_iter().enumerate() {
                let label = format!("+ {} ({})", candidate.name, candidate.zone.name());
                let candidate_clone = candidate.clone();
                candidate_buttons = candidate_buttons.child(
                    Button::new(("candidate", i))
                        .label(label)
                        .small()
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.add_city(candidate_clone.clone(), window, cx);
                            cx.notify();
                        })),
                );
            }

            div()
                .flex()
                .flex_col()
                .gap_3()
                .p_4()
                .rounded_md()
                .bg(rgb(0x1e293b))
                .border_1()
                .border_color(rgb(0x38bdf8))
                .child(
                    div()
                        .flex()
                        .justify_between()
                        .items_center()
                        .child(div().child("Add City"))
                        .child(
                            Button::new("cancel-add-city")
                                .label("Cancel")
                                .small()
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.adding_city = false;
                                    this.search_query.clear();
                                    this.search_input.update(cx, |input, cx| {
                                        input.set_value("", window, cx);
                                    });
                                    cx.notify();
                                })),
                        ),
                )
                .child(Input::new(&self.search_input).bordered(true))
                .child(candidate_buttons)
        } else {
            div().child(
                Button::new("open-add-city")
                    .label("+ Add City")
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.adding_city = true;
                        this.search_query.clear();
                        this.search_input.update(cx, |input, cx| {
                            input.set_value("", window, cx);
                        });
                        cx.notify();
                    })),
            )
        };

        div()
            .id("converter")
            .size_full()
            .overflow_y_scroll()
            .flex()
            .flex_col()
            .p_6()
            .gap_4()
            .bg(rgb(0x0f172a))
            .text_color(rgb(0xe2e8f0))
            .child(div().text_2xl().child("Time Zone Converter"))
            .child(div().text_sm().child(self.timeline.zone_detection.clone()))
            .child("Reference city")
            .child(references)
            .child(format!("Reference: {}", self.timeline.reference))
            .child(
                div()
                    .text_xl()
                    .child(self.timeline.display(self.timeline.reference)),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(self.timeline.offset_label())
                    .child(Button::new("now").label("Now").on_click(cx.listener(
                        |this, _, _, cx| {
                            this.timeline.reset_now();
                            cx.notify();
                        },
                    ))),
            )
            .child(rows)
            .child(add_section)
    }
}

fn main() {
    Application::new().run(|cx| {
        gpui_component::init(cx);
        cx.on_window_closed(|cx| {
            if cx.windows().is_empty() {
                cx.quit();
            }
        })
        .detach();
        let bounds = Bounds::centered(None, size(px(820.0), px(860.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                window_min_size: Some(size(px(720.0), px(700.0))),
                ..Default::default()
            },
            |window, cx| {
                window.set_window_title("Time Zone Converter - GPUI Component");
                let view = cx.new(|cx| Converter::new(window, cx));
                cx.new(|cx| Root::new(view, window, cx))
            },
        )
        .expect("Failed to open Time Zone Converter window");
        cx.activate(true);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{Modifiers, MouseButton, TestAppContext, VisualTestContext, point};

    #[gpui::test]
    fn slider_updates_the_shared_timeline(cx: &mut TestAppContext) {
        cx.update(gpui_component::init);
        let window = cx.update(|cx| {
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(Bounds::new(
                        point(px(0.0), px(0.0)),
                        size(px(820.0), px(860.0)),
                    ))),
                    ..Default::default()
                },
                |window, cx| {
                    let timeline = Timeline::at(
                        "2026-10-12T12:00:00Z".parse().unwrap(),
                        chrono_tz::America::Sao_Paulo,
                        String::new(),
                    );
                    cx.new(|cx| Converter::with_timeline(timeline, window, cx))
                },
            )
            .unwrap()
        });
        cx.run_until_parked();
        let mut visual = VisualTestContext::from_window(window.into(), cx);

        // Starts with 3 cities: Sao Paulo (0), New York (1), London (2)
        let initial_selectors = ["time-slider", "time-slider-1", "time-slider-2"];
        for (i, selector) in initial_selectors.iter().enumerate() {
            assert!(
                visual.debug_bounds(selector).is_some(),
                "City slider {i} should be visible"
            );
        }
        assert!(
            visual.debug_bounds("time-slider-3").is_none(),
            "4th slider should not exist initially"
        );

        let (sp_initial, london_initial) = window
            .update(cx, |view, _, _| {
                (
                    view.timeline.minute_of_day(view.cities[0].zone),
                    view.timeline.minute_of_day(view.cities[2].zone),
                )
            })
            .unwrap();
        assert_eq!(sp_initial, 540);
        assert_eq!(london_initial, 780);

        let sp_bounds = visual
            .debug_bounds("time-slider")
            .expect("Sao Paulo slider is visible");
        let position = point(
            sp_bounds.origin.x + sp_bounds.size.width * 0.75,
            sp_bounds.center().y,
        );
        visual.simulate_mouse_down(position, MouseButton::Left, Modifiers::default());
        visual.simulate_mouse_up(position, MouseButton::Left, Modifiers::default());
        visual.run_until_parked();

        let (sp_after, london_after) = window
            .update(cx, |view, _, _| {
                (
                    view.timeline.minute_of_day(view.cities[0].zone),
                    view.timeline.minute_of_day(view.cities[2].zone),
                )
            })
            .unwrap();
        assert!(
            (1070..=1090).contains(&sp_after),
            "Expected Sao Paulo roughly 1080m; got {sp_after}"
        );
        assert!(
            (1310..=1330).contains(&london_after),
            "Expected London to adjust to roughly 1320m; got {london_after}"
        );

        // Add Tokyo
        window
            .update(cx, |view, window, cx| {
                view.add_city(City::new("Tokyo", chrono_tz::Asia::Tokyo), window, cx);
                cx.notify();
            })
            .unwrap();
        visual.run_until_parked();

        let tokyo_bounds = visual
            .debug_bounds("time-slider-3")
            .expect("Tokyo slider should now be visible");
        let tokyo_center = point(
            tokyo_bounds.origin.x + tokyo_bounds.size.width * 0.50,
            tokyo_bounds.center().y,
        );
        visual.simulate_mouse_down(tokyo_center, MouseButton::Left, Modifiers::default());
        visual.simulate_mouse_up(tokyo_center, MouseButton::Left, Modifiers::default());
        visual.run_until_parked();

        let tokyo_final = window
            .update(cx, |view, _, _| {
                view.timeline.minute_of_day(chrono_tz::Asia::Tokyo)
            })
            .unwrap();
        assert!(
            (710..=730).contains(&tokyo_final),
            "Expected Tokyo roughly 720m (12:00); got {tokyo_final}"
        );

        // Remove a city (New York at index 1)
        window
            .update(cx, |view, _, cx| {
                view.remove_city(1);
                cx.notify();
            })
            .unwrap();
        visual.run_until_parked();

        let count = window.update(cx, |view, _, _| view.cities.len()).unwrap();
        assert_eq!(count, 3); // Started with 3, added 1, removed 1 => 3
    }
}
