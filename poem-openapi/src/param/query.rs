use std::ops::{Deref, DerefMut};

use poem::{Request, RequestBody};

use crate::{
    base::UrlQuery,
    registry::{MetaParamIn, MetaSchemaRef, Registry},
    types::ParseFromParameter,
    ApiExtractor, ApiExtractorType, ExtractParamOptions, ParseRequestError,
};

/// Represents the parameters passed by the query string.
pub struct Query<T>(pub T);

impl<T> Deref for Query<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Query<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[poem::async_trait]
impl<'a, T: ParseFromParameter> ApiExtractor<'a> for Query<T> {
    const TYPE: ApiExtractorType = ApiExtractorType::Parameter;
    const PARAM_IS_REQUIRED: bool = T::IS_REQUIRED;

    type ParamType = T;
    type ParamRawType = T::RawValueType;

    fn register(registry: &mut Registry) {
        T::register(registry);
    }

    fn param_in() -> Option<MetaParamIn> {
        Some(MetaParamIn::Query)
    }

    fn param_schema_ref() -> Option<MetaSchemaRef> {
        Some(T::schema_ref())
    }

    fn param_raw_type(&self) -> Option<&Self::ParamRawType> {
        self.0.as_raw_value()
    }

    async fn from_request(
        request: &'a Request,
        _body: &mut RequestBody,
        param_opts: ExtractParamOptions<Self::ParamType>,
    ) -> Result<Self, ParseRequestError> {
        let value = request
            .extensions()
            .get::<UrlQuery>()
            .and_then(|query| query.0.get(param_opts.name).map(|s| s.as_str()));
        let value = match (value, &param_opts.default_value) {
            (Some(value), _) => Some(value),
            (None, Some(default_value)) => return Ok(Self(default_value())),
            (None, _) => None,
        };

        ParseFromParameter::parse_from_parameter(value)
            .map(Self)
            .map_err(|err| ParseRequestError::ParseParam {
                name: param_opts.name,
                reason: err.into_message(),
            })
    }
}
