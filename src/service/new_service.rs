use std::error::Error as StdError;
use std::marker::PhantomData;

use futures::{Future, IntoFuture};

use body::Payload;
use super::Service;

/// dox
pub fn make_service_fn<F, Ctx, R>(f: F) -> MakeServiceFn<F, R>
where
    F: for<'a> Fn(&'a Ctx) -> R,
    R: IntoFuture,
{
    MakeServiceFn {
        f,
        _req: PhantomData,
    }
}

/// An asynchronous constructor of `Service`s.
pub trait NewService {
    /// The `Payload` body of the `http::Request`.
    type ReqBody: Payload;

    /// The `Payload` body of the `http::Response`.
    type ResBody: Payload;

    /// The error type that can be returned by `Service`s.
    type Error: Into<Box<StdError + Send + Sync>>;

    /// The resolved `Service` from `new_service()`.
    type Service: Service<
        ReqBody=Self::ReqBody,
        ResBody=Self::ResBody,
        Error=Self::Error,
    >;

    /// The future returned from `new_service` of a `Service`.
    type Future: Future<Item=Self::Service, Error=Self::InitError>;

    /// The error type that can be returned when creating a new `Service`.
    type InitError: Into<Box<StdError + Send + Sync>>;

    /// Create a new `Service`.
    fn new_service(&self) -> Self::Future;
}

/// An asynchronous constructor of `Service`s.
pub trait MakeService<Ctx> {
    /// The `Payload` body of the `http::Request`.
    type ReqBody: Payload;

    /// The `Payload` body of the `http::Response`.
    type ResBody: Payload;

    /// The error type that can be returned by `Service`s.
    type Error: Into<Box<StdError + Send + Sync>>;

    /// The resolved `Service` from `new_service()`.
    type Service: Service<
        ReqBody=Self::ReqBody,
        ResBody=Self::ResBody,
        Error=Self::Error,
    >;

    /// The future returned from `new_service` of a `Service`.
    type Future: Future<Item=Self::Service, Error=Self::MakeError>;

    /// The error type that can be returned when creating a new `Service`.
    type MakeError: Into<Box<StdError + Send + Sync>>;

    /// Create a new `Service`.
    fn make_service(&self, ctx: Ctx) -> Self::Future;
}

impl<F, R, S> NewService for F
where
    F: Fn() -> R,
    R: IntoFuture<Item=S>,
    R::Error: Into<Box<StdError + Send + Sync>>,
    S: Service,
{
    type ReqBody = S::ReqBody;
    type ResBody = S::ResBody;
    type Error = S::Error;
    type Service = S;
    type Future = R::Future;
    type InitError = R::Error;


    fn new_service(&self) -> Self::Future {
        (*self)().into_future()
    }
}

impl<N, Ctx> MakeService<Ctx> for N
where
    N: NewService
{
    type ReqBody = N::ReqBody;
    type ResBody = N::ResBody;
    type Error = N::Error;
    type Service = N::Service;
    type Future = N::Future;
    type MakeError = N::InitError;

    fn make_service(&self, _ctx: Ctx) -> Self::Future {
        self.new_service()
    }
}


// Not exported from crate as this will likely be replaced with `impl Service`.
#[allow(missing_debug_implementations)]
pub struct MakeServiceFn<F, R> {
    f: F,
    _req: PhantomData<fn() -> R>,
}

impl<'a, F, Ctx, Ret, S> MakeService<&'a Ctx> for MakeServiceFn<F, Ret>
where
    F: Fn(&'a Ctx) -> Ret,
    Ret: IntoFuture<Item=S>,
    Ret::Error: Into<Box<StdError + Send + Sync>>,
    S: Service,
{
    type ReqBody = S::ReqBody;
    type ResBody = S::ResBody;
    type Error = S::Error;
    type Service = S;
    type Future = Ret::Future;
    type MakeError = Ret::Error;

    fn make_service(&self, ctx: &'a Ctx) -> Self::Future {
        (self.f)(ctx).into_future()
    }
}

