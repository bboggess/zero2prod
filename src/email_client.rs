use std::time::Duration;

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
    ///
    /// `timeout` is the timeout for sending an email address
    pub fn new(
        base_url: Url,
        sender: SubscriberEmail,
        authorization_token: Secret<String>,
        timeout: Duration,
    ) -> Self {
        let http_client = Client::builder().timeout(timeout).build().unwrap();

        Self {
            sender,
            base_url,
            http_client,
            authorization_token,
        }
    }

    /// Sends an email to `recipient`. The subject line will be `subject`.
    ///
    /// Tries to use `html_content` for the body, but will fall back to `text_content`
    /// if the recipient doesn't support HTML in the body.
    ///
    /// Returns an `Err` if there is a failure communicating with the email client,
    /// including timeout.
    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), reqwest::Error> {
        let url = self.base_url.join("email").unwrap();
        let body = SendEmailRequest {
            from: self.sender.as_ref(),
            to: recipient.as_ref(),
            subject,
            html_body: html_content,
            text_body: text_content,
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
            .await?
            .error_for_status()?;

        Ok(())
    }
}

/// The format of a request body required by the Postmark email send API
#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct SendEmailRequest<'a> {
    from: &'a str,
    to: &'a str,
    subject: &'a str,
    html_body: &'a str,
    text_body: &'a str,
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::{domain::SubscriberEmail, email_client::EmailClient};
    use claim::{assert_err, assert_ok};
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{Paragraph, Sentence};
    use fake::{Fake, Faker};
    use secrecy::Secret;
    use url::Url;
    use wiremock::matchers::{any, header, header_exists, method, path};
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

    /// Fake email subject for tests
    fn subject() -> String {
        Sentence(1..2).fake()
    }

    /// Fake email body for tests
    fn content() -> String {
        Paragraph(1..10).fake()
    }

    /// Fake email address for tests
    fn email() -> SubscriberEmail {
        SubscriberEmail::parse(SafeEmail().fake()).unwrap()
    }

    /// Configure an email client listening at `base_url`
    fn email_client(base_url: Url) -> EmailClient {
        EmailClient::new(
            base_url,
            email(),
            Secret::new(Faker.fake()),
            Duration::from_millis(200), // fail fast in tests!
        )
    }

    #[tokio::test]
    async fn send_email_sends_the_expected_request() {
        let mock_server = MockServer::start().await;
        let url = Url::parse(&mock_server.uri()).unwrap();
        let email_client = email_client(url);

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

        let _ = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;

        // Mock::expect above has already handled our assertions
    }

    #[tokio::test]
    async fn send_email_returns_ok_if_server_returns_200() {
        let mock_server = MockServer::start().await;
        let url = Url::parse(&mock_server.uri()).unwrap();
        let email_client = email_client(url);

        // Don't care about actual behavior, just mock a 200 response
        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let outcome = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;

        assert_ok!(outcome);
    }

    #[tokio::test]
    async fn send_email_returns_err_if_server_returns_500() {
        let mock_server = MockServer::start().await;
        let url = Url::parse(&mock_server.uri()).unwrap();
        let email_client = email_client(url);

        // Don't care about actual behavior, just mock a 500 response
        Mock::given(any())
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;

        let outcome = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;

        assert_err!(outcome);
    }

    #[tokio::test]
    async fn send_email_times_out_if_server_takes_too_long() {
        let mock_server = MockServer::start().await;
        let url = Url::parse(&mock_server.uri()).unwrap();
        let email_client = email_client(url);

        let response = ResponseTemplate::new(200).set_delay(Duration::from_secs(180));
        Mock::given(any())
            .respond_with(response)
            .expect(1)
            .mount(&mock_server)
            .await;

        let outcome = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;

        assert_err!(outcome);
    }
}
