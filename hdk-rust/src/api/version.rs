use super::Dispatch;
use error::{ZomeApiResult,ZomeApiError};
use holochain_wasm_utils::api_serialization::meta::{MetaArgs,MetaResult,MetaMethod};

pub fn version<S: Into<String>>() -> ZomeApiResult<String> {
    let meta = Dispatch::Meta.with_input(MetaArgs {
        method: MetaMethod::Version,
    })?;
    let version = match meta
    {
        MetaResult::Version(ver) => Ok(ver),
        _=>Err(ZomeApiError::Internal("Wrong Meta Type, Problem In Core".to_string()))
    }?;

    Ok(version)
}