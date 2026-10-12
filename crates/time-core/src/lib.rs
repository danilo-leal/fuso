use chrono::{DateTime, Duration, Timelike, Utc};
pub use chrono_tz;
pub use chrono_tz::Tz;

pub const MIN_OFFSET: i32 = -1440;
pub const MAX_OFFSET: i32 = 1440;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct City {
    pub name: String,
    pub zone: Tz,
}

impl City {
    pub fn new(name: impl Into<String>, zone: Tz) -> Self {
        Self {
            name: name.into(),
            zone,
        }
    }
}

pub static POPULAR_CITIES: &[(&str, Tz)] = &[
    ("Sao Paulo", chrono_tz::America::Sao_Paulo),
    ("New York", chrono_tz::America::New_York),
    ("London", chrono_tz::Europe::London),
    ("Tokyo", chrono_tz::Asia::Tokyo),
    ("Sydney", chrono_tz::Australia::Sydney),
    ("Paris", chrono_tz::Europe::Paris),
    ("Los Angeles", chrono_tz::America::Los_Angeles),
    ("Chicago", chrono_tz::America::Chicago),
    ("Denver", chrono_tz::America::Denver),
    ("Toronto", chrono_tz::America::Toronto),
    ("Vancouver", chrono_tz::America::Vancouver),
    ("Mexico City", chrono_tz::America::Mexico_City),
    ("Buenos Aires", chrono_tz::America::Argentina::Buenos_Aires),
    ("Santiago", chrono_tz::America::Santiago),
    ("Bogota", chrono_tz::America::Bogota),
    ("Berlin", chrono_tz::Europe::Berlin),
    ("Madrid", chrono_tz::Europe::Madrid),
    ("Rome", chrono_tz::Europe::Rome),
    ("Amsterdam", chrono_tz::Europe::Amsterdam),
    ("Zurich", chrono_tz::Europe::Zurich),
    ("Dublin", chrono_tz::Europe::Dublin),
    ("Stockholm", chrono_tz::Europe::Stockholm),
    ("Athens", chrono_tz::Europe::Athens),
    ("Moscow", chrono_tz::Europe::Moscow),
    ("Istanbul", chrono_tz::Europe::Istanbul),
    ("Cairo", chrono_tz::Africa::Cairo),
    ("Johannesburg", chrono_tz::Africa::Johannesburg),
    ("Nairobi", chrono_tz::Africa::Nairobi),
    ("Lagos", chrono_tz::Africa::Lagos),
    ("Dubai", chrono_tz::Asia::Dubai),
    ("Riyadh", chrono_tz::Asia::Riyadh),
    ("Mumbai", chrono_tz::Asia::Kolkata),
    ("Bangkok", chrono_tz::Asia::Bangkok),
    ("Singapore", chrono_tz::Asia::Singapore),
    ("Hong Kong", chrono_tz::Asia::Hong_Kong),
    ("Shanghai", chrono_tz::Asia::Shanghai),
    ("Seoul", chrono_tz::Asia::Seoul),
    ("Jakarta", chrono_tz::Asia::Jakarta),
    ("Manila", chrono_tz::Asia::Manila),
    ("Taipei", chrono_tz::Asia::Taipei),
    ("Perth", chrono_tz::Australia::Perth),
    ("Melbourne", chrono_tz::Australia::Melbourne),
    ("Auckland", chrono_tz::Pacific::Auckland),
    ("Honolulu", chrono_tz::Pacific::Honolulu),
    ("UTC", chrono_tz::UTC),
];

pub fn default_cities() -> Vec<City> {
    vec![
        City::new("Sao Paulo", chrono_tz::America::Sao_Paulo),
        City::new("New York", chrono_tz::America::New_York),
        City::new("London", chrono_tz::Europe::London),
    ]
}

pub fn friendly_tz_name(tz: Tz) -> String {
    let name = tz.name();
    if let Some((_, city)) = name.split_once('/') {
        let city = city.split('/').next_back().unwrap_or(city);
        city.replace('_', " ")
    } else {
        name.to_string()
    }
}

