use std::collections::HashMap;
use std::ffi::OsString;
use std::fmt::{Display, Formatter};
use std::io;
use std::net::SocketAddr;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex, RwLock};

use reqwest::Error;
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{json, Value};
use tauri::State;
use url::form_urlencoded;
use MicrosoftAuthenticationError::{IOError, ServerError};

use crate::oauth::MicrosoftAuthenticationError::{MalformedOAuthRequest, MsError, NetworkError, XboxLiveResponseError};
use crate::oauth::server::{HttpServerError, start};
use crate::state::{MinecraftAuthentication, MinecraftProfile, OAuthConfig};

mod server;

const OAUTH_PATH: &str = "oauth/v2/microsoft";

pub type Result<T> = std::result::Result<T, MicrosoftAuthenticationError>;

#[derive(Debug)]
pub enum MicrosoftAuthenticationError {
    ServerError(HttpServerError),
    MalformedOAuthRequest(),
    IOError(io::Error),
    NetworkError(Error),
    MsError(MsErrorResponse),
    XboxLiveResponseError(String)
}

impl Serialize for MicrosoftAuthenticationError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}

impl Display for MicrosoftAuthenticationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            ServerError(e) => { e.to_string() }
            MalformedOAuthRequest() => { "Malformed OAuth request".to_string() }
            IOError(e) => { e.to_string() }
            NetworkError(e) => { e.to_string() }
            MsError(e) => { e.error_description.clone() }
            XboxLiveResponseError(e) => { e.clone() }
        };
        write!(f, "{}", str)
    }
}

impl From<Error> for MicrosoftAuthenticationError {
    fn from(value: Error) -> Self {
        NetworkError(value)
    }
}

#[tauri::command]
pub async fn use_no_auth(
    mc_creds: State<'_, Arc<Mutex<Option<MinecraftAuthentication>>>>,
) -> Result<()> {
    // *mc_creds.lock().unwrap() = Some(MinecraftAuthentication {
    //     access_token: "".to_string(),
    //     expires_in: 0,
    //     refresh_token: "".to_string(),
    //     profile: MinecraftProfile {
    //         id: "".to_string(),
    //         name: "".to_string(),
    //     },
    // });

    Ok(())
}

#[tauri::command]
pub async fn microsoft_login(
    oauth_config: State<'_, OAuthConfig<'static>>,
    mc_creds: State<'_, Arc<Mutex<Option<MinecraftAuthentication>>>>,
) -> Result<()> {
    let creds = launch_login(&oauth_config).unwrap().unwrap();

    let ms_token = get_ms_token(
        creds.token,
        oauth_config.deref(),
        format!("http://localhost:6879/{}", OAUTH_PATH)).await?;
    let xbx_live_token = get_xbxl_token(&ms_token).await?;
    let xsts_live_token = get_xsts_token(xbx_live_token).await?;
    let minecraft_token = get_minecraft_access_token(xsts_live_token).await?;
    let minecraft_profile = get_minecraft_profile(&minecraft_token).await.unwrap();

    *mc_creds.lock().unwrap() = Some(MinecraftAuthentication {
        access_token: minecraft_token.access_token,
        expires_in: minecraft_token.expires_in,
        refresh_token: ms_token.refresh_token,
        profile: minecraft_profile,
    });

    Ok(())
}

pub struct MicrosoftCredentials {
    pub token: String,
}

fn launch_login(
    config: &OAuthConfig
) -> Result<Option<MicrosoftCredentials>> {
    let amCreds = Arc::new(Mutex::new(None));

    let response_type = config.response_type.to_string();
    let am_clone = amCreds.clone();
    let server = start(SocketAddr::from(([127, 0, 0, 1], 6879)), move |path, stream| {
        let mut pairs = if let Some(query_start) = path.find('?') {
            let query = &path[query_start + 1..];

            form_urlencoded::parse(query.as_bytes())
        } else {
            return Err("Invalid, no request parameters.".to_string());
        };

        let pair = pairs.find(|it| {
            let x = it.0.deref();
            x == response_type
        }).ok_or("Invalid oauth request.".to_string())?;

        *am_clone.lock().unwrap() = Some(pair.1.to_string());
        Ok("You have been authenticated! You can now return to the launcher.".to_string())
    }).unwrap();

    open::that_detached(
        make_oauth_path(
            &config,
            format!("http://localhost:6879/{}", OAUTH_PATH).as_str(),
        )
    ).map_err(|e| IOError(e))?;

    server.join().unwrap();

    let credentials = amCreds.lock().unwrap().clone().map(|s| MicrosoftCredentials {
        token: s
    });

    Ok(credentials)
}

