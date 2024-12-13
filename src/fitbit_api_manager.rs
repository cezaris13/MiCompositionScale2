use crate::authorization::IAuthorization;
use crate::cli_error::CliError;
use crate::data_types::user::{User, UserData};
use crate::http_request_handler::IHttpRequestHandler;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::{header::AUTHORIZATION, Client, Error, Response, Url};
use serde_json::from_str;
use std::string::String;

pub struct FitbitApiManager<'a> {
    authorization: &'a dyn IAuthorization,
    http_request_handler: &'a dyn IHttpRequestHandler,
    http_client: &'a Client,
}

impl<'a> FitbitApiManager<'a> {
    pub fn new(
        authorization: &'a impl IAuthorization,
        http_request_handler: &'a impl IHttpRequestHandler,
        http_client: &'a Client,
    ) -> Self {
        Self {
            authorization,
            http_request_handler,
            http_client,
        }
    }
}

#[async_trait]
pub trait IFitbitApiManager: Sync {
    async fn get_user_data(&self) -> Result<UserData, CliError>;

    async fn update_body_fat(
        &self,
        body_fat: f32,
        datetime: DateTime<Utc>,
    ) -> Result<Response, CliError>;

    async fn update_body_weight(
        &self,
        body_weight: f32,
        datetime: DateTime<Utc>,
    ) -> Result<Response, CliError>;
}

#[async_trait]
impl<'a> IFitbitApiManager for FitbitApiManager<'a> {
    async fn get_user_data(&self) -> Result<UserData, CliError> {
        let access_token = self.authorization.get_access_token().await?;

        let response = self
            .http_client
            .get("https://api.fitbit.com/1/user/-/profile.json")
            .header(AUTHORIZATION, format!("Bearer {}", access_token))
            .send()
            .await;

        let response = self.http_request_handler.handle_http_request(response)?;

        let response_body: String = self
            .http_request_handler
            .get_response_body(response)
            .await?;
        let user_data: User = from_str(response_body.as_str())?;
        Ok(user_data.user)
    }

    async fn update_body_fat(
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

        let response = self
            .http_client
            .post(url)
            .header(AUTHORIZATION, format!("Bearer {}", access_token))
            .send()
            .await;

        self.http_request_handler.handle_http_request(response)
    }

    async fn update_body_weight(
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

        let response: Result<Response, Error> = self
            .http_client
            .post(url)
            .header(AUTHORIZATION, format!("Bearer {}", access_token))
            .send()
            .await;

        self.http_request_handler.handle_http_request(response)
    }
}
