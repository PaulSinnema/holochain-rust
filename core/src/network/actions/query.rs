use crate::{
    action::{
        Action, ActionWrapper, GetEntryKey, GetLinksKey, QueryKey, QueryPayload
    },
    context::Context,
    instance::dispatch_action,
    network::query::{GetLinksNetworkQuery,NetworkQueryResult}
};
use futures::{future::Future, task::Poll};

use holochain_persistence_api::cas::content::Address;

use holochain_core_types::{crud_status::CrudStatus, error::HcResult, time::Timeout};

use std::{pin::Pin, sync::Arc, thread};

use snowflake::ProcessUniqueId;

use holochain_wasm_utils::api_serialization::get_links::{GetLinksArgs, LinksStatusRequestKind};

/// FetchEntry Action Creator
/// This is the network version of get_entry that makes the network module start
/// a look-up process.
///
/// Returns a future that resolves to an ActionResponse.]

#[derive(Clone, PartialEq, Debug, Serialize)]
pub enum QueryMethod {
    Entry(Address),
    Link(GetLinksArgs, GetLinksNetworkQuery),
}

pub async fn query(
    context: Arc<Context>,
    method: QueryMethod,
    timeout: Timeout,
) -> HcResult<NetworkQueryResult> {
    let (key, payload) = match method {
        QueryMethod::Entry(address) => {
            let key = GetEntryKey {
                address: address,
                id: snowflake::ProcessUniqueId::new().to_string(),
            };
            (QueryKey::Entry(key), QueryPayload::Entry)
        }
        QueryMethod::Link(link_args, query) => {
            let key = GetLinksKey {
                base_address: link_args.entry_address.clone(),
                link_type: link_args.link_type.clone(),
                tag: link_args.tag.clone(),
                id: ProcessUniqueId::new().to_string(),
            };
            let crud_status = match link_args.options.status_request {
                LinksStatusRequestKind::All => None,
                LinksStatusRequestKind::Deleted => Some(CrudStatus::Deleted),
                LinksStatusRequestKind::Live => Some(CrudStatus::Live),
            };
            (
                QueryKey::Links(key.clone()),
                QueryPayload::Links((crud_status, query)),
            )
        }
    };

    let entry = Action::Query((key.clone(), payload.clone()));
    let action_wrapper = ActionWrapper::new(entry);
    dispatch_action(context.action_channel(), action_wrapper.clone());

    let key_inner = key.clone();
    let context_inner = context.clone();
    thread::Builder::new()
        .name(format!("get_timeout/{:?}", key))
        .spawn(move || {
            thread::sleep(timeout.into());
            let timeout_action = Action::QueryTimeout(key_inner);
            let action_wrapper = ActionWrapper::new(timeout_action);
            dispatch_action(context_inner.action_channel(), action_wrapper.clone());
        })
        .expect("Could not spawn thread for get timeout");

    await!(QueryFuture {
        context: context.clone(),
        key: key.clone(),
    })
}

/// GetEntryFuture resolves to a HcResult<Entry>.
/// Tracks the state of the network module
pub struct QueryFuture {
    context: Arc<Context>,
    key: QueryKey,
}

impl Future for QueryFuture {
    type Output = HcResult<NetworkQueryResult>;

    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context) -> Poll<Self::Output> {
        if let Some(err) = self.context.action_channel_error("GetEntryFuture") {
            return Poll::Ready(Err(err));
        }
        if let Err(error) = self
            .context
            .state()
            .expect("Could not get state  in future")
            .network()
            .initialized()
        {
            return Poll::Ready(Err(error));
        }
        //
        // TODO: connect the waker to state updates for performance reasons
        // See: https://github.com/holochain/holochain-rust/issues/314
        //
        cx.waker().clone().wake();
        match self
            .context
            .state()
            .expect("Could not get state in future")
            .network()
            .get_query_results
            .get(&self.key)
        {
            Some(Some(result)) => Poll::Ready(result.clone()),
            _ => Poll::Pending,
        }
    }
}
