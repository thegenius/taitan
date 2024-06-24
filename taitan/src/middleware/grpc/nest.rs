use std::convert::Infallible;

use axum::{response::IntoResponse, routing::any_service, Router};
use bytes::Bytes;
use futures::{Future, FutureExt};
use http_body_util::{combinators::BoxBody, BodyExt, Empty, Full};

use hyper::{body::Incoming, Request};
use tonic::server::NamedService;
use tower::Service;
type HttpBoxBody = BoxBody<Bytes, hyper::Error>;

type Req = Request<axum::body::Body>;

/// This trait automatically nests the NamedService at the correct path.
pub trait NestTonic: Sized {
    fn nest_tonic<S, P: AsRef<str>>(self, path: P, svc: S) -> Self
    where
        S: Service<Req, Error = Infallible> + Clone + Send + 'static,
        S::Response: IntoResponse + 'static,
        S::Future: Send + 'static;
}

impl NestTonic for Router {
    fn nest_tonic<S, P: AsRef<str>>(self, path: P, svc: S) -> Self
    where
        S: Service<Req, Error = Infallible> + Clone + Send + 'static,
        S::Response: IntoResponse + 'static,
        S::Future: Send + 'static,
    {
        // Nest it at /S::NAME, and wrap the service in an AxumTonicService
        self.route(path.as_ref(), any_service(AxumTonicService { svc }))
    }
}

//------------------------------------------------------------------------------------------------
//  Service
//------------------------------------------------------------------------------------------------

/// The service that converts a tonic service into an axum-compatible one.
#[derive(Clone, Debug)]
struct AxumTonicService<S> {
    svc: S,
}
impl<S> AxumTonicService<S> {
    pub fn new(svc: S) -> Self {
        Self { svc: svc }
    }
}

impl<S> Service<Req> for AxumTonicService<S>
where
    S: Service<Req> + Clone,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.svc.poll_ready(cx)
    }

    fn call(&mut self, req: Req) -> Self::Future {
        println!("processing request: {} {}", req.method(), req.uri().path());
        self.svc.call(req)
    }
}

// impl<B, S> Service<Request<B>> for AxumTonicService<S>
// where
//     S: Service<Request<B>, Error = Infallible, Response = hyper::Response<tonic::body::BoxBody>>,
//     S::Future: Unpin,
// {
//     type Response = axum::response::Response;
//     type Error = Infallible;
//     type Future = AxumTonicServiceFut<S::Future>;

//     fn poll_ready(
//         &mut self,
//         cx: &mut std::task::Context<'_>,
//     ) -> std::task::Poll<Result<(), Self::Error>> {
//         self.svc.poll_ready(cx)
//     }

//     fn call(&self, req: Request<B>) -> Self::Future {
//         AxumTonicServiceFut {
//             fut: self.svc.call(req),
//         }
//     }
// }

// //------------------------------------------------------------------------------------------------
// //  Future
// //------------------------------------------------------------------------------------------------

// /// The future that is returned by the AxumTonicService
// struct AxumTonicServiceFut<F> {
//     fut: F,
// }

// impl<F> Future for AxumTonicServiceFut<F>
// where
//     F: Future<Output = Result<hyper::Response<tonic::body::BoxBody>, Infallible>> + Unpin,
// {
//     type Output = Result<axum::response::Response, Infallible>;

//     fn poll(
//         mut self: std::pin::Pin<&mut Self>,
//         cx: &mut std::task::Context<'_>,
//     ) -> std::task::Poll<Self::Output> {
//         // we only have to map this, whenever an actual response is returned
//         self.fut
//             .poll_unpin(cx)
//             .map_ok(|response| response.into_response())
//     }
// }
