use anyhow::{anyhow, Result};
use async_trait::async_trait;
use num_bigint::BigInt;
use riichi_hand::points::{Fu, Han, Honbas, PointsCalculationMode, PointsCustom};
use serenity::builder::{CreateApplicationCommand, CreateEmbed};
use serenity::client::Context;
use serenity::model::application::command::CommandOptionType;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::utils::Colour;

use crate::slash_commands::utils::{get_int_option, get_string_option};
use crate::slash_commands::SlashCommand;
use crate::Chombot;

const HAND_COMMAND: &str = "score";
const HAN_OPTION: &str = "han";
const FU_OPTION: &str = "fu";
const HONBAS_OPTION: &str = "honbas";
const MODE_OPTION: &str = "mode";

const DEFAULT_MODE: &str = "default";
const LOOSE_MODE: &str = "loose";
const UNLIMITED_MODE: &str = "unlimited";

const MIN_HAN: i64 = -1600;
const MAX_HAN: i64 = 1600;
const MIN_FU: i64 = -100000;
const MAX_FU: i64 = 100000;
const MIN_HONBAS: i64 = -10000;
const MAX_HONBAS: i64 = 10000;
const DEFAULT_HONBAS: i64 = 0;

type Points = PointsCustom<BigInt>;

pub struct ScoreCommand;

impl ScoreCommand {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl SlashCommand for ScoreCommand {
    fn get_name(&self) -> &'static str {
        HAND_COMMAND
    }

    fn add_application_command(&self, command: &mut CreateApplicationCommand) {
        command
            .description("Calculate the score for given number of han and fu points")
            .create_option(|option| {
                option
                    .name(HAN_OPTION)
                    .description("Number of han points")
                    .kind(CommandOptionType::Integer)
                    .min_int_value(MIN_HAN)
                    .max_int_value(MAX_HAN)
                    .required(true)
            })
            .create_option(|option| {
                option
                    .name(FU_OPTION)
                    .description("Number of fu points")
                    .kind(CommandOptionType::Integer)
                    .min_int_value(MIN_FU)
                    .max_int_value(MAX_FU)
                    .required(true)
            })
            .create_option(|option| {
                option
                    .name(HONBAS_OPTION)
                    .description("Number of honbas (counter sticks)")
                    .kind(CommandOptionType::Integer)
                    .min_int_value(MIN_HONBAS)
                    .max_int_value(MAX_HONBAS)
                    .required(false)
            })
            .create_option(|option| {
                option
                    .name(MODE_OPTION)
                    .description("Calculating mode")
                    .kind(CommandOptionType::String)
                    .add_string_choice("Default", DEFAULT_MODE)
                    .add_string_choice("Loose", LOOSE_MODE)
                    .add_string_choice("Unlimited", UNLIMITED_MODE)
                    .required(false)
            });
    }

    async fn handle(
        &self,
        ctx: &Context,
        command: &ApplicationCommandInteraction,
        _chombot: &Chombot,
    ) -> Result<()> {
        let han = get_int_option(&command.data.options, HAN_OPTION)
            .ok_or(anyhow!("Missing han value"))?;
        let fu =
            get_int_option(&command.data.options, FU_OPTION).ok_or(anyhow!("Missing fu value"))?;
        let honbas = get_int_option(&command.data.options, HONBAS_OPTION).unwrap_or(DEFAULT_HONBAS);
        let mode = get_string_option(&command.data.options, MODE_OPTION).unwrap_or(DEFAULT_MODE);
        let points_calculation_mode = match mode {
            DEFAULT_MODE => Ok(PointsCalculationMode::Default),
            LOOSE_MODE => Ok(PointsCalculationMode::Loose),
            UNLIMITED_MODE => Ok(PointsCalculationMode::Unlimited),
            _ => Err(anyhow!("Invalid mode: {mode}")),
        }?;

        let han = Han::new(i32::try_from(han)?);
        let fu = Fu::new(i32::try_from(fu)?);
        let honbas = Honbas::new(i32::try_from(honbas)?);
        let points = Points::from_calculated(points_calculation_mode, han, fu, honbas)?;
        let embed = create_points_embed(han, fu, honbas, &points);

        command
            .edit_original_interaction_response(&ctx.http, |response| response.add_embed(embed))
            .await?;

        Ok(())
    }
}

fn create_points_embed(han: Han, fu: Fu, honbas: Honbas, points: &Points) -> CreateEmbed {
    let fields = [
        (
            "Non-dealer tsumo",
            format_ko_tsumo_points(&points.ko_tsumo()),
            false,
        ),
        ("Non-dealer ron", format_points(&points.ko_ron()), false),
        ("Dealer tsumo", format_points(&points.oya_tsumo()), false),
        ("Dealer ron", format_points(&points.oya_ron()), false),
    ];

    let mut embed = CreateEmbed::default();
    embed
        .title(format!("**{han} {fu} {honbas}**"))
        .color(Colour::DARK_GREEN)
        .fields(fields);

    embed
}

fn format_points(points: &Option<BigInt>) -> String {
    match points {
        None => "N/A".to_owned(),
        Some(value) => value.to_string(),
    }
}

fn format_ko_tsumo_points(points: &Option<(BigInt, BigInt)>) -> String {
    match points {
        None => "N/A".to_owned(),
        Some((value_ko, value_oya)) => format!("{}/{}", value_ko, value_oya),
    }
}
