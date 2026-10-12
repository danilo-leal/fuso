use base_gpui::{
    button::ButtonRoot,
    input::Input,
    slider::{SliderControl, SliderRoot, SliderThumb, SliderTrack, SliderValues},
};
use gpui::{
    Bounds, Context, ElementId, IntoElement, Render, SharedString, Window, WindowBounds,
    WindowOptions, div, linear_color_stop, linear_gradient, prelude::*, px, rgb, size,
};
use time_core::{
    City, Timeline, chrono_tz, day_night_icon, day_night_label, default_cities, is_daytime,
    search_available_cities,
};

struct Converter {
    timeline: Timeline,
    cities: Vec<City>,
    adding_city: bool,
    search_query: String,
}

impl Converter {
    fn new(timeline: Timeline) -> Self {
        Self {
            timeline,
            cities: default_cities(),
            adding_city: false,
            search_query: String::new(),
        }
    }

    fn add_city(&mut self, city: City) {
        if !self.cities.iter().any(|c| c.zone == city.zone) {
            self.cities.push(city);
            self.adding_city = false;
            self.search_query.clear();
        }
    }

    fn remove_city(&mut self, index: usize) {
        if self.cities.len() <= 1 {
            return;
        }
        let removed_zone = self.cities[index].zone;
        self.cities.remove(index);
        if self.timeline.reference == removed_zone {
            self.timeline.reference = self.cities.first().map(|c| c.zone).unwrap_or(removed_zone);
        }
    }
}

fn button(id: impl Into<ElementId>, label: impl Into<SharedString>) -> ButtonRoot {
    let label = label.into();
    ButtonRoot::new()
        .id(id)
        .style_with_state(|state, div| {
            div.px_3()
                .py_2()
                .rounded_md()
                .border_1()
                .border_color(rgb(if state.focused { 0x38bdf8 } else { 0x475569 }))
                .bg(rgb(0x1e293b))
                .text_color(rgb(0xe2e8f0))
        })
        .child(label)
}

fn remove_button(id: impl Into<ElementId>) -> ButtonRoot {
    ButtonRoot::new()
        .id(id)
        .style_with_state(|state, div| {
            div.px_2()
                .py_1()
                .rounded_md()
                .border_1()
                .border_color(rgb(if state.focused { 0xef4444 } else { 0x475569 }))
                .bg(rgb(0x1e293b))
                .text_color(rgb(0x94a3b8))
                .hover(|d| d.text_color(rgb(0xf87171)).border_color(rgb(0xef4444)))
                .text_xs()
        })
        .child("✕ Remove")
}

fn city_pick_button(id: impl Into<ElementId>, label: impl Into<SharedString>) -> ButtonRoot {
    let label = label.into();
    ButtonRoot::new()
        .id(id)
        .style_with_state(|state, div| {
            div.px_3()
                .py_1p5()
                .rounded_md()
                .border_1()
                .border_color(rgb(if state.focused { 0x38bdf8 } else { 0x334155 }))
                .bg(rgb(0x1e293b))
                .hover(|d| d.bg(rgb(0x334155)).border_color(rgb(0x38bdf8)))
                .text_color(rgb(0xe2e8f0))
                .text_sm()
        })
        .child(label)
}

fn day_night_track() -> impl IntoElement {
    div()
        .flex()
        .size_full()
        .rounded_full()
        .overflow_hidden()
        .child(div().flex_1().h_full().bg(linear_gradient(
            90.0,
            linear_color_stop(rgb(0x0f172a), 0.0), // Deep Night (00:00)
            linear_color_stop(rgb(0xf59e0b), 1.0), // Sunrise (06:00)
        )))
        .child(div().flex_1().h_full().bg(linear_gradient(
            90.0,
            linear_color_stop(rgb(0xf59e0b), 0.0), // Sunrise (06:00)
            linear_color_stop(rgb(0x38bdf8), 1.0), // Midday (12:00)
        )))
        .child(div().flex_1().h_full().bg(linear_gradient(
            90.0,
            linear_color_stop(rgb(0x38bdf8), 0.0), // Midday (12:00)
            linear_color_stop(rgb(0xf43f5e), 1.0), // Sunset (18:00)
        )))
        .child(div().flex_1().h_full().bg(linear_gradient(
            90.0,
            linear_color_stop(rgb(0xf43f5e), 0.0), // Sunset (18:00)
            linear_color_stop(rgb(0x0f172a), 1.0), // Deep Night (24:00)
        )))
}

