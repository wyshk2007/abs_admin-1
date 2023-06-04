use crate::domain::vo::RespVO;
use crate::middleware::auth::{check_auth, checked_token, is_white_list_api};
use crate::service::CONTEXT;
use std::{rc::Rc};
use ntex::service::{Middleware, Service};
use ntex::util::BoxFuture;
use ntex::web::{Error, ErrorRenderer, WebRequest, WebResponse};
use ntex::web::error::ErrorUnauthorized;

pub struct Auth;

impl<S> Middleware<S> for Auth{
    type Service = AuthMiddleware<S>;

    fn create(&self, service: S) -> Self::Service {
        AuthMiddleware { service:Rc::new(service) }
    }
}

pub struct AuthMiddleware<S> {
    service: Rc<S>,
}

impl<S, Err> Service<WebRequest<Err>> for AuthMiddleware<S>
    where
        S: Service<WebRequest<Err>, Response = WebResponse, Error = Error>,
        Err: ErrorRenderer,
{
    type Response = WebResponse;
    type Error = Error;
    type Future<'f> = BoxFuture<'f, Result<Self::Response, Self::Error>> where Self: 'f;

    ntex::forward_poll_ready!(service);

    fn call(&self, req: WebRequest<Err>) -> Self::Future<'_> {
        let svc = self.service.clone();
        let token = req
            .headers()
            .get("access_token")
            .map(|v| v.to_str().unwrap_or_default().to_string())
            .unwrap_or_default();
        let path = req.path().to_string();
        Box::pin(async move {
            //debug mode not enable auth
            if !CONTEXT.config.debug {
                if !is_white_list_api(&path) {
                    match checked_token(&token, &path).await {
                        Ok(data) => match check_auth(&data, &path).await {
                            Ok(_) => {}
                            Err(e) => {
                                let resp: RespVO<String> = RespVO::from_error_info("-1", &format!("无权限访问:{}", e.to_string()));
                                return Ok(req.into_response(resp.resp_json()));
                            }
                        },
                        Err(e) => {
                            //401 http code will exit login
                            let resp: RespVO<String> = RespVO::from_error_info("401",&format!("Unauthorized for:{}", e.to_string()));
                            return Err(ErrorUnauthorized(serde_json::json!(&resp).to_string()).into());
                        }
                    }
                }
            }
            let res = svc.call(req).await?;
            Ok(res)
        })
    }
}