use crate::{
    Config, attempt_history::ProviderAttemptHistory,
    provider_registry::NotificationProviderRegistry,
};

#[must_use]
pub fn render_diagnostics_page(
    config: &Config,
    provider_attempt_history: &ProviderAttemptHistory,
) -> String {
    let providers = NotificationProviderRegistry::new(config);
    let provider_diagnostics = providers.diagnostics();
    let http_webhook = provider_diagnostics.http_webhook;
    let discord_webhook = provider_diagnostics.discord_webhook;
    let telegram = provider_diagnostics.telegram;
    let template = &config.template;
    let http_webhook_enabled = yes_no_label(http_webhook.enabled);
    let http_webhook_target_configured = yes_no_label(http_webhook.target_url_configured);
    let http_webhook_target_valid = yes_no_label(http_webhook.target_url_valid);
    let http_webhook_shared_secret_configured = yes_no_label(http_webhook.shared_secret_configured);
    let http_webhook_send_path_enabled = yes_no_label(http_webhook.send_path_enabled);
    let discord_webhook_enabled = yes_no_label(discord_webhook.enabled);
    let discord_webhook_url_configured = yes_no_label(discord_webhook.webhook_url_configured);
    let discord_webhook_url_valid = yes_no_label(discord_webhook.webhook_url_valid);
    let discord_webhook_send_path_enabled = yes_no_label(discord_webhook.send_path_enabled);
    let telegram_enabled = yes_no_label(telegram.enabled);
    let telegram_api_base_url_configured = yes_no_label(telegram.api_base_url_configured);
    let telegram_api_base_url_valid = yes_no_label(telegram.api_base_url_valid);
    let telegram_bot_token_configured = yes_no_label(telegram.bot_token_configured);
    let telegram_chat_id_configured = yes_no_label(telegram.chat_id_configured);
    let telegram_send_path_enabled = yes_no_label(telegram.send_path_enabled);
    let provider_send_configured = yes_no_label(providers.send_path_configured());
    let provider_send_path_count = providers.send_path_count();
    let configuration_status = providers.configuration_status();
    let summary_template_configured = yes_no_label(template.summary_template_configured());
    let summary_template_valid = yes_no_label(template.summary_template_valid());
    let provider_attempt_history_count = provider_attempt_history.snapshot().len();
    let provider_attempt_history_capacity = provider_attempt_history.capacity();
    format!(
        r#"<!doctype html>
<html lang="en">
<head><meta charset="utf-8"><title>Nako Notification Bridge</title></head>
<body>
  <h1>Nako Notification Bridge</h1>
  <p>Base URL: {}</p>
  <p>Mode: ack only</p>
  <p>Provider send configured: {provider_send_configured}</p>
  <p>Provider send path count: {provider_send_path_count}</p>
  <p>Configuration status: {}</p>
  <p>Summary template status: {}</p>
  <p>Summary template configured: {summary_template_configured}</p>
  <p>Summary template valid: {summary_template_valid}</p>
  <p>Provider attempt history count: {provider_attempt_history_count}</p>
  <p>Provider attempt history capacity: {provider_attempt_history_capacity}</p>
  <p>HTTP webhook provider status: {}</p>
  <p>HTTP webhook enabled: {http_webhook_enabled}</p>
  <p>HTTP webhook target configured: {http_webhook_target_configured}</p>
  <p>HTTP webhook target valid: {http_webhook_target_valid}</p>
  <p>HTTP webhook shared secret configured: {http_webhook_shared_secret_configured}</p>
  <p>HTTP webhook send path enabled: {http_webhook_send_path_enabled}</p>
  <p>Discord webhook provider status: {}</p>
  <p>Discord webhook enabled: {discord_webhook_enabled}</p>
  <p>Discord webhook URL configured: {discord_webhook_url_configured}</p>
  <p>Discord webhook URL valid: {discord_webhook_url_valid}</p>
  <p>Discord webhook send path enabled: {discord_webhook_send_path_enabled}</p>
  <p>Telegram provider status: {}</p>
  <p>Telegram enabled: {telegram_enabled}</p>
  <p>Telegram API base URL configured: {telegram_api_base_url_configured}</p>
  <p>Telegram API base URL valid: {telegram_api_base_url_valid}</p>
  <p>Telegram bot token configured: {telegram_bot_token_configured}</p>
  <p>Telegram chat id configured: {telegram_chat_id_configured}</p>
  <p>Telegram send path enabled: {telegram_send_path_enabled}</p>
  <p>This page is hosted by the Addon Sidecar and is not trusted Nako Admin UI.</p>
</body>
</html>"#,
        config.base_url,
        configuration_status.as_str(),
        template.status().as_str(),
        http_webhook.status,
        discord_webhook.status,
        telegram.status
    )
}

const fn yes_no_label(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostics_page_renderer_reports_provider_state_without_leaking_values() {
        let config = Config::from_env_lookup(|name| match name {
            "NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_ENABLED" => Some("true".to_owned()),
            "NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_URL" => {
                Some("https://hooks.example/internal/path".to_owned())
            }
            "NAKO_NOTIFICATION_BRIDGE_HTTP_WEBHOOK_SHARED_SECRET" => {
                Some("webhook-secret-should-not-appear".to_owned())
            }
            _ => None,
        });
        let history = ProviderAttemptHistory::new(8);

        let html = render_diagnostics_page(&config, &history);

        assert!(html.contains("HTTP webhook provider status: configured"));
        assert!(html.contains("HTTP webhook enabled: yes"));
        assert!(html.contains("Provider attempt history capacity: 8"));
        assert!(!html.contains("hooks.example"));
        assert!(!html.contains("webhook-secret-should-not-appear"));
    }
}
