use matrix_sdk::RumaApiError;
use ruma::{
    UserId,
    api::{
        EndpointError, IncomingResponse, Metadata, OutgoingRequest,
        auth_scheme::{AccessToken, AuthScheme},
        error::{FromHttpResponseError, IntoHttpError},
        path_builder::PathBuilder,
    },
    exports::{
        bytes::BufMut,
        http::{Request, Response},
        serde_json,
    },
    metadata,
};
use serde::Deserialize;

use crate::{ClientError, client::Client};

metadata! {
    @for MutualRoomsRequest,
    method: GET,
    rate_limited: true,
    authentication: AccessToken,
    history: {
        unstable => "/_matrix/client/v1/mutual_rooms",
    },
}

#[derive(Debug, Clone)]
struct MutualRoomsRequest {
    user_id: String,
    from: Option<String>,
}

impl MutualRoomsRequest {
    fn new(user_id: String, from: Option<String>) -> Self {
        Self { user_id, from }
    }
}

impl OutgoingRequest for MutualRoomsRequest {
    type EndpointError = RumaApiError;
    type IncomingResponse = MutualRoomsResponse;

    fn try_into_http_request<T: Default + BufMut + AsRef<[u8]>>(
        self,
        base_url: &str,
        authentication_input: <Self::Authentication as AuthScheme>::Input<'_>,
        path_builder_input: <Self::PathBuilder as PathBuilder>::Input<'_>,
    ) -> Result<Request<T>, IntoHttpError> {
        let mut query = vec![("user_id", self.user_id.as_str())];
        if let Some(from) = self.from.as_deref() {
            query.push(("from", from));
        }

        let query =
            url::form_urlencoded::Serializer::new(String::new()).extend_pairs(query).finish();
        let url = Self::make_endpoint_url(path_builder_input, base_url, &[], &query)?;

        let mut request = Request::builder().method(Self::METHOD).uri(url).body(T::default())?;
        Self::Authentication::add_authentication(&mut request, authentication_input)
            .map_err(|err| IntoHttpError::Authentication(err.into()))?;

        Ok(request)
    }
}

#[derive(Clone, Deserialize, uniffi::Record)]
pub struct MutualRoomsResponse {
    pub count: u64,
    pub joined: Vec<String>,
    pub next_batch: Option<String>,
}

impl IncomingResponse for MutualRoomsResponse {
    type EndpointError = RumaApiError;

    fn try_from_http_response<T: AsRef<[u8]>>(
        response: Response<T>,
    ) -> Result<Self, FromHttpResponseError<Self::EndpointError>> {
        if response.status().is_success() {
            Ok(serde_json::from_slice::<Self>(response.body().as_ref())?)
        } else {
            Err(FromHttpResponseError::Server(Self::EndpointError::from_http_response(response)))
        }
    }
}

#[matrix_sdk_ffi_macros::export]
impl Client {
    pub async fn get_mutual_rooms(
        &self,
        user_id: String,
        from: Option<String>,
    ) -> Result<MutualRoomsResponse, ClientError> {
        let user_id = UserId::parse(user_id)?;
        Ok(self.inner.send(MutualRoomsRequest::new(user_id.to_string(), from)).await?)
    }
}
