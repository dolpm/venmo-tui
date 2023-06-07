use std::{fmt, sync::Arc};

use regex::Regex;

use reqwest::{cookie::Jar, Url};
use serde::Serialize;

use crate::types::{Identity, LoginResponse, StoriesResponse};

pub struct Api {
    client: reqwest::Client,
    cookie_jar: Arc<Jar>,
    pub identity: Option<Identity>,
    csrf: String,
    bearer: String,
    db: sled::Db,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LoginQuery<'a> {
    username: &'a str,
    password: &'a str,
    is_group: bool,
}

#[derive(Serialize)]
struct __Variables {}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileQuery<'a> {
    operation_name: &'a str,
    variables: __Variables,
    query: &'a str,
}

#[derive(Debug)]
pub enum ApiError {
    LoginFailure(String),
    Unauthaurized(String),
    LogoutFailure(String),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::LoginFailure(e) => write!(f, "login failed! please retry... {e}"),
            ApiError::Unauthaurized(e) => write!(f, "unauthorized! {e}"),
            ApiError::LogoutFailure(e) => write!(f, "logout failed! please retry... {e}"),
        }
    }
}

impl Api {
    fn set_cookies(&mut self, resp: &reqwest::Response) {
        resp.headers().iter().for_each(|v| {
            if v.0.as_str() == "set-cookie" {
                let cookie_name =
                    v.1.to_str()
                        .expect("failed to convert to string")
                        .split("=")
                        .collect::<Vec<_>>()[0];

                if cookie_name == "api_access_token" {
                    let regex =
                        Regex::new("api_access_token=([^;]*)").expect("failed to create regex");
                    let as_str = v.1.to_str().expect("failed to convert to string");
                    let m = regex.find(as_str).unwrap();
                    self.bearer =
                        as_str[m.start()..m.end()].split("=").collect::<Vec<_>>()[1].to_string();
                }

                self.db
                    .insert(cookie_name, v.1.to_str().expect("failed to convert"))
                    .expect("failed to add value to db");
            }
        });
    }

    async fn load_cookies(&mut self) {
        let url = "https://venmo.com".parse::<Url>().unwrap();

        self.db.iter().enumerate().for_each(|(_, v)| match v {
            Err(_) => {}
            Ok((v1, v2)) => {
                let k = String::from_utf8(v1.to_vec()).expect("failed");
                let v = String::from_utf8(v2.to_vec()).expect("failed");

                if k == "api_access_token" {
                    let regex =
                        Regex::new("api_access_token=([^;]*)").expect("failed to create regex");
                    let m = regex.find(&v).unwrap();
                    self.bearer =
                        v[m.start()..m.end()].split("=").collect::<Vec<_>>()[1].to_string();
                }
                self.cookie_jar.add_cookie_str(&v, &url);
            }
        });

        self.fetch_csrf().await.expect("failed to load csrf");
    }

    pub async fn logged_in(&mut self) -> bool {
        match self.client.get("https://account.venmo.com/").send().await {
            Err(_) => false,
            Ok(resp) => resp.url().as_str() != "https://venmo.com/account/sign-in?next=%2F",
        }
    }

