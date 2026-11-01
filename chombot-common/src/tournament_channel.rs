use std::fmt::Write;

use slug::slugify;

use crate::channel_creator::ChannelName;
use crate::tournaments_watcher::ema::TournamentEntry;

/// English month names and their Polish genitive forms ("14 lutego").
const MONTHS: [(&str, &str); 12] = [
    ("january", "stycznia"),
    ("february", "lutego"),
    ("march", "marca"),
    ("april", "kwietnia"),
    ("may", "maja"),
    ("june", "czerwca"),
    ("july", "lipca"),
    ("august", "sierpnia"),
    ("september", "września"),
    ("october", "października"),
    ("november", "listopada"),
    ("december", "grudnia"),
];

/// Polish names of cities, keyed by the slug of the EMA calendar's spelling.
/// Other cities are used as the calendar writes them.
const POLISH_CITY_NAMES: &[(&str, &str)] = &[
    ("aachen", "Akwizgran"),
    ("antwerp", "Antwerpia"),
    ("antwerpen", "Antwerpia"),
    ("athens", "Ateny"),
    ("athina", "Ateny"),
    ("basel", "Bazylea"),
    ("belgrade", "Belgrad"),
    ("beograd", "Belgrad"),
    ("bern", "Berno"),
    ("berne", "Berno"),
    ("bialystok", "Białystok"),
    ("bologna", "Bolonia"),
    ("bratislava", "Bratysława"),
    ("bruges", "Brugia"),
    ("brugge", "Brugia"),
    ("brussel", "Bruksela"),
    ("brussels", "Bruksela"),
    ("bruxelles", "Bruksela"),
    ("bucharest", "Bukareszt"),
    ("bucuresti", "Bukareszt"),
    ("budapest", "Budapeszt"),
    ("cluj-napoca", "Kluż-Napoka"),
    ("cologne", "Kolonia"),
    ("copenhagen", "Kopenhaga"),
    ("cracow", "Kraków"),
    ("czestochowa", "Częstochowa"),
    ("den-haag", "Haga"),
    ("dresden", "Drezno"),
    ("edinburgh", "Edynburg"),
    ("firenze", "Florencja"),
    ("florence", "Florencja"),
    ("frankfurt-am-main", "Frankfurt nad Menem"),
    ("gdansk", "Gdańsk"),
    ("geneva", "Genewa"),
    ("geneve", "Genewa"),
    ("genoa", "Genua"),
    ("genova", "Genua"),
    ("gent", "Gandawa"),
    ("ghent", "Gandawa"),
    ("gothenburg", "Göteborg"),
    ("hannover", "Hanower"),
    ("hanover", "Hanower"),
    ("istanbul", "Stambuł"),
    ("kaunas", "Kowno"),
    ("kharkiv", "Charków"),
    ("kharkov", "Charków"),
    ("kiev", "Kijów"),
    ("kobenhavn", "Kopenhaga"),
    ("koln", "Kolonia"),
    ("kosice", "Koszyce"),
    ("krakow", "Kraków"),
    ("kyiv", "Kijów"),
    ("lausanne", "Lozanna"),
    ("leipzig", "Lipsk"),
    ("lisboa", "Lizbona"),
    ("lisbon", "Lizbona"),
    ("ljubljana", "Lublana"),
    ("lodz", "Łódź"),
    ("london", "Londyn"),
    ("lucerne", "Lucerna"),
    ("luzern", "Lucerna"),
    ("lviv", "Lwów"),
    ("madrid", "Madryt"),
    ("marseille", "Marsylia"),
    ("milan", "Mediolan"),
    ("milano", "Mediolan"),
    ("minsk", "Mińsk"),
    ("munchen", "Monachium"),
    ("munich", "Monachium"),
    ("murcia", "Murcja"),
    ("myslowice", "Mysłowice"),
    ("naples", "Neapol"),
    ("napoli", "Neapol"),
    ("nice", "Nicea"),
    ("nuremberg", "Norymberga"),
    ("nurnberg", "Norymberga"),
    ("odesa", "Odessa"),
    ("olomouc", "Ołomuniec"),
    ("padova", "Padwa"),
    ("padua", "Padwa"),
    ("paris", "Paryż"),
    ("pilsen", "Pilzno"),
    ("plzen", "Pilzno"),
    ("poznan", "Poznań"),
    ("prague", "Praga"),
    ("praha", "Praga"),
    ("reunion-island", "Reunion"),
    ("riga", "Ryga"),
    ("roma", "Rzym"),
    ("rome", "Rzym"),
    ("rzeszow", "Rzeszów"),
    ("sevilla", "Sewilla"),
    ("seville", "Sewilla"),
    ("stockholm", "Sztokholm"),
    ("strasbourg", "Strasburg"),
    ("the-hague", "Haga"),
    ("torino", "Turyn"),
    ("torun", "Toruń"),
    ("toulouse", "Tuluza"),
    ("trieste", "Triest"),
    ("turin", "Turyn"),
    ("valencia", "Walencja"),
    ("venezia", "Wenecja"),
    ("venice", "Wenecja"),
    ("verona", "Werona"),
    ("vienna", "Wiedeń"),
    ("vilnius", "Wilno"),
    ("warsaw", "Warszawa"),
    ("wien", "Wiedeń"),
    ("wroclaw", "Wrocław"),
    ("yerevan", "Erywań"),
    ("zaragoza", "Saragossa"),
    ("zielona-gora", "Zielona Góra"),
    ("zurich", "Zurych"),
];