fn slider_time_ticks() -> impl IntoElement {
    div()
        .flex()
        .justify_between()
        .items_center()
        .px_1()
        .text_xs()
        .text_color(rgb(0x64748b))
        .child(div().child("🌙 00:00"))
        .child(div().child("🌅 06:00"))
        .child(div().child("☀️ 12:00"))
        .child(div().child("🌇 18:00"))
        .child(div().child("🌙 24:00"))
}

fn day_night_badge(minute: u32) -> impl IntoElement {
    let icon = day_night_icon(minute);
    let label = day_night_label(minute);
    let (bg_color, text_color, border_color) = if is_daytime(minute) {
        (rgb(0x0c4a6e), rgb(0x38bdf8), rgb(0x0284c7))
    } else {
        (rgb(0x1e1b4b), rgb(0xa5b4fc), rgb(0x4338ca))
    };

    div()
        .flex()
        .items_center()
        .gap_1()
        .px_2()
        .py_0p5()
        .rounded_full()
        .bg(bg_color)
        .border_1()
        .border_color(border_color)
        .text_xs()
        .text_color(text_color)
        .child(icon)
        .child(label)
}

impl Render for Converter {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut references = div().flex().flex_wrap().gap_2();
        for (index, city) in self.cities.iter().enumerate() {
            let zone = city.zone;
            references =
                references.child(button(("reference", index), city.name.clone()).on_click(
                    cx.listener(move |this, _, _, cx| {
                        this.timeline.reference = zone;
                        cx.notify();
                    }),
                ));
        }
        if !self.cities.iter().any(|c| c.zone == chrono_tz::UTC) {
            references = references.child(button("reference-utc", "UTC").on_click(cx.listener(
                |this, _, _, cx| {
                    this.timeline.reference = chrono_tz::UTC;
                    cx.notify();
                },
            )));
        }

        let mut rows = div().flex().flex_col().gap_3();
        let can_remove = self.cities.len() > 1;

        for (index, city) in self.cities.iter().enumerate() {
            let entity = cx.entity();
            let delta = self.timeline.day_difference(city.zone);
            let city_minute = f64::from(self.timeline.minute_of_day(city.zone));
            let city_zone = city.zone;
            let city_name = city.name.clone();
            let slider = SliderRoot::new()
                .id(("time-slider", index))
                .min(0.0)
                .max(1440.0)
                .step(1.0)
                .value(SliderValues::Single(city_minute))
                .aria_label(format!("Time in {}", city_name))
                .on_value_change(move |values, _, _, cx| {
                    let SliderValues::Single(value) = values else {
                        panic!("Single-thumb time slider emitted a range");
                    };
                    entity.update(cx, |this, cx| {
                        this.timeline
                            .set_city_minute(city_zone, value.round() as u32);
                        cx.notify();
                    });
                })
                .child(
                    SliderControl::new()
                        .id(("time-control", index))
                        .w_full()
                        .h(px(24.0))
                        .style_with_state(move |_, control| {
                            control.debug_selector(move || {
                                if index == 0 {
                                    "time-slider".into()
                                } else {
                                    format!("time-slider-{index}")
                                }
                            })
                        })
                        .child(
                            SliderTrack::new()
                                .w_full()
                                .h(px(8.0))
                                .rounded_full()
                                .overflow_hidden()
                                .child_any(day_night_track()),
                        )
                        .child(
                            SliderThumb::new()
                                .id(("time-thumb", index))
                                .w(px(18.0))
                                .h(px(18.0))
                                .rounded_full()
                                .bg(rgb(0xffffff))
                                .border_2()
                                .border_color(rgb(0x38bdf8)),
                        ),
                );

            let minute_u32 = city_minute.round() as u32;
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
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(self.timeline.display(city.zone))
                        .child(day_night_badge(minute_u32)),
                )
                .child(div().w(px(90.0)).text_sm().child(match delta {
                    0 => "same day".into(),
                    _ => format!("{delta:+} day"),
                }));

