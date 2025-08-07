//! This is module documentation for api
//!
//! A model for connecting to basic REST API.
//! A description of all available entry points can be
use std::{borrow::Cow, env, str::FromStr};

use reqwest::{
    Method, StatusCode,
    header::{HeaderMap, HeaderName, HeaderValue},
};

use crate::{Bot, Result};

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

/// This struct represents a request to API
#[derive(Debug)]
pub struct Rq<'a> {
    bot: Bot,
    uri: Cow<'a, str>,
    json: serde_json::Value,
    form: Vec<(Cow<'a, str>, Cow<'a, str>)>,
    method: Method,
    headers: HeaderMap,
    params: Vec<(Cow<'a, str>, Cow<'a, str>)>,
}

/// This struct represents a response from API
#[derive(Debug, Default, Clone)]
pub struct Rs {
    pub data: serde_json::Value,
    pub raw: Option<String>,
    pub status: StatusCode,
}

impl<'a> Rq<'a> {
    pub fn new(bot: Bot) -> Self {
        Self {
            bot,
            uri: Cow::from(""),
            json: serde_json::json!({}),
            form: vec![],
            method: Method::GET,
            headers: HeaderMap::new(),
            params: vec![],
        }
    }

    pub fn from_static<S>(str: S) -> Result<Self>
    where
        S: Into<Cow<'a, str>>,
    {
        let bot = Bot::new(str.into())?;
        Ok(Rq::new(bot))
    }

    pub fn uri<S>(self, uri: S) -> Self
    where
        S: Into<Cow<'a, str>>,
    {
        let uri = uri.into();
        Self { uri, ..self }
    }

    pub fn method<S>(self, method: S) -> Result<Self>
    where
        S: Into<Cow<'a, str>>,
    {
        let method = Method::from_str(&method.into())?;
        Ok(Self { method, ..self })
    }

    pub fn add_secret_header<S>(mut self, header: &[u8], value: S) -> Result<Self>
    where
        S: Into<Cow<'a, str>>,
    {
        let mut val = HeaderValue::from_str(&value.into())?;
        val.set_sensitive(true);
        let header = HeaderName::from_bytes(header)?;
        self.headers.insert(header, val);
        Ok(self)
    }

    pub fn add_header<S>(mut self, header: &[u8], value: S) -> Result<Self>
    where
        S: Into<String>,
    {
        let Ok(hdr) = HeaderName::from_bytes(header) else {
            tracing::error!(
                header = ?String::from_utf8_lossy(header),
                "Invalid header name"
            );
            return Ok(self);
        };
        let val = HeaderValue::from_str(&value.into())?;
        self.headers.insert(hdr, val);
        Ok(self)
    }

    pub fn with_json(self) -> Result<Self> {
        self.add_header(b"Accept", "application/json")?
            .add_header(b"Content-Type", "application/json")
    }

    pub fn add_params<I, S>(mut self, params: I) -> Self
    where
        I: IntoIterator<Item = (S, S)>,
        S: Into<Cow<'a, str>>,
    {
        let params: Vec<(Cow<str>, Cow<str>)> = params
            .into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect();
        self.params = params;
        self
    }

    pub fn load_payload(mut self, value: serde_json::Value) -> Result<Self> {
        for (key, value) in value
            .as_object()
            .ok_or(anyhow::anyhow!("Payload value must be valid JSON"))?
        {
            self.json[key] = value.clone();
        }
        Ok(self)
    }

    pub fn add_payload(mut self, key: &str, value: serde_json::Value) -> Self {
        self.json[key] = value;
        self
    }

    pub fn add_form<S>(mut self, key: S, value: S) -> Self
    where
        S: Into<Cow<'a, str>>,
    {
        self.form.push((key.into(), value.into()));
        self
    }

    pub fn apply_if<T, F>(self, val: Option<T>, fun: F) -> Self
    where
        Self: Sized,
        F: FnOnce(Self, T) -> Self,
    {
        if let Some(val) = val {
            fun(self, val)
        } else {
            self
        }
    }

    pub fn url(&self) -> Result<url::Url> {
        Ok(self.bot.url.join(&self.uri)?)
    }

    pub async fn apply(&self) -> Result<Rs> {
        let client = reqwest::Client::builder()
            .user_agent(APP_USER_AGENT)
            .build()?;
        let url = self.url()?;
        let req = match self.method {
            Method::GET => client.get(url),
            Method::POST => client.post(url),
            Method::DELETE => client.delete(url),
            Method::PUT => client.put(url),
            _ => return Err(anyhow::anyhow!("Unsupported HTTP method")),
        };
        let req = if self.form.is_empty() {
            req.headers(self.headers.clone())
                .query(&self.params)
                .json(&self.json)
        } else {
            req.headers(self.headers.clone())
                .query(&self.params)
                .form(&self.form)
        };

        let res = req.send().await?;
        if !res.status().is_success() {
            tracing::error!(?res, code = ?res.status());
            let raw = res.text().await?;
            return Ok(Rs {
                raw: Some(raw),
                ..Default::default()
            });
        }

        let raw = res.text().await?;
        match serde_json::from_str(&raw) {
            Ok(data) => Ok(Rs {
                data,
                ..Default::default()
            }),
            Err(_) => Ok(Rs {
                raw: Some(raw),
                ..Default::default()
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_api_builder() {
        let form = Some(("foo", "bar"));
        let rq = Rq::from_static("https://reqres.in")
            .unwrap()
            .uri("/api/users")
            .method("GET")
            .unwrap()
            .add_secret_header(b"x-api-key", "reqres-free-v1")
            .unwrap()
            .with_json()
            .unwrap()
            .add_params(vec![("page", "2")])
            .load_payload(serde_json::json!({"first_name":"George"}))
            .unwrap()
            .add_payload("baz", serde_json::json!("bar"))
            .apply_if(form, |r, v| r.add_form(v.0, v.1));

        assert_eq!(rq.method, Method::GET);
        assert_eq!(rq.uri, "/api/users");
        assert_eq!(rq.headers.len(), 3);
        assert_eq!(rq.headers.get("x-api-key").unwrap(), "reqres-free-v1");
        assert_eq!(rq.headers.get("Content-Type").unwrap(), "application/json");
        assert_eq!(rq.params.len(), 1);
        assert_eq!(rq.json["first_name"], "George");
        assert_eq!(rq.form[0].1, "bar");

        let rs = rq.apply().await.unwrap();
        assert_eq!(rs.data["data"].as_array().unwrap().len(), 6);
    }
}