pub fn search_available_cities(query: &str) -> Vec<City> {
    let trimmed = query.trim().to_lowercase();
    let mut results = Vec::new();
    let mut seen_zones = std::collections::HashSet::new();

    // First check popular cities
    for &(name, zone) in POPULAR_CITIES {
        let matches = trimmed.is_empty()
            || name.to_lowercase().contains(&trimmed)
            || zone.name().to_lowercase().contains(&trimmed);
        if matches && seen_zones.insert(zone) {
            results.push(City::new(name, zone));
        }
    }

    // If query is non-empty, also search all TZ_VARIANTS
    if !trimmed.is_empty() {
        for zone in chrono_tz::TZ_VARIANTS {
            let zone_name = zone.name();
            let friendly = friendly_tz_name(zone);
            let matches = zone_name.to_lowercase().contains(&trimmed)
                || friendly.to_lowercase().contains(&trimmed);
            if matches && seen_zones.insert(zone) {
                results.push(City::new(friendly, zone));
            }
        }
    }

    results
}

pub struct Timeline {
    anchor: DateTime<Utc>,
    offset_minutes: i32,
    pub reference: Tz,
    pub zone_detection: String,
}

impl Timeline {
    pub fn now() -> Self {
        let (reference, zone_detection) = match iana_time_zone::get_timezone() {
            Ok(name) => match name.parse::<Tz>() {
                Ok(zone) => (
                    zone,
                    format!("Mac time zone: {name}. Choose a city to override."),
                ),
                Err(error) => {
                    eprintln!("Cannot interpret system time zone {name}: {error}");
                    (
                        chrono_tz::UTC,
                        format!(
                            "Unrecognized system zone {name}; using UTC. Choose a reference city."
                        ),
                    )
                }
            },
            Err(error) => {
                eprintln!("Cannot detect system time zone: {error}");
                (
                    chrono_tz::UTC,
                    "System zone unavailable; using UTC. Choose a reference city.".into(),
                )
            }
        };
        Self::at(Utc::now(), reference, zone_detection)
    }

    pub fn at(anchor: DateTime<Utc>, reference: Tz, zone_detection: String) -> Self {
        Self {
            anchor: anchor.with_second(0).unwrap().with_nanosecond(0).unwrap(),
            offset_minutes: 0,
            reference,
            zone_detection,
        }
    }

    pub fn instant(&self) -> DateTime<Utc> {
        self.anchor + Duration::minutes(i64::from(self.offset_minutes))
    }

    pub fn offset_minutes(&self) -> i32 {
        self.offset_minutes
    }

    pub fn set_offset(&mut self, minutes: i32) {
        self.offset_minutes = minutes.clamp(MIN_OFFSET, MAX_OFFSET);
    }

    pub fn reset_now(&mut self) {
        self.anchor = Utc::now()
            .with_second(0)
            .unwrap()
            .with_nanosecond(0)
            .unwrap();
        self.offset_minutes = 0;
    }

    pub fn display(&self, zone: Tz) -> String {
        self.instant()
            .with_timezone(&zone)
            .format("%a %Y-%m-%d  %H:%M  %Z (%:z)")
            .to_string()
    }

    pub fn day_difference(&self, zone: Tz) -> i64 {
        let instant = self.instant();
        (instant.with_timezone(&zone).date_naive()
            - instant.with_timezone(&self.reference).date_naive())
        .num_days()
    }

    pub fn offset_label(&self) -> String {
        let minutes = self.offset_minutes;
        format!(
            "{}{:02}h {:02}m from anchor",
            if minutes < 0 { "-" } else { "+" },
            minutes.abs() / 60,
            minutes.abs() % 60
        )
    }

    pub fn minute_of_day(&self, zone: Tz) -> u32 {
        let local = self.instant().with_timezone(&zone);
        local.hour() * 60 + local.minute()
    }

