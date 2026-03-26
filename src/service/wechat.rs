use crate::config::AppConfig;
use crate::error::{AppError, AppResult};
use crate::model::request::{
    WechatAccessTokenResponse, WechatCode2SessionResponse, WechatPhoneNumberResponse,
};

const CODE2SESSION_URL: &str = "https://api.weixin.qq.com/sns/jscode2session";
const GET_ACCESS_TOKEN_URL: &str = "https://api.weixin.qq.com/cgi-bin/token";
const GET_PHONE_NUMBER_URL: &str = "https://api.weixin.qq.com/wxa/business/getuserphonenumber";

pub struct WechatService {
    client: reqwest::Client,
}

impl WechatService {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    /// 通过code获取openid和session_key
    pub async fn code2session(&self, code: &str) -> AppResult<WechatCode2SessionResponse> {
        let config = AppConfig::get();

        let url = format!(
            "{}?appid={}&secret={}&js_code={}&grant_type=authorization_code",
            CODE2SESSION_URL, config.wechat.appid, config.wechat.secret, code
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await?
            .json::<WechatCode2SessionResponse>()
            .await?;

        if let Some(errcode) = response.errcode {
            if errcode != 0 {
                return Err(AppError::WechatApi(
                    response.errmsg.unwrap_or_else(|| "未知错误".to_string()),
                ));
            }
        }

        if response.openid.is_none() {
            return Err(AppError::WechatApi("获取openid失败".to_string()));
        }

        Ok(response)
    }

    /// 获取access_token (用于调用微信服务端API)
    pub async fn get_access_token(&self) -> AppResult<String> {
        let config = AppConfig::get();

        let url = format!(
            "{}?grant_type=client_credential&appid={}&secret={}",
            GET_ACCESS_TOKEN_URL, config.wechat.appid, config.wechat.secret
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await?
            .json::<WechatAccessTokenResponse>()
            .await?;

        if let Some(errcode) = response.errcode {
            if errcode != 0 {
                return Err(AppError::WechatApi(
                    response.errmsg.unwrap_or_else(|| "获取access_token失败".to_string()),
                ));
            }
        }

        response
            .access_token
            .ok_or_else(|| AppError::WechatApi("access_token为空".to_string()))
    }

    /// 通过手机号code获取用户手机号
    pub async fn get_phone_number(&self, code: &str) -> AppResult<String> {
        let access_token = self.get_access_token().await?;

        let url = format!("{}?access_token={}", GET_PHONE_NUMBER_URL, access_token);

        let response = self
            .client
            .post(&url)
            .json(&serde_json::json!({ "code": code }))
            .send()
            .await?
            .json::<WechatPhoneNumberResponse>()
            .await?;

        if let Some(errcode) = response.errcode {
            if errcode != 0 {
                return Err(AppError::WechatApi(
                    response.errmsg.unwrap_or_else(|| "获取手机号失败".to_string()),
                ));
            }
        }

        response
            .phone_info
            .and_then(|info| info.pure_phone_number)
            .ok_or_else(|| AppError::PhoneGetFailed)
    }
}

impl Default for WechatService {
    fn default() -> Self {
        Self::new()
    }
}
