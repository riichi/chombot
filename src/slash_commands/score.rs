use anyhow::Result;
use log::info;
use num_bigint::BigInt;
use poise::serenity_prelude::{Color, CreateEmbed};
use poise::ChoiceParameter;
use riichi_hand::points::{Fu, Han, Honbas, PointsCalculationMode, PointsCustom};

use crate::PoiseContext;

const DEFAULT_HONBAS: i64 = 0;

type Points = PointsCustom<BigInt>;

#[derive(Debug, ChoiceParameter)]
pub enum Mode {
    Default,
    Loose,
    Unlimited,
}

impl Default for Mode {
    fn default() -> Self {
        Self::Default
    }
}

impl From<Mode> for PointsCalculationMode {
    fn from(value: Mode) -> Self {
        match value {
            Mode::Default => PointsCalculationMode::Default,
            Mode::Loose => PointsCalculationMode::Loose,
            Mode::Unlimited => PointsCalculationMode::Unlimited,
        }
    }
}

/// Calculate the score for given number of han and fu points.
#[poise::command(slash_command)]
pub async fn score(
    ctx: PoiseContext<'_>,
    #[description = "Number of han points"]
    #[min = -1600]
    #[max = 1600]
    han: i64,
    #[description = "Number of fu points"]
    #[min = -100000]
    #[max = 100000]
    fu: i64,
    #[description = "Number of honbas (counter sticks)"]
    #[min = -10000]
    #[max = 10000]
    honbas: Option<i64>,
    #[description = "Calculating mode"] mode: Option<Mode>,
) -> Result<()> {
    let honbas = honbas.unwrap_or(DEFAULT_HONBAS);
    let points_calculation_mode: PointsCalculationMode = mode.unwrap_or_default().into();

    let han = Han::new(i32::try_from(han)?);
    let fu = Fu::new(i32::try_from(fu)?);
    let honbas = Honbas::new(i32::try_from(honbas)?);
    let points = Points::from_calculated(points_calculation_mode, han, fu, honbas)?;
    let fields = create_points_embed_fields(points);

    ctx.send(|reply| reply.embed(move |embed| create_points_embed(embed, han, fu, honbas, fields)))
        .await?;

    Ok(())
}

fn create_points_embed(
    embed: &mut CreateEmbed,
    han: Han,
    fu: Fu,
    honbas: Honbas,
    fields: impl Iterator<Item = (&'static str, String, bool)>,
) -> &mut CreateEmbed {
    embed
        .title(format!("**{han} {fu} {honbas}**"))
        .color(Color::DARK_GREEN)
        .fields(fields)
}

fn create_points_embed_fields(
    points: Points,
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
    match points {
        None => "N/A".to_owned(),
        Some(value) => value.to_string(),
    }
}

fn format_ko_tsumo_points(points: Option<(BigInt, BigInt)>) -> String {
    match points {
        None => "N/A".to_owned(),
        Some((value_ko, value_oya)) => format!("{}/{}", value_ko, value_oya),
    }
}
