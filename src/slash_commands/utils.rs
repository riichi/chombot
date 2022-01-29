use serenity::model::user::User;
use serenity::model::guild::PartialMember;
use serenity::model::interactions::application_command::{
    ApplicationCommandInteractionDataOption as DataOption,
    ApplicationCommandInteractionDataOptionValue as DataOptionValue,
};

fn get_option<'a>(options: &'a Vec<DataOption>, option_name: &'static str) -> Option<&'a DataOption> {
    options.iter().find(|option| option.name == option_name)
}

pub fn get_string_option<'a>(options: &'a Vec<DataOption>, option_name: &'static str) -> Option<&'a str> {
    if let Some(option) = get_option(options, option_name) {
        if let Some(DataOptionValue::String(value)) = &option.resolved {
            return Some(value.as_str());
        }
    }
    None
}

pub fn get_user_option<'a>(options: &'a Vec<DataOption>, option_name: &'static str) -> Option<(&'a User, &'a Option<PartialMember>)> {
    if let Some(option) = get_option(options, option_name) {
        if let Some(DataOptionValue::User(user, partial_member)) = &option.resolved {
            return Some((user, partial_member));
        }
    }
    None
}