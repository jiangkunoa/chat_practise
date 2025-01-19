use std::future::{ready, Ready};

use actix_web::{
    body::EitherBody, dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform}, Error, HttpMessage, HttpResponse
};
use futures::{FutureExt, TryFutureExt};
use futures_util::future::LocalBoxFuture;
use actix_web::FromRequest;
use actix_web::HttpRequest;
use actix_web::dev::Payload;
use std::ops::Deref;
use log::info;

use crate::web::jwt::{validate_jwt, Claims};

pub struct AuthMiddleware {
    pub whitelist: Vec<String>,
}


impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Transform = AuthMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService {
             service: service,
             whitelist: self.whitelist.clone(),
            }))
    }
}

pub struct AuthMiddlewareService<S> {
    service: S,
    whitelist: Vec<String>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let path = req.path().to_string();
        if self.whitelist.contains(&path) || path.starts_with("/app/") || path.ends_with("favicon.ico") {
            return self.service
            .call(req)
            .map_ok(ServiceResponse::map_into_left_body)
            .boxed_local()
        }

        if let Some(user) = req.headers().get("Authorization") {
            let token = user.to_str().unwrap_or("");
            if let Ok(claims) = validate_jwt(token) {
                req.extensions_mut().insert(claims);
                return self.service
                .call(req)
                .map_ok(ServiceResponse::map_into_left_body)
                .boxed_local()
            }
        }
        // 如果用户未登录，返回 401 Unauthorized
        Box::pin(async {
            // Ok(req.into_response(HttpResponse::Unauthorized().finish()))
            info!("未登录的请求");
            Ok(req.into_response(
                HttpResponse::TooManyRequests()
                    .finish()
                    .map_into_right_body(),
            ))
        })
    }
}





pub struct ClaimsExtractor(pub Claims);

impl FromRequest for ClaimsExtractor {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        // 从请求扩展中提取 Claims
        if let Some(claims) = req.extensions().get::<Claims>() {
            ready(Ok(ClaimsExtractor(claims.clone())))
        } else {
            ready(Err(actix_web::error::ErrorUnauthorized("Unauthorized")))
        }
    }
}

impl Deref for ClaimsExtractor {
    type Target = Claims;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}