fn make_oauth_path(
    config: &OAuthConfig,
    redirect_uri: &str,
) -> OsString {
    let str = format!(
        "https://login.microsoftonline.com/{tenant}/oauth2/v2.0/authorize?\
        client_id={client_id}&\
        response_type={response_type}&\
        redirect_uri={redirect_uri}&\
        scope={scope}&\
        prompt=select_account",
        tenant = config.tenant,
        client_id = config.client_id,
        response_type = config.response_type,
        redirect_uri = redirect_uri,
        scope = config.scope
    );

    OsString::from(str)
}

#[derive(Deserialize)]
struct MsTokenResponse {
    access_token: String,
    token_type: String,
    expires_in: u64,
    scope: String,
    refresh_token: String,
    id_token: Option<String>,
}

#[derive(Deserialize, Debug)]
struct MsErrorResponse {
    pub error: String,
    pub error_description: String,
    pub error_codes: Vec<i32>,
    pub timestamp: String,
    pub trace_id: String,
    pub correlation_id: String,
}

async fn get_ms_token<'a>(
    code: String,
    oauth_config: &OAuthConfig<'a>,
    redirect_uri: String,
) -> Result<MsTokenResponse> {
    let url = reqwest::Url::parse(format!(
        "https://login.microsoftonline.com/{tenant}/oauth2/v2.0/token",
        tenant = oauth_config.tenant
    ).as_str()).unwrap();

    println!("{}", url);
    let client = reqwest::Client::new();

    let mut params = HashMap::new();
    params.insert("client_id", oauth_config.client_id);
    params.insert("scope", "xboxlive.signin");
    params.insert("code", code.as_str());
    params.insert("grant_type", "authorization_code");
    params.insert("redirect_uri", redirect_uri.as_str());

    // Serialize the parameters to URL-encoded format
    let body = serde_urlencoded::to_string(&params).unwrap();

    let response = client.post(url)
        .header(ACCEPT, "application/x-www-form-urlencoded")
        .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(body) // Set the URL-encoded form data as the body
        .send().await?;

    return if response.status().is_success() {
        Ok(response.json::<MsTokenResponse>().await?)
    } else {
        Err(MsError(response.json::<MsErrorResponse>().await?))
    };
}

struct XbxlAuthResponse {
    pub token: String,
    pub user_hash: String,
}

fn parse_xl_token_response(value: Value) -> Result<XbxlAuthResponse> {
    let token = value["Token"].as_str().ok_or(XboxLiveResponseError(
        "Failed to find value Token in response".to_string()
    ))?.to_string();

    let user_hash = value["DisplayClaims"].as_object().ok_or(XboxLiveResponseError("Failed to find value 'DisplayClaims' in response".to_string()))?
        ["xui"].as_array().ok_or(XboxLiveResponseError("Failed to find value 'xui' in DisplayClaims in response".to_string()))?
        .get(0).ok_or(XboxLiveResponseError("Failed to find first value in array in xui in DisplayClaims in response".to_string()))?
        .as_object().ok_or(XboxLiveResponseError("Failed to parse type as object in first value in array in xui in DisplayClaims in response".to_string()))?
        ["uhs"].as_str().ok_or(XboxLiveResponseError("Failed to find value 'uhs' in first value in array in xui in DisplayClaims in response".to_string()))?.to_string();

    Ok(XbxlAuthResponse {
        token,
        user_hash,
    })
}

async fn get_xbxl_token(
    ms_token_response: &MsTokenResponse,
) -> Result<XbxlAuthResponse> {
    let client = reqwest::Client::new();

    let body = json!({
        "Properties": {
           "AuthMethod": "RPS",
           "SiteName": "user.auth.xboxlive.com",
           "RpsTicket": format!("d={}", ms_token_response.access_token)
        },
        "RelyingParty": "http://auth.xboxlive.com",
        "TokenType": "JWT"
    });

    let response = client.post("https://user.auth.xboxlive.com/user/authenticate")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&body)
        .send().await?;

    return if response.status().is_success() {
        let response: Value = response.json().await?;

        parse_xl_token_response(response)
    } else {
        Err(XboxLiveResponseError("Server error, received non 200 response code.".to_string()))
    };
}


