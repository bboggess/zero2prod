use reqwest::Client;
use secrecy::{ExposeSecret, Secret};
use serde::Serialize;
use url::Url;

use crate::domain::SubscriberEmail;

/// An email client that can send email to recipients on our behalf.
pub struct EmailClient {
    sender: SubscriberEmail,
    http_client: Client,
    base_url: Url,
    authorization_token: Secret<String>,
}

impl EmailClient {
    /// Creates an email client. Emails will be sent from `sender`.
    ///
    /// `base_url` is a URL where requests can be sent to the client. `authorization_token`
    /// is used to authorize all requests to the client.  
    pub fn new(
        base_url: Url,
        sender: SubscriberEmail,
        authorization_token: Secret<String>,
    ) -> Self {
        Self {
            sender,
            base_url,
            http_client: Client::new(),
            authorization_token,
        }
    }

    /// Sends an email to `recipient`. The subject line will be `subject`.
    ///
    /// Tries to use `html_content` for the body, but will fall back to `text_content`
    /// if the recipient doesn't support HTML in the body.
    ///
    /// Returns an `Err` if there is a failure communicating with the email client.
    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), reqwest::Error> {
        let url = self.base_url.join("email").unwrap();
        let body = SendEmailRequest {
            from: self.sender.as_ref().into(),
            to: recipient.as_ref().into(),
            subject: subject.into(),
            html_body: html_content.into(),
            text_body: text_content.into(),
        };

        let _ = self
            .http_client
            .post(url)
            .header(
                "X-Postmark-Server-Token",
                self.authorization_token.expose_secret(),
            )
            .json(&body)
            .send()
            .await?;

        Ok(())
    }
}

/// The format of a request body required by the Postmark email send API
#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct SendEmailRequest {
    from: String,
    to: String,
    subject: String,
    html_body: String,
    text_body: String,
}

#[cfg(test)]
mod tests {
    use crate::{domain::SubscriberEmail, email_client::EmailClient};
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{Paragraph, Sentence};
    use fake::{Fake, Faker};
    use secrecy::Secret;
    use url::Url;
    use wiremock::matchers::{header, header_exists, method, path};
    use wiremock::{Match, Mock, MockServer, ResponseTemplate};

    /// A wiremock matcher that checks for requests with the required JSON elements
    /// in the body.
    struct EmailBodyMatcher;

    impl Match for EmailBodyMatcher {
        fn matches(&self, request: &wiremock::Request) -> bool {
            let result: Result<serde_json::Value, _> = serde_json::from_slice(&request.body);

            if let Ok(body) = result {
                // The Postmark API example calls out a few required elements in the body,
                // check that those are all present.
                body.get("From").is_some()
                    && body.get("To").is_some()
                    && body.get("Subject").is_some()
                    && body.get("HtmlBody").is_some()
                    && body.get("TextBody").is_some()
            } else {
                false
            }
        }
    }

    #[tokio::test]
    async fn send_email_sends_the_expected_request() {
        let mock_server = MockServer::start().await;
        let url = Url::parse(&mock_server.uri()).unwrap();
        let sender = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let email_client = EmailClient::new(url, sender, Secret::new(Faker.fake()));

        // This asserts that our server will receive exactly one request
        Mock::given(header_exists("X-Postmark-Server-Token"))
            .and(header("Content-Type", "application/json"))
            .and(path("/email"))
            .and(method("POST"))
            .and(EmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Fake all of the auxiliary data needed to send an actual email
        let subscriber_email = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..10).fake();

        // Finally time to act
        let _ = email_client
            .send_email(subscriber_email, &subject, &content, &content)
            .await;

        // Mock::expect above has already handled our assertions
    }
}
