use serde::Serialize;
use serde::de::DeserializeOwned;
use std::sync::Arc;

use super::models::WhuApiResponse;
use super::{token::TokenService, transport::WhuApiTransport};
use crate::{Error, Result, WhuApiOptions};

#[derive(Clone)]
pub struct WhuApiClient {
    transport: Arc<WhuApiTransport>,
    token_service: Arc<TokenService>,
}

impl WhuApiClient {
    pub fn new(options: WhuApiOptions) -> Result<Self> {
        let transport = Arc::new(WhuApiTransport::new(&options)?);
        let token_service = Arc::new(TokenService::new(options, (*transport).clone())?);
        Ok(Self {
            transport,
            token_service,
        })
    }

    pub(crate) async fn get_data<T: DeserializeOwned, Q: Serialize>(
        &self,
        path: &str,
        query: Option<&Q>,
    ) -> Result<T> {
        let token = self.token_service.get_access_token().await?;
        let wrapper: WhuApiResponse<T> = self.transport.get(path, query, &token).await?;
        if wrapper.code != 0 {
            return Err(Error::Api {
                path: path.to_owned(),
                message: wrapper.message,
            });
        }
        wrapper.data.ok_or_else(|| Error::Api {
            path: path.to_owned(),
            message: "empty response data".into(),
        })
    }

    pub(crate) async fn get_data_or_null<T: DeserializeOwned, Q: Serialize>(
        &self,
        path: &str,
        query: Option<&Q>,
    ) -> Result<Option<T>> {
        let token = self.token_service.get_access_token().await?;
        let wrapper: WhuApiResponse<T> = self.transport.get(path, query, &token).await?;
        Ok(wrapper.data)
    }
}