async fn get_xsts_token(
    xbxl_auth_response: XbxlAuthResponse,
) -> Result<XbxlAuthResponse> {
    let client = reqwest::Client::new();

    let body = json!({
        "Properties": {
            "SandboxId": "RETAIL",
            "UserTokens": [
                xbxl_auth_response.token // from above
            ]
        },
        "RelyingParty": "rp://api.minecraftservices.com/",
        "TokenType": "JWT"
    });

    let response = client.post("https://xsts.auth.xboxlive.com/xsts/authorize")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&body)
        .send().await?;

    return if response.status().is_success() {
        let response: Value = response.json().await?;

        parse_xl_token_response(response)
    } else {
        Err(XboxLiveResponseError("Server error, received non 200 response code.".to_string()))
    };
}

#[derive(serde::Deserialize)]
struct MinecraftAccessToken {
    username: String,
    roles: Vec<String>, // Empty array for roles
    access_token: String,
    token_type: String,
    expires_in: u64,
}

async fn get_minecraft_access_token(
    xsts_token_response: XbxlAuthResponse
) -> Result<MinecraftAccessToken> {
    let client = reqwest::Client::new();

    let body = json!({
        "identityToken": format!("XBL3.0 x={};{}", xsts_token_response.user_hash, xsts_token_response.token)
    });

    let response = client.post("https://api.minecraftservices.com/authentication/login_with_xbox")
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&body)
        .send().await?;

    return if response.status().is_success() {
        let response: MinecraftAccessToken = response.json().await?;

        Ok(response)
    } else {
        Err(XboxLiveResponseError("Server error, received non 200 response code.".to_string()))
    };
}


async fn get_minecraft_profile(
    minecraft_access_token: &MinecraftAccessToken
) -> Result<MinecraftProfile> {
    let client = reqwest::Client::new();

    let response = client.get("https://api.minecraftservices.com/minecraft/profile")
        .header(AUTHORIZATION, format!("Bearer {}", minecraft_access_token.access_token))
        .send().await?;

    return if response.status().is_success() {
        let response: MinecraftProfile = response.json().await?;

        Ok(response)
    } else {
        println!("{}", response.text().await.unwrap());
        Err(XboxLiveResponseError("Server error, received non 200 response code.".to_string()))
    };
}

#[cfg(test)]
mod tests {
    use crate::oauth::{get_minecraft_access_token, get_minecraft_profile, get_ms_token, get_xbxl_token, get_xsts_token, launch_login, make_oauth_path, OAUTH_PATH};
    use crate::state::OAuthConfig;

    #[test]
    fn test_open_browser() {
        open::that_detached(
            "https://google.com"
        ).unwrap();
    }

    #[tokio::test]
    async fn test_server_and_web_open() {
        let config = OAuthConfig {
            client_id: "d64e5a9a-514f-482a-a8b4-967918739d9c",
            response_type: "code",
            scope: "XboxLive.signin%20offline_access",
            tenant: "consumers",
        };

        let creds = launch_login(&config).unwrap().unwrap();

        println!("CODE: {}", creds.token);

        let token = get_ms_token(creds.token, &config, format!("http://localhost:6879/{}", OAUTH_PATH)).await.unwrap();
        println!("ACCESS TOKEN: {}", token.access_token);

        let xbx_live_token = get_xbxl_token(&token).await.unwrap();

        println!("XBXLIVE TOKEN: {}", xbx_live_token.token);

        let xsts_live_token = get_xsts_token(xbx_live_token).await.unwrap();

        println!("XSTS TOKEN: {}", xsts_live_token.token);

        let minecraft_token = get_minecraft_access_token(xsts_live_token).await.unwrap();

        println!("Minecraft TOKEN: {}", minecraft_token.access_token);

        let minecraft_profile = get_minecraft_profile(&minecraft_token).await.unwrap();

        println!("Minecraft PROFILE: {}, {}", minecraft_profile.name, minecraft_profile.id);
    }

    #[test]
    fn create_url() {
        let config = OAuthConfig {
            client_id: "test",
            response_type: "yes",
            scope: "scope",
            tenant: "someone",
        };

        let path = make_oauth_path(
            &config,
            format!("http://localhost:6879/{}", OAUTH_PATH).as_str(),
        );

        println!("{}", path.to_str().unwrap())
    }
}