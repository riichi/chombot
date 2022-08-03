use serenity::model::application::interaction::application_command::{
    CommandDataOption, CommandDataOptionValue,
};
use serenity::model::guild::PartialMember;
use serenity::model::user::User;

fn get_option<'a>(
    options: &'a [CommandDataOption],
    option_name: &'static str,
) -> Option<&'a CommandDataOption> {
    options.iter().find(|option| option.name == option_name)
}

pub fn get_string_option<'a>(
    options: &'a [CommandDataOption],
    option_name: &'static str,
) -> Option<&'a str> {
    if let Some(option) = get_option(options, option_name) {
        if let Some(CommandDataOptionValue::String(value)) = &option.resolved {
            return Some(value.as_str());
        }
    }
    None
}

pub fn get_user_option<'a>(
    options: &'a [CommandDataOption],
    option_name: &'static str,
) -> Option<(&'a User, &'a Option<PartialMember>)> {
    if let Some(option) = get_option(options, option_name) {
        if let Some(CommandDataOptionValue::User(user, partial_member)) = &option.resolved {
            return Some((user, partial_member));
        }
    }
    None
}
