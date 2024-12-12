use crate::authorization::Authorization;
use crate::cli_error::CliError;
use crate::data_types::{User, UserData};
use crate::http_request_handler::HttpRequestHandler;

use chrono::{DateTime, Utc};
use reqwest::{header::AUTHORIZATION, Client, Error, Response, Url};
use serde_json::from_str;
use std::string::String;

#[derive(Debug)]
pub struct FitbitApiManager {
    authorization: Authorization,
}

impl FitbitApiManager {
    pub fn new(authorization: Authorization) -> Self {
        Self { authorization }
    }

    pub async fn get_user_data(&self) -> Result<UserData, CliError> {
        let access_token = self.authorization.get_access_token().await?;

        let client: Client = Client::new();
        let response = client
            .get("https://api.fitbit.com/1/user/-/profile.json")
            .header(AUTHORIZATION, format!("Bearer {}", access_token))
            .send()
            .await;

        let response = HttpRequestHandler::handle_http_request(response)?;

        let response_body: String = HttpRequestHandler::get_response_body(response).await?;
        let user_data: User = from_str(response_body.as_str())?;
        Ok(user_data.user)
    }

    pub async fn update_body_fat(
        &self,
        body_fat: f32,
        datetime: DateTime<Utc>,
    ) -> Result<Response, CliError> {
        let access_token = self.authorization.get_access_token().await?;

        let params = [
            ("fat", body_fat.to_string()),
            ("date", datetime.format("%Y-%m-%d").to_string()),
            ("time", datetime.format("%H:%M:%S").to_string()),
        ];

        let url: Url =
            Url::parse_with_params("https://api.fitbit.com/1/user/-/body/log/fat.json", &params)?;

        let client: Client = Client::new();
        let response = client
            .post(url)
            .header(AUTHORIZATION, format!("Bearer {}", access_token))
            .send()
            .await;

        HttpRequestHandler::handle_http_request(response)
    }

    pub async fn update_body_weight(
        &self,
        body_weight: f32,
        datetime: DateTime<Utc>,
    ) -> Result<Response, CliError> {
        let access_token = self.authorization.get_access_token().await?;

        let params = [
            ("weight", body_weight.to_string()),
            ("date", datetime.format("%Y-%m-%d").to_string()),
            ("time", datetime.format("%H:%M:%S").to_string()),
        ];
        let url: Url = Url::parse_with_params(
            "https://api.fitbit.com/1/user/-/body/log/weight.json",
            &params,
        )?;

        let client: Client = Client::new();
        let response: Result<Response, Error> = client
            .post(url)
            .header(AUTHORIZATION, format!("Bearer {}", access_token))
            .send()
            .await;

        HttpRequestHandler::handle_http_request(response)
    }
}