            if can_remove {
                let remove_entity = cx.entity();
                row_header = row_header.child(remove_button(("remove-city", index)).on_click(
                    move |_, _, cx| {
                        remove_entity.update(cx, |this, cx| {
                            this.remove_city(index);
                            cx.notify();
                        });
                    },
                ));
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
                    .child(slider)
                    .child(slider_time_ticks()),
            );
        }

        let add_section = if self.adding_city {
            let weak_this = cx.weak_entity();
            let async_cx = cx.to_async();
            let search_input = Input::new()
                .id("city-search-input")
                .placeholder("Search city or time zone (e.g. Paris, Tokyo, Dubai)...")
                .default_value(self.search_query.clone())
                .w_full()
                .h(px(34.0))
                .on_value_change(move |value| {
                    let mut async_cx = async_cx.clone();
                    weak_this
                        .update(&mut async_cx, |this, cx| {
                            this.search_query = value.to_string();
                            cx.notify();
                        })
                        .ok();
                })
                .style_with_state(|state, input| {
                    input
                        .px_3()
                        .py_1()
                        .rounded_md()
                        .border_1()
                        .border_color(rgb(if state.focused { 0x38bdf8 } else { 0x475569 }))
                        .bg(rgb(0x0f172a))
                        .text_color(rgb(0xe2e8f0))
                });

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
                candidate_buttons =
                    candidate_buttons.child(city_pick_button(("candidate", i), label).on_click(
                        cx.listener(move |this, _, _, cx| {
                            this.add_city(candidate_clone.clone());
                            cx.notify();
                        }),
                    ));
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
                        .child(button("cancel-add-city", "Cancel").on_click(cx.listener(
                            |this, _, _, cx| {
                                this.adding_city = false;
                                cx.notify();
                            },
                        ))),
                )
                .child(search_input)
                .child(candidate_buttons)
        } else {
            div().child(button("open-add-city", "+ Add City").on_click(cx.listener(
                |this, _, _, cx| {
                    this.adding_city = true;
                    this.search_query.clear();
                    cx.notify();
                },
            )))
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
            .child("Base GPUI / headless widgets, styled here")
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
                    .child(button("now", "Now").on_click(cx.listener(|this, _, _, cx| {
                        this.timeline.reset_now();
                        cx.notify();
                    }))),
            )
            .child("Drag any city's slider to adjust all times. Tab to a thumb; use arrow keys.")
            .child(rows)
            .child(add_section)
            .child(
                div()
                    .text_sm()
                    .child("Dates are local. Day labels compare each city with the reference."),
            )
    }
}

fn main() {
    gpui_platform::application().run(|cx| {
        base_gpui::init(cx);
        cx.on_window_closed(|cx, _| {
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
                window.set_window_title("Time Zone Converter - Base GPUI");
                cx.new(|_| Converter::new(Timeline::now()))
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
        cx.update(base_gpui::init);
        let window = cx.open_window(size(px(820.0), px(1000.0)), |_, _| {
            Converter::new(Timeline::at(
                "2026-10-12T12:00:00Z".parse().unwrap(),
                chrono_tz::America::Sao_Paulo,
                String::new(),
            ))
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
            .update(cx, |view, _, cx| {
                view.add_city(City::new("Tokyo", chrono_tz::Asia::Tokyo));
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
