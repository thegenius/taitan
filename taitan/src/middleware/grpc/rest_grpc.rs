use axum::body::Body;
use axum::{http::header::CONTENT_TYPE, response::IntoResponse, Router};
use futures::{future::BoxFuture, ready};
use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full};
use hyper::{body::Incoming, Request, Response};
use std::{
    convert::Infallible,
    task::{Context, Poll},
};
use tower::{make::Shared, Service};

/// This service splits all incoming requests either to the rest-service, or to
/// the grpc-service based on the `content-type` header.
///
/// Only if the header `content-type = application/grpc` exists, will the requests be handled
/// by the grpc-service. All other requests go to the rest-service.
#[derive(Debug, Clone)]
pub struct RestGrpcService {
    rest_router: Router,
    rest_ready: bool,
    grpc_router: Router,
    grpc_ready: bool,
}

impl RestGrpcService {
    /// Create a new service, which splits requests between the rest- and grpc-router.
    pub fn new(rest_router: Router, grpc_router: Router) -> Self {
        Self {
            rest_router,
            rest_ready: false,
            grpc_router,
            grpc_ready: false,
        }
    }

    pub fn into_make_service(self) -> Shared<Self> {
        Shared::new(self)
    }
}

type Req = Request<Body>;

impl Service<Req> for RestGrpcService {
    type Response = Response<Body>;
    type Error = Infallible;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        ctx: &mut std::task::Context<'_>,
    ) -> Poll<std::result::Result<(), <Self as Service<Request<Body>>>::Error>> {
        loop {
            match (self.rest_ready, self.grpc_ready) {
                (true, true) => {
                    return Ok(()).into();
                }
                (false, _) => {
                    ready!(<axum::Router as Service<Req>>::poll_ready(
                        &mut self.rest_router,
                        ctx
                    ))?;
                    self.rest_ready = true;
                }
                (_, false) => {
                    ready!(<axum::Router as Service<Req>>::poll_ready(
                        &mut self.grpc_router,
                        ctx
                    ))?;
                    self.grpc_ready = true;
                }
            }
        }
    }

    fn call(&mut self, req: Req) -> <Self as Service<Req>>::Future {
        assert!(
            self.grpc_ready,
            "grpc service not ready. Did you forget to call `poll_ready`?"
        );
        assert!(
            self.rest_ready,
            "rest service not ready. Did you forget to call `poll_ready`?"
        );

        // if we get a grpc request call the grpc service, otherwise call the rest service
        // when calling a service it becomes not-ready so we have drive readiness again
        if is_grpc_request(&req) {
            self.grpc_ready = false;
            let future = self.grpc_router.call(req);
            Box::pin(async move {
                let res = future.await?;
                Ok(res.into_response())
            })
        } else {
            self.rest_ready = false;
            let future = self.rest_router.call(req);
            Box::pin(async move {
                let res = future.await?;
                Ok(res.into_response())
            })
        }
    }
}

// type Req = Request<axum::body::Body>;
// impl Service<Req> for RestGrpcService {
//     type Response = Response<Body>;
//     type Error = Infallible;
//     type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

//     fn poll_ready(
//         &mut self,
//         ctx: &mut std::task::Context<'_>,
//     ) -> Poll<
//         std::result::Result<(), <Self as Service<Body>>::Error>,
//     > {
//         loop {
//             match (self.rest_ready, self.grpc_ready) {
//                 (true, true) => {
//                     return Ok(()).into();
//                 }
//                 (false, _) => {
//                     ready!(<axum::Router as Service<Body>>::poll_ready(&mut self.rest_router, ctx))?;
//                     self.rest_ready = true;
//                 }
//                 (_, false) => {
//                     ready!(<axum::Router as Service<Body>>::poll_ready(&mut self.grpc_router, ctx))?;
//                     self.grpc_ready = true;
//                 }
//             }
//         }
//     }

//     fn call(
//         &mut self,
//         req: Req,
//     ) -> <Self as tower::Service<Body>>::Future {
//         assert!(
//             self.grpc_ready,
//             "grpc service not ready. Did you forget to call `poll_ready`?"
//         );
//         assert!(
//             self.rest_ready,
//             "rest service not ready. Did you forget to call `poll_ready`?"
//         );

//         // if we get a grpc request call the grpc service, otherwise call the rest service
//         // when calling a service it becomes not-ready so we have drive readiness again
//         if is_grpc_request(&req) {
//             self.grpc_ready = false;
//             let future = self.grpc_router.call(req);
//             Box::pin(async move {
//                 let res = future.await?;
//                 Ok(res.into_response())
//             })
//         } else {
//             self.rest_ready = false;
//             let future = self.rest_router.call(req);
//             Box::pin(async move {
//                 let res = future.await?;
//                 Ok(res.into_response())
//             })
//         }
//     }
// }

// impl Service<Request<Body>> for RestGrpcService {
//     type Response = Response<BoxBody>;
//     type Error = Infallible;
//     type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

//     fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
//         // drive readiness for each inner service and record which is ready
//         loop {
//             match (self.rest_ready, self.grpc_ready) {
//                 (true, true) => {
//                     return Ok(()).into();
//                 }
//                 (false, _) => {
//                     ready!(self.rest_router.poll_ready(cx))?;
//                     self.rest_ready = true;
//                 }
//                 (_, false) => {
//                     ready!(self.grpc_router.poll_ready(cx))?;
//                     self.grpc_ready = true;
//                 }
//             }
//         }
//     }

//     fn call(&mut self, req: Request<Body>) -> Self::Future {
//         // require users to call `poll_ready` first, if they don't we're allowed to panic
//         // as per the `tower::Service` contract
//         assert!(
//             self.grpc_ready,
//             "grpc service not ready. Did you forget to call `poll_ready`?"
//         );
//         assert!(
//             self.rest_ready,
//             "rest service not ready. Did you forget to call `poll_ready`?"
//         );

//         // if we get a grpc request call the grpc service, otherwise call the rest service
//         // when calling a service it becomes not-ready so we have drive readiness again
//         if is_grpc_request(&req) {
//             self.grpc_ready = false;
//             let future = self.grpc_router.call(req);
//             Box::pin(async move {
//                 let res = future.await?;
//                 Ok(res.into_response())
//             })
//         } else {
//             self.rest_ready = false;
//             let future = self.rest_router.call(req);
//             Box::pin(async move {
//                 let res = future.await?;
//                 Ok(res.into_response())
//             })
//         }
//     }
// }

fn is_grpc_request<B>(req: &Request<B>) -> bool {
    req.headers()
        .get(CONTENT_TYPE)
        .map(|content_type| content_type.as_bytes())
        .filter(|content_type| content_type.starts_with(b"application/grpc"))
        .is_some()
}
