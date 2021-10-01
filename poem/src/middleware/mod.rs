//! Commonly used middleware.

mod add_data;
#[cfg(feature = "compression")]
mod compression;
mod cookie_jar_manager;
mod cors;
mod normalize_path;
mod set_header;
#[cfg(feature = "tower-compat")]
mod tower_compat;
#[cfg(feature = "tracing")]
mod tracing;

pub use add_data::{AddData, AddDataEndpoint};
#[cfg(feature = "compression")]
pub use compression::{Compression, CompressionEndpoint};
pub use cookie_jar_manager::{CookieJarManager, CookieJarManagerEndpoint};
pub use cors::{Cors, CorsEndpoint};
pub use normalize_path::{NormalizePath, NormalizePathEndpoint};
pub use set_header::{SetHeader, SetHeaderEndpoint};
#[cfg(feature = "tower-compat")]
pub use tower_compat::TowerLayerCompatExt;

#[cfg(feature = "tracing")]
pub use self::tracing::{Tracing, TracingEndpoint};
use crate::endpoint::Endpoint;

/// Represents a middleware trait.
///
/// # Example
///
/// ```
/// use poem::{handler, web::Data, Endpoint, EndpointExt, Middleware, Request};
///
/// /// A middleware that extract token from HTTP headers.
/// struct TokenMiddleware;
///
/// impl<E: Endpoint> Middleware<E> for TokenMiddleware {
///     type Output = TokenMiddlewareImpl<E>;
///
///     fn transform(&self, ep: E) -> Self::Output {
///         TokenMiddlewareImpl { ep }
///     }
/// }
///
/// /// The new endpoint type generated by the TokenMiddleware.
/// struct TokenMiddlewareImpl<E> {
///     ep: E,
/// }
///
/// const TOKEN_HEADER: &str = "X-Token";
///
/// /// Token data
/// struct Token(String);
///
/// #[poem::async_trait]
/// impl<E: Endpoint> Endpoint for TokenMiddlewareImpl<E> {
///     type Output = E::Output;
///
///     async fn call(&self, mut req: Request) -> Self::Output {
///         if let Some(value) = req
///             .headers()
///             .get(TOKEN_HEADER)
///             .and_then(|value| value.to_str().ok())
///         {
///             // Insert token data to extensions of request.
///             let token = value.to_string();
///             req.extensions_mut().insert(Token(token));
///         }
///
///         // call the inner endpoint.
///         self.ep.call(req).await
///     }
/// }
///
/// #[handler]
/// async fn index(Data(token): Data<&Token>) -> String {
///     token.0.clone()
/// }
///
/// // Use the `TokenMiddleware` middleware to convert the `index` endpoint.
/// let ep = index.with(TokenMiddleware);
///
/// # tokio::runtime::Runtime::new().unwrap().block_on(async {
/// let mut resp = ep
///     .call(Request::builder().header(TOKEN_HEADER, "abc").finish())
///     .await;
/// assert_eq!(resp.take_body().into_string().await.unwrap(), "abc");
/// # });
/// ```
pub trait Middleware<E: Endpoint> {
    /// New endpoint type.
    ///
    /// If you don't know what type to use, then you can use [`Box<dyn
    /// Endpoint>`], which will bring some performance loss, but it is
    /// insignificant.
    type Output: Endpoint;

    /// Transform the input [`Endpoint`] to another one.
    fn transform(&self, ep: E) -> Self::Output;
}

poem_derive::generate_implement_middlewares!();

/// A middleware implemented by a closure.
pub struct FnMiddleware<T>(T);

impl<T, E, E2> Middleware<E> for FnMiddleware<T>
where
    T: Fn(E) -> E2,
    E: Endpoint,
    E2: Endpoint,
{
    type Output = E2;

    fn transform(&self, ep: E) -> Self::Output {
        (self.0)(ep)
    }
}

/// Make middleware with a closure.
pub fn make<T>(f: T) -> FnMiddleware<T> {
    FnMiddleware(f)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        handler,
        http::{header::HeaderName, HeaderValue, StatusCode},
        web::Data,
        EndpointExt, IntoResponse, Request, Response,
    };

    #[tokio::test]
    async fn test_make() {
        #[handler(internal)]
        fn index() -> &'static str {
            "abc"
        }

        struct AddHeader<E> {
            ep: E,
            header: HeaderName,
            value: HeaderValue,
        }

        #[async_trait::async_trait]
        impl<E: Endpoint> Endpoint for AddHeader<E> {
            type Output = Response;

            async fn call(&self, req: Request) -> Self::Output {
                let mut resp = self.ep.call(req).await.into_response();
                resp.headers_mut()
                    .insert(self.header.clone(), self.value.clone());
                resp
            }
        }

        let ep = index.with(make(|ep| AddHeader {
            ep,
            header: HeaderName::from_static("hello"),
            value: HeaderValue::from_static("world"),
        }));
        let mut resp = ep.call(Request::default()).await;
        assert_eq!(
            resp.headers()
                .get(HeaderName::from_static("hello"))
                .cloned(),
            Some(HeaderValue::from_static("world"))
        );
        assert_eq!(resp.take_body().into_string().await.unwrap(), "abc");
    }

    #[tokio::test]
    async fn test_with_multiple_middlewares() {
        #[handler(internal)]
        fn index(data: Data<&i32>) -> String {
            data.0.to_string()
        }

        let ep = index.with((
            AddData::new(10),
            SetHeader::new().appending("myheader-1", "a"),
            SetHeader::new().appending("myheader-2", "b"),
        ));

        let mut resp = ep.call(Request::default()).await;
        assert_eq!(resp.status(), StatusCode::OK);
        assert_eq!(
            resp.headers().get("myheader-1"),
            Some(&HeaderValue::from_static("a"))
        );
        assert_eq!(
            resp.headers().get("myheader-2"),
            Some(&HeaderValue::from_static("b"))
        );
        assert_eq!(resp.take_body().into_string().await.unwrap(), "10");
    }
}
