use reqwest::{Client, Url};
use serde::Serialize;

use crate::{Result, WhuApiOptions};

#[derive(Clone)]
pub(crate) struct WhuApiTransport {
    base_url: String,
    client: Client,
}

impl WhuApiTransport {
    pub(crate) fn new(options: &WhuApiOptions) -> Result<Self> {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .build()?;
        Ok(Self {
            base_url: options.base_url.trim_end_matches('/').to_owned(),
            client,
        })
    }

    pub(crate) fn url(&self, path: &str) -> Result<Url> {
        Ok(Url::parse(&format!("{}{}", self.base_url, path))?)
    }

    pub(crate) async fn get_raw<T: serde::de::DeserializeOwned>(&self, url: Url) -> Result<T> {
        Ok(self
            .client
            .get(url)
            .header("Accept", "application/json, text/plain, */*")
            .header("Accept-Language", "zh-CN,zh;q=0.9")
            .header(
                "Referer",
                "http://zwhqbsd.whu.edu.cn/MobilePayWeb/?t=20260123",
            )
            .header("Origin", "http://zwhqbsd.whu.edu.cn")
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    pub(crate) async fn get<T: serde::de::DeserializeOwned, Q: Serialize>(
        &self,
        path: &str,
        query: Option<&Q>,
        token: &str,
    ) -> Result<T> {
        let mut request = self
            .client
            .get(self.url(path)?)
            .bearer_auth(token)
            .header("Accept", "application/json, text/plain, */*")
            .header("Accept-Language", "zh-CN,zh;q=0.9")
            .header(
                "Referer",
                "http://zwhqbsd.whu.edu.cn/MobilePayWeb/?t=20260123",
            )
            .header("Origin", "http://zwhqbsd.whu.edu.cn");
        if let Some(query) = query {
            request = request.query(query);
        }
        Ok(request.send().await?.error_for_status()?.json().await?)
    }
}