/// E.g. `🏆2026-02-kraków-hatsumi-taikai-2026`; the city is left out if the
/// tournament name already contains it.
#[must_use]
pub fn tournament_channel_name(entry: &TournamentEntry) -> Option<ChannelName> {
    let mut raw = String::from("🏆");
    if let Some((_, month, year)) = parse_date(&entry.date) {
        let _ = write!(raw, "{year}-{month:02}-");
    }

    let city = polish_city(&entry.place);
    let name = slugify(&entry.name);
    if !contains_words(&name, &slugify(&entry.place)) && !contains_words(&name, &slugify(&city)) {
        let _ = write!(raw, "{city}-");
    }

    raw.push_str(&entry.name);
    ChannelName::new(&raw)
}

/// E.g. `Kraków :: 14-15 lutego 2026 :: https://...`.
#[must_use]
pub fn tournament_topic(entry: &TournamentEntry) -> String {
    [
        polish_city(&entry.place),
        polish_date(&entry.date),
        entry.url.clone(),
    ]
    .into_iter()
    .filter(|part| !part.is_empty())
    .collect::<Vec<_>>()
    .join(" :: ")
}

#[must_use]
pub fn polish_city(place: &str) -> String {
    let place = place.trim();
    let key = slugify(place);
    POLISH_CITY_NAMES
        .iter()
        .find(|(ascii, _)| *ascii == key)
        .map_or_else(|| place.to_owned(), |(_, polish)| (*polish).to_owned())
}

/// `14-15 February 2026` → `14-15 lutego 2026`; unparseable dates are kept.
#[must_use]
pub fn polish_date(date: &str) -> String {
    let Some((days, month, year)) = parse_date(date) else {
        return date.trim().to_owned();
    };
    let polish_month = |month: usize| MONTHS[month - 1].1;

    match days
        .split_once('-')
        .map(|(start, end)| (start.parse::<u8>(), end.parse::<u8>()))
    {
        // Runs into the next month, e.g. `31-1 January 2026`.
        Some((Ok(start), Ok(end))) if end < start => {
            let (next_month, next_year) = if month == 12 {
                (1, year + 1)
            } else {
                (month + 1, year)
            };
            let first = polish_month(month);
            let next = polish_month(next_month);
            if next_year == year {
                format!("{start} {first} - {end} {next} {year}")
            } else {
                format!("{start} {first} {year} - {end} {next} {next_year}")
            }
        }
        _ => format!("{days} {} {year}", polish_month(month)),
    }
}

/// Parses `<days> <English month> <year>` into days, month number and year.
fn parse_date(date: &str) -> Option<(&str, usize, u16)> {
    let mut words = date.split_whitespace().rev();
    let year = words.next()?.parse().ok()?;
    let month = words.next()?;
    let month = MONTHS
        .iter()
        .position(|(english, _)| english.eq_ignore_ascii_case(month))?;
    let days = words.next()?;
    words.next().is_none().then_some((days, month + 1, year))
}

/// Whether the words of slug `needle` appear consecutively in slug `haystack`.
fn contains_words(haystack: &str, needle: &str) -> bool {
    let haystack: Vec<&str> = haystack.split('-').collect();
    let needle: Vec<&str> = needle.split('-').collect();
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(name: &str, date: &str, place: &str, url: &str) -> TournamentEntry {
        TournamentEntry {
            name: name.to_owned(),
            url: url.to_owned(),
            rules: "Riichi".to_owned(),
            date: date.to_owned(),
            place: place.to_owned(),
            approval_status: "OK".to_owned(),
            registration_start: String::new(),
            results_status: String::new(),
        }
    }

    fn channel_name(name: &str, date: &str, place: &str) -> String {
        tournament_channel_name(&entry(name, date, place, ""))
            .unwrap()
            .as_str()
            .to_owned()
    }

    #[test]
    fn follows_the_server_convention() {
        let hatsumi = entry(
            "Hatsumi Taikai 2026",
            "14-15 February 2026",
            "Krakow",
            "https://chombo.club/tournaments/2025/12/15/hatsumi-taikai-2026/",
        );

        assert_eq!(
            tournament_channel_name(&hatsumi).unwrap().as_str(),
            "🏆2026-02-kraków-hatsumi-taikai-2026"
        );
        assert_eq!(
            tournament_topic(&hatsumi),
            "Kraków :: 14-15 lutego 2026 :: \
             https://chombo.club/tournaments/2025/12/15/hatsumi-taikai-2026/"
        );
        assert_eq!(
            tournament_topic(&entry("Cup", "TBA", "Oss", "")),
            "Oss :: TBA"
        );
    }

    #[test]
    fn names_channels() {
        assert_eq!(
            channel_name(
                "Zensekai no Heiwa no Taikai 2026",
                "25-26 March 2026",
                "Cologne"
            ),
            "🏆2026-03-kolonia-zensekai-no-heiwa-no-taikai-2026"
        );
        assert_eq!(
            channel_name("Vienna Riichi Open 2026", "5-6 December 2026", "Vienna"),
            "🏆2026-12-vienna-riichi-open-2026"
        );
        assert_eq!(
            channel_name("Boss Cup", "1 May 2026", "Oss"),
            "🏆2026-05-oss-boss-cup"
        );
        assert_eq!(channel_name("Cup", "sometime", "Warsaw"), "🏆warszawa-cup");
    }

    #[test]
    fn translates_dates() {
        assert_eq!(polish_date("24 January 2026"), "24 stycznia 2026");
        assert_eq!(
            polish_date("31-1 January 2026"),
            "31 stycznia - 1 lutego 2026"
        );
        assert_eq!(
            polish_date("30-2 December 2026"),
            "30 grudnia 2026 - 2 stycznia 2027"
        );
        assert_eq!(polish_date("14-15 Smarch 2026"), "14-15 Smarch 2026");
    }
}
