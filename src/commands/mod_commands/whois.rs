use poise::serenity_prelude as serenity;

use crate::{Context, Error, db::database, types::{ModerationAction, ModerationCase}};

const MAX_CASES_PER_CATEGORY: usize = 3;
const MAX_REASON_LENGTH: usize = 150;
const MAX_MESSAGE_LENGTH: usize = 1900;

#[poise::command(   slash_command,
                    default_member_permissions = "MANAGE_GUILD",
                    required_permissions = "MANAGE_GUILD")]
pub async fn whois(
    ctx: Context<'_>,
    member_id: serenity::UserId,
) -> Result<(), Error> {
    let user_id = i64::try_from(member_id.get())?;
    let cases = match database::get_user_modcases(
        &ctx.data().db,
        user_id,
    )
    .await
    {
        Ok(cases) => cases,
        Err(err) => {
            eprintln!("Failed to fetch moderation history: {err}");

            ctx.say("Failed to fetch this user's moderation history.")
                .await?;

            return Ok(());
        }
    };

    let warnings: Vec<&ModerationCase> = cases
        .iter()
        .filter(|case| case.action == ModerationAction::Warn)
        .collect();
    let kicks: Vec<&ModerationCase> = cases
        .iter()
        .filter(|case| case.action == ModerationAction::Kick)
        .collect();
    let bans: Vec<&ModerationCase> = cases
        .iter()
        .filter(|case| case.action == ModerationAction::Ban)
        .collect();
    let notes: Vec<&ModerationCase> = cases
        .iter()
        .filter(|case| case.action == ModerationAction::Note)
        .collect();
    
    let message = format!(
        "Moderation history for <@{}> (`{}`)\n\n\
         Warnings: {}\n\n{}\n\
         Kicks: {}\n\n{}\n\
         Bans: {}\n\n{}\n\
         Notes: {}\n\n{}\n",
        user_id,
        user_id,
        warnings.len(),
        format_cases(&warnings),
        kicks.len(),
        format_cases(&kicks),
        bans.len(),
        format_cases(&bans),
        notes.len(),
        format_cases(&notes),
    );

    ctx.say(message).await?;

    Ok(())
}

fn format_cases(cases: &[&ModerationCase]) -> String {
    if cases.is_empty() {
        return "- None".to_string();
    }

    let mut output = String::new();

    for case in cases.iter().take(MAX_CASES_PER_CATEGORY) {
        let date = case.created_at.format("%d/%m/%Y");
        let mut reason: String = case
            .reason
            .chars()
            .take(MAX_REASON_LENGTH)
            .collect();
        if case.reason.chars().count() > MAX_REASON_LENGTH {
            reason.push('…');
        }
        // Prevent newlines in a reason from breaking the formatting.
        let reason = reason.replace('\n', " ").replace('\r', "");

        let moderator = match u64::try_from(case.moderator_id) {
            Ok(id) => format!("<@{}>", id),
            Err(_) => "unknown-user".to_string(),
        };

        output.push_str(&format!(
            "- {}: {} - {}\n",
            date,
            reason,
            moderator
        ));
    }

    let remaining = cases.len().saturating_sub(MAX_CASES_PER_CATEGORY);

    if remaining > 0 {
        output.push_str(&format!(
            "- *And {} more...*\n",
            remaining
        ));
    }

    output.trim_end().to_string()
}