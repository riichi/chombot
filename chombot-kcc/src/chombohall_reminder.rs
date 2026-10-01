use chrono::{DateTime, Datelike, Months, NaiveDate, NaiveTime, TimeZone, Utc, Weekday};
use chrono_tz::Europe::Warsaw;
use chrono_tz::Tz;
use log::{error, info};
use poise::serenity_prelude::{ChannelId, Context};
use tokio::time::sleep;

const REMINDER_TIME: NaiveTime = NaiveTime::from_hms_opt(20, 0, 0).unwrap();

/// Posts the chombohall reminder on the Tuesday before the third Wednesday
/// of each month at 20:00 Polish time.
pub fn start_chombohall_reminder(channel_id: ChannelId, message: String, ctx: Context) {
    tokio::spawn(async move {
        loop {
            let next = next_reminder_time(Utc::now().with_timezone(&Warsaw));
            info!("Next chombohall reminder scheduled at {next}");
            let delay = (next.with_timezone(&Utc) - Utc::now())
                .to_std()
                .unwrap_or_default();
            sleep(delay).await;

            if let Err(e) = channel_id.say(&ctx, &message).await {
                error!("Could not send chombohall reminder: {e:?}");
            }
        }
    });
}

/// Returns the first reminder time strictly after `now`.
fn next_reminder_time(now: DateTime<Tz>) -> DateTime<Tz> {
    let mut month_start = now.date_naive().with_day(1).unwrap();
    loop {
        let reminder = reminder_time_in_month(month_start);
        if reminder > now {
            return reminder;
        }
        month_start = month_start + Months::new(1);
    }
}

fn reminder_time_in_month(month_start: NaiveDate) -> DateTime<Tz> {
    let third_wednesday = NaiveDate::from_weekday_of_month_opt(
        month_start.year(),
        month_start.month(),
        Weekday::Wed,
        3,
    )
    .unwrap();
    let tuesday = third_wednesday.pred_opt().unwrap();
    Warsaw
        .from_local_datetime(&tuesday.and_time(REMINDER_TIME))
        .single()
        .expect("20:00 is never ambiguous or skipped in Europe/Warsaw")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn warsaw(y: i32, m: u32, d: u32, h: u32, min: u32) -> DateTime<Tz> {
        Warsaw.with_ymd_and_hms(y, m, d, h, min, 0).unwrap()
    }

    #[test]
    fn next_in_same_month() {
        // October 2026: Wednesdays are 7, 14, 21
        assert_eq!(
            next_reminder_time(warsaw(2026, 10, 1, 12, 0)),
            warsaw(2026, 10, 20, 20, 0)
        );
    }

    #[test]
    fn just_before_reminder() {
        assert_eq!(
            next_reminder_time(warsaw(2026, 10, 20, 19, 59)),
            warsaw(2026, 10, 20, 20, 0)
        );
    }

    #[test]
    fn exactly_at_reminder_moves_to_next_month() {
        // November 2026: Wednesdays are 4, 11, 18
        assert_eq!(
            next_reminder_time(warsaw(2026, 10, 20, 20, 0)),
            warsaw(2026, 11, 17, 20, 0)
        );
    }

    #[test]
    fn month_starting_on_wednesday() {
        // July 2026 starts on Wednesday: Wednesdays are 1, 8, 15
        assert_eq!(
            next_reminder_time(warsaw(2026, 7, 1, 0, 0)),
            warsaw(2026, 7, 14, 20, 0)
        );
    }

    #[test]
    fn month_starting_on_thursday() {
        // April 2027 starts on Thursday: Wednesdays are 7, 14, 21
        assert_eq!(
            next_reminder_time(warsaw(2027, 4, 1, 0, 0)),
            warsaw(2027, 4, 20, 20, 0)
        );
    }

    #[test]
    fn year_rollover() {
        // January 2027: Wednesdays are 6, 13, 20
        assert_eq!(
            next_reminder_time(warsaw(2026, 12, 31, 23, 0)),
            warsaw(2027, 1, 19, 20, 0)
        );
    }

    #[test]
    fn summer_time_offset() {
        // 20:00 CEST is 18:00 UTC
        assert_eq!(
            next_reminder_time(warsaw(2026, 7, 1, 0, 0)).with_timezone(&Utc),
            Utc.with_ymd_and_hms(2026, 7, 14, 18, 0, 0).unwrap()
        );
    }
}
