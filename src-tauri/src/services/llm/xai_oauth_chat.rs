use crate::services::llm_oauth::XaiCatalogModel;

pub(super) struct CatalogChatRequest<'a>(pub(super) super::stream_http::RequestConfig<'a>);

impl<'a> std::ops::Deref for CatalogChatRequest<'a> {
    type Target = super::stream_http::RequestConfig<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub(super) async fn post(
    request: &CatalogChatRequest<'_>,
    measurement: Option<&mut crate::services::provider_usage::RequestMeasurement>,
) -> Result<reqwest::Response, super::stream_http::RequestError> {
    // Ce type fermé empêche le chemin OAuth d'envoyer une requête chat non restreinte.
    super::stream_http::post_chat_request_measured(request, measurement).await
}

pub(super) fn prepare<'request, 'catalog>(
    request: super::stream_http::RequestConfig<'request>,
    model: &'catalog XaiCatalogModel,
) -> CatalogChatRequest<'catalog>
where
    'request: 'catalog,
{
    CatalogChatRequest(super::stream_http::RequestConfig {
        provider_id: request.provider_id,
        model: request.model,
        messages: request.messages,
        tools: request.tools,
        think: request.think,
        reasoning_mode: super::xai_oauth_transport::catalog_reasoning_mode(
            model,
            request.reasoning_mode,
        ),
        max_tokens: request.max_tokens,
        purpose: request.purpose,
        session_id: request.session_id,
        fast_mode: request.fast_mode,
        continuation_target: request.continuation_target,
    })
}