    pub fn set_city_minute(&mut self, zone: Tz, target_minute: u32) {
        let current_minute = self.minute_of_day(zone) as i32;
        let target = (target_minute as i32) % 1440;
        let mut delta = target - current_minute;
        if delta > 720 {
            delta -= 1440;
        } else if delta < -720 {
            delta += 1440;
        }
        self.set_offset(self.offset_minutes + delta);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn timeline(instant: &str) -> Timeline {
        Timeline::at(
            instant.parse().unwrap(),
            chrono_tz::America::Sao_Paulo,
            String::new(),
        )
    }

    #[test]
    fn cities_share_an_instant_and_show_date_rollover() {
        let t = timeline("2026-10-12T23:30:00Z");
        assert!(
            t.display(chrono_tz::America::Sao_Paulo)
                .contains("2026-10-12  20:30")
        );
        assert!(
            t.display(chrono_tz::Asia::Tokyo)
                .contains("2026-10-13  08:30")
        );
        assert_eq!(t.day_difference(chrono_tz::Asia::Tokyo), 1);
    }

    #[test]
    fn slider_crosses_dst_spring_gap_without_inventing_local_times() {
        let mut t = timeline("2026-03-08T06:30:00Z");
        assert!(
            t.display(chrono_tz::America::New_York)
                .contains("01:30  EST")
        );
        t.set_offset(60);
        assert!(
            t.display(chrono_tz::America::New_York)
                .contains("03:30  EDT")
        );
    }

    #[test]
    fn dst_repeated_hour_has_distinct_offsets() {
        let mut t = timeline("2026-11-01T05:30:00Z");
        assert!(
            t.display(chrono_tz::America::New_York)
                .contains("01:30  EDT (-04:00)")
        );
        t.set_offset(60);
        assert!(
            t.display(chrono_tz::America::New_York)
                .contains("01:30  EST (-05:00)")
        );
    }

    #[test]
    fn reference_selection_preserves_instant_and_changes_rollover_label() {
        let mut t = timeline("2026-10-12T23:30:00Z");
        let instant = t.instant();
        t.reference = chrono_tz::Asia::Tokyo;
        assert_eq!(t.instant(), instant);
        assert_eq!(t.day_difference(chrono_tz::America::Sao_Paulo), -1);
    }

    #[test]
    fn minute_precision_and_slider_limits() {
        let mut t = timeline("2026-10-12T23:30:59Z");
        assert_eq!(t.instant().second(), 0);
        t.set_offset(9999);
        assert_eq!(t.offset_minutes(), MAX_OFFSET);
        t.set_offset(-9999);
        assert_eq!(t.offset_minutes(), MIN_OFFSET);
        assert_eq!(t.offset_label(), "-24h 00m from anchor");
    }

    #[test]
    fn adjusting_city_slider_adjusts_other_cities() {
        let mut t = timeline("2026-10-12T12:00:00Z");
        // London is 13:00 (minute 780), Tokyo is 21:00 (minute 1260)
        assert_eq!(t.minute_of_day(chrono_tz::Europe::London), 780);
        assert_eq!(t.minute_of_day(chrono_tz::Asia::Tokyo), 1260);

        // Adjust Tokyo from 21:00 (1260) to 23:00 (1380) (+2 hours)
        t.set_city_minute(chrono_tz::Asia::Tokyo, 1380);
        assert_eq!(t.minute_of_day(chrono_tz::Asia::Tokyo), 1380);
        // London should adjust by +2 hours to 15:00 (900)
        assert_eq!(t.minute_of_day(chrono_tz::Europe::London), 900);
        assert_eq!(t.offset_minutes(), 120);

        // Adjust London across midnight to 01:00 (60)
        t.set_city_minute(chrono_tz::Europe::London, 60);
        assert_eq!(t.minute_of_day(chrono_tz::Europe::London), 60);
        // Tokyo was at 23:00 (1380), London advanced +10 hours, so Tokyo advances to 09:00 (540)
        assert_eq!(t.minute_of_day(chrono_tz::Asia::Tokyo), 540);
        assert_eq!(t.offset_minutes(), 720);
    }

    #[test]
    fn default_cities_has_three_items() {
        let cities = default_cities();
        assert_eq!(cities.len(), 3);
        assert_eq!(cities[0].name, "Sao Paulo");
        assert_eq!(cities[1].name, "New York");
        assert_eq!(cities[2].name, "London");
    }

    #[test]
    fn search_available_cities_finds_popular_and_iana_zones() {
        let empty_search = search_available_cities("");
        assert!(empty_search.len() >= 40);

        let paris_results = search_available_cities("paris");
        assert!(
            paris_results
                .iter()
                .any(|c| c.name == "Paris" && c.zone == chrono_tz::Europe::Paris)
        );

        let bogota_results = search_available_cities("bogota");
        assert!(
            bogota_results
                .iter()
                .any(|c| c.zone == chrono_tz::America::Bogota)
        );
    }
}
