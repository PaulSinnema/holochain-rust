use crate::{
    context::Context,
    nucleus::ribosome::callback::{init::init, CallbackParams, CallbackResult},
};
use holochain_core_types::{
    dna::Dna,
    error::{HcResult, HolochainError},
};
use holochain_wasm_utils::api_serialization::init::InitParams;
use std::sync::Arc;

/// Creates a network proxy object and stores DNA and agent hash in the network state.
pub async fn call_init(dna: Dna, context: &Arc<Context>) -> HcResult<()> {
    // map init across every zome. Find which zomes init callback errored, if any
    let errors: Vec<(String, String)> = dna
        .zomes
        .keys()
        .map(|zome_name| {
            let params = context
                .params
                .clone()
                .map(|dna_params| InitParams {
                    params: dna_params.init,
                })
                .unwrap_or(InitParams::default());
            (
                zome_name,
                init(context.clone(), zome_name, &CallbackParams::Init(params)),
            )
        })
        .filter_map(|(zome_name, result)| match result {
            CallbackResult::Fail(error_string) => Some((zome_name.to_owned(), error_string)),
            _ => None,
        })
        .collect();

    if errors.is_empty() {
        Ok(())
    } else {
        Err(HolochainError::ErrorGeneric(format!(
            "At least one zome init returned error: {:?}",
            errors
        )))
    }
}