    async fn fetch_csrf(&mut self) -> Result<(), ApiError> {
        // logged in
        let url = if self.logged_in().await {
            "https://account.venmo.com/"
        } else {
            "https://venmo.com/account/sign-in"
        };

        self.csrf = match self
            .client
            .get(url)
            .header("accept", "*/*")
            .header("accept-language", "en-US,en;q=0.5")
            .send()
            .await
        {
            Err(e) => {
                // todo: more verbose error for this case
                return Err(ApiError::LoginFailure(e.to_string()));
            }
            Ok(v) => {
                self.set_cookies(&v);
                let text = v.text().await.expect("failed to load text");
                let csrf_regex =
                    Regex::new(r#""csrfToken":"([^"]*)""#).expect("failed to create regex");
                let m = csrf_regex.find(&text).unwrap();
                text[m.start()..m.end()]
                    .split(":")
                    .map(|x| x.replace(r#"""#, ""))
                    .collect::<Vec<_>>()[1]
                    .to_string()
            }
        };

        Ok(())
    }

    pub async fn new() -> Result<Self, ApiError> {
        let jar = Arc::new(Jar::default());

        let client = reqwest::ClientBuilder::new()
            .cookie_provider(jar.clone())
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:109.0) Gecko/20100101 Firefox/113.0")
            .build()
            .expect("failed to build client");

        let mut api = Api {
            db: sled::open("db").expect("failed to initialize DB"),
            cookie_jar: jar,
            csrf: "".to_string(),
            bearer: "".to_string(),
            client,
            identity: None,
        };

        api.load_cookies().await;
        api.fetch_csrf().await?;

        Ok(api)
    }

    pub async fn login(
        &mut self,
        username: &str,
        password: &str,
    ) -> Result<LoginResponse, ApiError> {
        let resp = match self
            .client
            .post("https://venmo.com/api/login")
            .header("content-type", "application/json")
            .header("csrf-token", &self.csrf)
            .header("xsrf-token", &self.csrf)
            .json(&LoginQuery {
                username,
                password,
                is_group: false,
            })
            .send()
            .await
        {
            Err(e) => {
                return Err(ApiError::LoginFailure(e.to_string()));
            }
            Ok(resp) => {
                self.set_cookies(&resp);
                match resp.json::<LoginResponse>().await {
                    Ok(v) => v,
                    Err(e) => return Err(ApiError::LoginFailure(e.to_string())),
                }
            }
        };

        Ok(resp)
    }

    pub async fn get_profile(&mut self) -> Result<Identity, ApiError> {
        let profile_query = r#"
        query Identity($input: ProfileInput) {
          profile(input: $input) {
            ... on Profile {
              availableIdentities {
                ... on BusinessIdentity {
                  isDenylisted
                  isSuspended
                  type
                  avatar {
                    url
                    __typename
                  }
                  displayName
                  handle
                  id
                  profileBackgroundPicture
                  balance {
                    userBalance {
                      value
                      __typename
                    }
                    __typename
                  }
                  __typename
                }
                ... on Identity {
                  isDenylisted
                  isSuspended
                  type
                  avatar {
                    url
                    __typename
                  }
                  displayName
                  handle
                  id
                  balance {
                    userBalance {
                      value
                      __typename
                    }
                    __typename
                  }
                  __typename
                }
                __typename
              }
              __typename
            }
            __typename
          }
        }
        "#;

        let id = match self
            .client
            .post("https://api.venmo.com/graphql")
            .header("Host", "api.venmo.com")
            .header("accept", "*/*")
            .header("content-type", "application/json")
            .bearer_auth(&self.bearer)
            .json(&ProfileQuery {
                operation_name: "Identity",
                variables: __Variables {},
                query: profile_query,
            })
            .send()
            .await
        {
            Err(e) => {
                return Err(ApiError::LoginFailure(e.to_string()));
            }
            Ok(resp) => serde_json::from_value::<Identity>(
                resp.json::<serde_json::Value>()
                    .await
                    .expect("failed to parse")["data"]["profile"]["availableIdentities"][0]
                    .clone(),
            )
            .expect("failed to parse"),
        };

        self.identity = Some(id.clone());

        Ok(id)
    }

    pub async fn get_recents(
        &mut self,
        items_to_load: u32,
        prev: Option<&str>,
    ) -> Result<StoriesResponse, ApiError> {
        if self.identity.is_none() {
            return Err(ApiError::Unauthaurized("Identity not found.".to_string()));
        }

        let mut response: Option<StoriesResponse> = None;

        while response.is_none()
            || response.as_ref().unwrap().stories.len() < items_to_load as usize
        {
            let mut resp = match self
                .client
                .get("https://account.venmo.com/api/stories")
                .header("accept", "*/*")
                .bearer_auth(&self.bearer)
                .query(&[
                    ("feedType", "me"),
                    ("externalId", &self.identity.as_ref().unwrap().id),
                    (
                        "nextId",
                        if response.is_none() {
                            match prev.as_ref() {
                                Some(p) => p,
                                None => "",
                            }
                        } else {
                            &response.as_ref().unwrap().next_id
                        },
                    ),
                ])
                .send()
                .await
            {
                Err(e) => {
                    return Err(ApiError::LoginFailure(e.to_string()));
                }
                Ok(resp) => resp
                    .json::<StoriesResponse>()
                    .await
                    .expect("failed to parse")
                    .clone(),
            };

            match response.as_mut() {
                Some(old_resp) => {
                    old_resp.next_id = resp.next_id;
                    old_resp.stories.append(&mut resp.stories);
                }
                None => response = Some(resp),
            }
        }

        Ok(response.unwrap())
    }

    pub async fn logout(&mut self) -> Result<(), ApiError> {
        if let Err(_) = self
            .client
            .get("https://account.venmo.com/account/logout")
            .send()
            .await
        {
            return Err(ApiError::LogoutFailure(
                "logout request failed! please try again".to_string(),
            ));
        }
        self.db.clear().expect("failed to clear db");
        Ok(())
    }
}