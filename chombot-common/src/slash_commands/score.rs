use anyhow::Result;
use log::info;
use num_bigint::BigInt;
use poise::serenity_prelude::{Color, CreateEmbed};
use poise::{ChoiceParameter, CreateReply};
use riichi_hand::points::{Fu, Han, Honbas, PointsCalculationMode, PointsCustom};

use crate::{ChombotPoiseContext, ChombotPoiseUserData};

type Points = PointsCustom<BigInt>;

#[derive(Debug, ChoiceParameter, Default)]
pub enum Mode {
    #[default]
    Default,
    Loose,
    Unlimited,
}

impl From<Mode> for PointsCalculationMode {
    fn from(value: Mode) -> Self {
        match value {
            Mode::Default => Self::Default,
            Mode::Loose => Self::Loose,
            Mode::Unlimited => Self::Unlimited,
        }
    }
}

/// Calculate the score for given number of han and fu points.
#[poise::command(slash_command)]
pub async fn score<T: ChombotPoiseUserData>(
    ctx: ChombotPoiseContext<'_, T>,
    #[description = "Number of han points"]
    #[min = -1600]
    #[max = 1600]
    han: i32,
    #[description = "Number of fu points"]
    #[min = -100000]
    #[max = 100000]
    fu: i32,
    #[description = "Number of honbas (counter sticks)"]
    #[min = -10000]
    #[max = 10000]
    honbas: Option<i32>,
    #[description = "Calculating mode"] mode: Option<Mode>,
) -> Result<()> {
    let points_calculation_mode: PointsCalculationMode = mode.unwrap_or_default().into();

    let han = Han::new(han);
    let fu = Fu::new(fu);
    let honbas = honbas.map(Honbas::new).unwrap_or_default();
    let points = Points::from_calculated(points_calculation_mode, han, fu, honbas)?;
    let fields = create_points_embed_fields(&points);

    ctx.send(CreateReply::default().embed(create_points_embed(han, fu, honbas, fields)))
        .await?;

    Ok(())
}

fn create_points_embed(
    han: Han,
    fu: Fu,
    honbas: Honbas,
    fields: impl Iterator<Item = (&'static str, String, bool)>,
) -> CreateEmbed {
    CreateEmbed::new()
        .title(format!("**{han} {fu} {honbas}**"))
        .color(Color::DARK_GREEN)
        .fields(fields)
}

fn create_points_embed_fields(
    points: &Points,
) -> impl Iterator<Item = (&'static str, String, bool)> {
    info!("{points:?}");
    [
        (
            "Non-dealer tsumo",
            format_ko_tsumo_points(points.ko_tsumo()),
            false,
        ),
        ("Non-dealer ron", format_points(points.ko_ron()), false),
        ("Dealer tsumo", format_points(points.oya_tsumo()), false),
        ("Dealer ron", format_points(points.oya_ron()), false),
    ]
    .into_iter()
}

fn format_points(points: Option<BigInt>) -> String {
    points.map_or_else(|| "N/A".to_owned(), |value| value.to_string())
}

fn format_ko_tsumo_points(points: Option<(BigInt, BigInt)>) -> String {
    match points {
        None => "N/A".to_owned(),
        Some((value_ko, value_oya)) => format!("{value_ko}/{value_oya}"),
    }
}

#[cfg(test)]
mod tests {
    use riichi_hand::points::Honbas;

    use super::*;

    macro_rules! test_create_points_embed_fields_impl {
        {$id:ident, $points:expr, $ko_tsumo:expr, $ko_ron:expr, $oya_tsumo:expr, $oya_ron:expr} => {
            #[test]
            fn $id() {
                let points: Points = $points;
                let fields: Vec<_> = create_points_embed_fields(&points).collect();
                assert_eq!(fields, vec![
                    ("Non-dealer tsumo", String::from($ko_tsumo), false),
                    ("Non-dealer ron", String::from($ko_ron), false),
                    ("Dealer tsumo", String::from($oya_tsumo), false),
                    ("Dealer ron", String::from($oya_ron), false),
                ])
            }
        };
    }

    test_create_points_embed_fields_impl! {
        test_create_points_embed_fields_mangan,
        Points::mangan(Honbas::default()),
        "2000/4000",
        "8000",
        "4000",
        "12000"
    }
    test_create_points_embed_fields_impl! {
        test_create_points_embed_fields_no_ron,
        Points::from_calculated(PointsCalculationMode::Default, Han::new(3), Fu::new(20), Honbas::default()).unwrap(),
        "700/1300",
        "N/A",
        "1300",
        "N/A"
    }
    test_create_points_embed_fields_impl! {
        test_create_points_embed_fields_no_tsumo,
        Points::from_calculated(PointsCalculationMode::Default, Han::new(2), Fu::new(25), Honbas::default()).unwrap(),
        "N/A",
        "1600",
        "N/A",
        "2400"
    }
}
